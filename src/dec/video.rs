use core::convert::TryInto;
use core::mem::size_of;

pub struct VideoSlice<'a> {
    buf: &'a [u8],
    index: usize,
}

impl<'a> Iterator for VideoSlice<'a> {
    type Item = super::LeafParser<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptrsize = size_of::<usize>();
        if self.buf.len() > self.index + ptrsize {
            let frame_size = u16::from_le_bytes(
                self.buf[self.index..self.index + ptrsize]
                    .try_into()
                    .unwrap()
            ) as usize;

            let start = self.index + ptrsize;
            let end = start + frame_size;
            self.index = end;

            if self.buf.len() < end {
                Self::Item::new(&self.buf[start..end]).ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}
