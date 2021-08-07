use std::{cmp::min, io::{Result as IoResult, Write}};

use super::LinearQuadTree;

pub struct VideoEncoder<W: Write> {
    writer: W,
    buf: [u8; 1024],
    cursor: usize,
    leaf_buf: Vec<u8>,
}

impl<W: Write> VideoEncoder<W> {
    pub fn new(writer: W) -> Self {
        Self { 
            writer,
            buf: [0; 1024],
            cursor: 0,
            leaf_buf: Vec::with_capacity(512)
        }
    }

    fn encode_buf(&mut self) -> IoResult<()> {
        let mut encoder = LinearQuadTree::new(&mut self.leaf_buf);
        let len = encoder.parse_slice_12864(&self.buf)?;
        self.cursor = 0;

        self.writer.write_all(&(len as u16).to_le_bytes())?;
        self.writer.write_all(&self.leaf_buf)?;

        self.leaf_buf.clear();

        Ok(())
    }
}

impl<W: Write> Write for VideoEncoder<W> {
    fn flush(&mut self) -> IoResult<()> {
        self.buf[self.cursor..].fill(0);

        self.encode_buf()
    }

    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        let to_write = min(self.buf.len() - self.cursor, buf.len());
        self.buf[self.cursor..self.cursor + to_write].copy_from_slice(&buf[..to_write]);
        self.cursor += to_write;

        self.encode_buf()?;

        Ok(to_write)
    }
}
