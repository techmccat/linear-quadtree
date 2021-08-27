use core::convert::TryInto;

pub struct VideoSlice<'a> {
    buf: &'a [u8],
    index: usize,
}

impl<'a> VideoSlice<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, index: 0 }
    }
}

impl<'a> Iterator for VideoSlice<'a> {
    type Item = super::LeafParser<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.index + 2;

        if self.buf.len() >= start {
            let frame_size =
                u16::from_le_bytes(self.buf[self.index..start].try_into().unwrap()) as usize;

            let end = start + frame_size;
            self.index = end;

            if self.buf.len() >= end {
                Self::Item::new(&self.buf[start..end]).ok()
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
    use crate::dec::LeafParser;

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

        let expect = [
            LeafParser::new(&buf[2..3]).unwrap(),
            LeafParser::new(&buf[5..8]).unwrap(),
            LeafParser::new(&buf[10..14]).unwrap(),
        ];

        for (l, r) in expect.iter().zip(VideoSlice::new(&buf)) {
            assert_eq!(*l, r)
        }
    }
}
