use core::{convert::TryInto, marker::PhantomData};

use super::Decoder;

#[derive(Debug)]
pub struct VideoSlice<'a, D> {
    buf: &'a [u8],
    index: usize,
    _dec: PhantomData<D>,
}

impl<'a, D: Decoder<'a>> VideoSlice<'a, D> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            index: 0,
            _dec: PhantomData,
        }
    }
}

impl<'a, D: Decoder<'a>> Iterator for VideoSlice<'a, D> {
    type Item = D;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.index + 2;

        if self.buf.len() >= start {
            let frame_size =
                u16::from_le_bytes(self.buf[self.index..start].try_into().unwrap()) as usize;

            let end = start + frame_size;
            self.index = end;

            if self.buf.len() >= end {
                Self::Item::from_buf(&self.buf[start..end]).ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dec::{LeafParserV1, LeafParserV2};

    #[test]
    fn parse() {
        #[rustfmt::skip]
        let buf = [
            1, 0,
            0, // garbage, just need to check
            3, 0,
            0, 0, 0,
            4, 0,
            0, 0, 0, 0
        ];

        let expect_v1 = [
            LeafParserV1::new(&buf[2..3]).unwrap(),
            LeafParserV1::new(&buf[5..8]).unwrap(),
            LeafParserV1::new(&buf[10..14]).unwrap(),
        ];

        for (l, r) in expect_v1.iter().zip(VideoSlice::<LeafParserV1>::new(&buf)) {
            assert_eq!(*l, r)
        }

        let expect_v2 = [
            LeafParserV2::from_buf(&buf[2..3]).unwrap(),
            LeafParserV2::from_buf(&buf[5..8]).unwrap(),
            LeafParserV2::from_buf(&buf[10..14]).unwrap(),
        ];
        for (l, r) in expect_v2.iter().zip(VideoSlice::<LeafParserV2>::new(&buf)) {
            assert_eq!(*l, r)
        }
    }
}
