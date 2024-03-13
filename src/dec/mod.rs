use crate::{FrameMeta, Leaf, LeafData, utils::next_pos};

use bitvec::prelude::*;
use core::{
    cmp,
    convert::{TryFrom, TryInto},
};
use embedded_graphics::{
    image::{Image, ImageDrawable, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
};

pub mod video;

#[cfg(test)]
mod tests;

impl Dimensions for Leaf {
    fn bounding_box(&self) -> Rectangle {
        let es = 1 << (7 - self.depth());
        let mut x = 0;
        let mut y = 0;

        for (i, p) in self.pos.iter().enumerate() {
            x |= (p & 1) << 6 - i;
            y |= (p >> 1) << 6 - i;
        }
        let point = Point::new(x as i32, y as i32);
        let size = Size::new_equal(es);

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
        match self.data {
            LeafData::Feature(f) => target.fill_solid(&self.bounding_box(), f.into()),
            LeafData::Bitmap(b) => {
                let data = [b[0] & 0xf0, b[0] << 4, b[1] & 0xf0, b[1] << 4];
                let raw = ImageRaw::<BinaryColor>::new(&data, 4);
                let image = Image::new(&raw, self.bounding_box().top_left);
                image.draw(target)
            }
        }
    }
}

impl Leaf {
    fn draw_sub_image<DT>(&self, target: &mut DT, area: &Rectangle) -> Result<(), DT::Error>
    where
        DT: DrawTarget<Color = <Self as Drawable>::Color>,
    {
        match self.data {
            LeafData::Feature(f) => target.fill_solid(area, f.into()),
            LeafData::Bitmap(b) => {
                let data = [b[0] & 0xf0, b[0] << 4, b[1] & 0xf0, b[1] << 4];
                let raw = ImageRaw::<BinaryColor>::new(&data, 4);
                raw.draw_sub_image(target, area)
            }
        }
    }
}

pub struct DrawWrapper<D>(D);

pub trait Decoder<'a>: Sized {
    type Iterator: Iterator<Item = Leaf>;
    fn from_buf(buf: &'a [u8]) -> Result<Self, ParseError>;
    fn iter(&self) -> Self::Iterator;
    fn flush_after(&self) -> bool;
    fn clear_framebuffer(&self) -> Option<BinaryColor>;
    fn drawable(self) -> DrawWrapper<Self> {
        DrawWrapper(self)
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidHeader,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LeafParserV1<'a> {
    buf: &'a [u8],
    meta: FrameMeta,
}

impl<'a> LeafParserV1<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self, ParseError> {
        match buf.get(0).map(|meta| FrameMeta::try_from(*meta)) {
            Some(Ok(meta)) => Ok(Self {
                buf: &buf[1..],
                meta,
            }),
            _ => Err(ParseError::InvalidHeader),
        }
    }
}

impl<'a> Decoder<'a> for LeafParserV1<'a> {
    type Iterator = LeafParserIterV1<'a>;
    fn from_buf(buf: &'a [u8]) -> Result<Self, ParseError> {
        Self::new(buf)
    }
    fn iter(&self) -> LeafParserIterV1<'a> {
        LeafParserIterV1 {
            buf: self.buf,
            index: 0,
            feature: self.meta.active_feature,
        }
    }
    fn flush_after(&self) -> bool {
        self.meta.display
    }
    fn clear_framebuffer(&self) -> Option<BinaryColor> {
        if !self.meta.partial {
            Some(BinaryColor::from(!self.meta.active_feature))
        } else {
            None
        }
    }
}

pub struct LeafParserIterV1<'a> {
    buf: &'a [u8],
    index: usize,
    feature: bool,
}

impl<'a> Iterator for LeafParserIterV1<'a> {
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
            let next = self.buf.get(self.index);
            let next = if let Some(b) = next {
                b.view_bits::<Msb0>()
            } else {
                return None;
            };

            for i in 3..=cmp::min(depth, 5) as usize {
                let bitpos = (i - 3) * 2;
                let side = next[bitpos..=bitpos + 1].load();
                pos.push(side).expect("Exceeded max depth");
            }
        }

        self.index += 1;

        let data;
        if depth < 6 {
            data = LeafData::Feature(self.feature)
        } else {
            if self.buf.len() < self.index + 2 {
                return None;
            }
            data = LeafData::Bitmap(self.buf[self.index..self.index + 2].try_into().unwrap());
            self.index += 2;
        };

        Some(Self::Item { pos, data })
    }
}

impl<'a, D: Decoder<'a>> OriginDimensions for DrawWrapper<D> {
    fn size(&self) -> Size {
        Size::new_equal(2u32.pow(7))
    }
}

impl<'a, D: Decoder<'a>> ImageDrawable for DrawWrapper<D> {
    type Color = BinaryColor;

    fn draw<DT>(&self, target: &mut DT) -> Result<(), DT::Error>
    where
        DT: DrawTarget<Color = Self::Color>,
    {
        if let Some(c) = self.0.clear_framebuffer() {
            target.clear(c)?;
        }

        for leaf in self.0.iter() {
            leaf.draw(target)?
        }

        Ok(())
    }

    fn draw_sub_image<DT>(&self, target: &mut DT, area: &Rectangle) -> Result<(), DT::Error>
    where
        DT: DrawTarget<Color = Self::Color>,
    {
        for leaf in self.0.iter() {
            let rect = leaf.bounding_box().intersection(area);

            if !rect.is_zero_sized() {
                leaf.draw_sub_image(target, &rect)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeafParserV2<'a> {
    buf: &'a [u8]
}

impl<'a> Decoder<'a> for LeafParserV2<'a> {
    type Iterator = LeafParserIterV2<'a>;
    fn from_buf(buf: &'a [u8]) -> Result<Self, ParseError> {
        Ok(Self{ buf })
    }
    fn iter(&self) -> Self::Iterator {
        LeafParserIterV2 {
            inner: self.buf.view_bits().chunks_exact(2),
            pos: Default::default(),
        }
    }
    fn flush_after(&self) -> bool {
        true
    }
    fn clear_framebuffer(&self) -> Option<BinaryColor> {
        None
    }
}

pub struct LeafParserIterV2<'a> {
    inner: bitvec::slice::ChunksExact<'a, u8, Msb0>,
    pos: crate::Position,
}

impl Iterator for LeafParserIterV2<'_> {
    type Item = Leaf;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(bits) = self.inner.next() {
            match bits.load::<u8>() {
                0b00 => next_pos(&mut self.pos)?, // empty node
                0b01 => self.pos.push(0).expect("Max depth exceeded"), // branch
                0b10 => { // leaf, value = 0
                    let pos = self.pos.clone();
                    next_pos(&mut self.pos)?;
                    return Some(Leaf::new(LeafData::Feature(false), pos))
                },
                0b11 => { // leaf, value = 1
                    let pos = self.pos.clone();
                    next_pos(&mut self.pos)?;
                    return Some(Leaf::new(LeafData::Feature(true), pos))
                }
                _ => unreachable!(),
            }
        }
        None
    }
}
