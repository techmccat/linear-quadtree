use crate::Leaf;

use bitvec::prelude::*;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, primitives::Rectangle};

pub mod video;

#[cfg(test)]
mod tests;

impl Leaf {
    pub fn feature(&self) -> BinaryColor {
        self.feature.into()
    }
}

impl Dimensions for Leaf {
    fn bounding_box(&self) -> Rectangle {
        let mut s = 128;
        let mut x = 0;
        let mut y = 0;

        for p in &self.pos {
            s /= 2;
            match p {
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
        
        Rectangle::new(point, size)
    }
}

impl Drawable for Leaf {
    type Color = BinaryColor;
    type Output = ();

    fn draw<DT>(&self, target: &mut DT) -> Result<Self::Output, DT::Error>
    where
        DT: DrawTarget<Color = Self::Color>,
    {
        let Rectangle{top_left: point, size} = self.bounding_box();
        let rect = if self.pos.len() == 0 { 
            Rectangle::new(point, size).intersection(&target.bounding_box())
        } else {
            Rectangle::new(point, size)
        };
        target.fill_solid(&rect, self.feature.into())
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidHeader
}

#[derive(Debug, PartialEq)]
pub struct LeafParser<'a> {
    buf: &'a [u8],
    feature: bool,
}

impl<'a> LeafParser<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self, ParseError> {
        match buf.get(0) {
            Some(1) => Ok(Self { buf, feature: true }),
            Some(0) => Ok(Self { buf, feature: false }),
            _ => Err(ParseError::InvalidHeader)
        }
    }
}

impl<'a> IntoIterator for &'a LeafParser<'a> {
    type IntoIter = LeafParserIter<'a>;
    type Item = Leaf;

    fn into_iter(self) -> Self::IntoIter {
        LeafParserIter {
            buf: &self.buf[1..],
            index: 0,
            feature: self.feature,
        }
    }
}

pub struct LeafParserIter<'a> {
    buf: &'a [u8],
    index: usize,
    feature: bool,
}

impl<'a> Iterator for LeafParserIter<'a> {
    type Item = Leaf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() < self.index + 1 {
            return None;
        };

        let byte = self.buf[self.index];
        let cur = byte.view_bits::<Msb0>();

        let mut pos = heapless::Vec::new();

        let (depth, base_index) = if cur[0] {
            let depth: u8 = cur[1..=3].load();
            (depth, 1)
        } else {
            (7, 0)
        };

        for i in base_index..3 {
            let bitpos = (i + 1) * 2;
            let side = cur[bitpos..=bitpos + 1].load();
            pos.push(side).expect("Exceeded max depth");
        }

        pos.truncate(depth as usize);

        if depth > 2 {
            self.index += 1;
            let next = self.buf[self.index].view_bits::<Msb0>();

            for i in 3..=(if depth < 7 { depth } else { 6 }) as usize {
                let bitpos = (i - 3) * 2;
                let side = next[bitpos..=bitpos + 1].load();
                pos.push(side).expect("Exceeded max depth");
            }
        }

        self.index += 1;

        Some(Self::Item {
            pos,
            feature: self.feature
        })
    }
}

impl OriginDimensions for LeafParser<'_> {
    fn size(&self) -> Size {
        Size::new_equal(2u32.pow(7))
    }
}

impl ImageDrawable for LeafParser<'_> {
    type Color = BinaryColor;

    fn draw<DT>(&self, target: &mut DT) -> Result<(), DT::Error>
    where
        DT: DrawTarget<Color = Self::Color>,
    {
        target.clear(Self::Color::from(!self.feature))?;

        for leaf in self.into_iter() {
            leaf.draw(target)?
        }

        Ok(())
    }

    // TODO: z-curves are a thing, I should use them
    fn draw_sub_image<DT>(&self, target: &mut DT, area: &Rectangle) -> Result<(), DT::Error>
    where
        DT: DrawTarget<Color = Self::Color>,
    {
        for leaf in self.into_iter() {
            let rect = leaf.bounding_box().intersection(area);

            if !rect.is_zero_sized() {
                target.fill_solid(&rect, leaf.feature.into())?
            }
        }
        Ok(())
    }
}
