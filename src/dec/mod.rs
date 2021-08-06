use bitvec::prelude::*;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, primitives::Rectangle};

pub mod video;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq)]
pub struct ParsedLeaf {
    depth: u8,
    pos: [u8; 7],
    feature: bool,
}

impl ParsedLeaf {
    pub fn position_and_size(&self) -> (Point, Size) {
        let mut s = 128;
        let mut x = 0;
        let mut y = 0;

        for i in 0..self.depth as usize {
            s /= 2;
            match self.pos[i] {
                0 => (),
                1 => x += s,
                2 => y += s,
                3 => {
                    x += s;
                    y += s
                }
                _ => (),
            }
        }

        let point = Point::new(x as i32, y as i32);
        let size = Size::new_equal(s);
        
        (point, size)
    }

    pub fn rectangle(&self) -> Rectangle {
        let (point, size) = self.position_and_size();
        let base = Rectangle::new(Point::zero(), Size::new(128, 64));
        Rectangle::new(point, size).intersection(&base)
    }

    pub fn color(&self) -> BinaryColor {
        self.feature.into()
    }
}

impl OriginDimensions for ParsedLeaf {
    fn size(&self) -> Size {
        if self.depth == 0 {
            Size::new(128, 64)
        } else {
            let s = 128 / (2u32.pow(self.depth as u32));
            Size::new_equal(s as u32)
        }
    }
}

impl Drawable for ParsedLeaf {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let (point, size) = self.position_and_size();
        let rect = if self.depth == 0 { 
            Rectangle::new(point, size).intersection(&target.bounding_box())
        } else {
            Rectangle::new(point, size)
        };
        target.fill_solid(&rect, self.color())
    }
}

pub struct LeafParser<'a> {
    buf: &'a [u8],
}

impl<'a> LeafParser<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

}

impl<'a> IntoIterator for &'a LeafParser<'a> {
    type IntoIter = LeafParserIter<'a>;
    type Item = ParsedLeaf;

    fn into_iter(self) -> Self::IntoIter {
        LeafParserIter {
            buf: self.buf,
            index: 0,
        }
    }
}

pub struct LeafParserIter<'a> {
    buf: &'a [u8],
    index: usize,
}

impl<'a> Iterator for LeafParserIter<'a> {
    type Item = ParsedLeaf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() < self.index + 1 {
            return None;
        };

        let byte = self.buf[self.index];
        let cur = byte.view_bits::<Msb0>();

        let mut pos = [0; 7];

        let (depth, base_index) = if cur[0] {
            let depth = cur[1..=3].load();
            (depth, 1)
        } else {
            (7, 0)
        };

        for i in base_index..3 {
            let bitpos = (i + 1) * 2;
            pos[i - base_index] = cur[bitpos..=bitpos + 1].load();
        }

        if depth > 2 {
            self.index += 1;
            let next = self.buf[self.index].view_bits::<Msb0>();

            for i in 3..=(if depth < 7 { depth } else { 6 }) as usize {
                let bitpos = (i - 3) * 2;
                pos[i - base_index] = next[bitpos..=bitpos + 1].load();
            }
        }

        self.index += 1;

        let ret = Some(Self::Item {
            depth,
            pos,
            feature: true,
        });

        ret
    }
}

impl OriginDimensions for LeafParser<'_> {
    fn size(&self) -> Size {
        Size::new(128, 64)
    }
}

impl ImageDrawable for LeafParser<'_> {
    type Color = BinaryColor;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        for leaf in self.into_iter() {
            leaf.draw(target)?
        }

        Ok(())
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        for leaf in self.into_iter() {
            let rect = leaf.rectangle().intersection(area);

            if !rect.is_zero_sized() {
                target.fill_solid(&rect, leaf.color())?
            }
        }
        Ok(())
    }
}
