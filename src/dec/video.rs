use core::convert::TryInto;
use core::mem::size_of;

pub struct LeafSlice<'a> {
    buf: &'a [u8],
    index: usize,
}

impl<'a> Iterator for LeafSlice<'a> {
    type Item = super::LeafParser<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptrsize = size_of::<usize>();
        if self.buf.len() > self.index + ptrsize {
            let frame_size = usize::from_le_bytes(
                self.buf[self.index..self.index + ptrsize]
                    .try_into()
                    .unwrap(),
            );

            let start = self.index + ptrsize;
            let end = start + frame_size;
            self.index = end;

            if self.buf.len() < end {
                Some(Self::Item::new(&self.buf[start..end]))
            } else {
                None
            }
        } else {
            None
        }
    }
}
