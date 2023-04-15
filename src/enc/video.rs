use super::QuadTree;

use std::{
    cmp::min,
    io::{Result as IoResult, Write},
};

pub struct VideoEncoder<W: Write> {
    writer: W,
    buf: [u8; 1024],
    cursor: usize,
    leaf_buf: Vec<u8>,
    i_frame_interval: u16,
    frame_counter: u16,
    previous_tree: Option<QuadTree>
}

impl<W: Write> VideoEncoder<W> {
    pub fn new(writer: W, i_frame_interval: u16) -> Self {
        Self {
            writer,
            buf: [0; 1024],
            cursor: 0,
            leaf_buf: Vec::with_capacity(512),
            i_frame_interval,
            frame_counter: i_frame_interval,
            previous_tree: None,
        }
    }

    fn encode_buf(&mut self) -> IoResult<()> {
        if self.frame_counter < self.i_frame_interval {
            self.frame_counter += 1;
            self.encode_p_frame()
        } else {
            self.frame_counter = 1;
            self.encode_i_frame()
        }
    }

    fn encode_i_frame(&mut self) -> IoResult<()> {
        let tree = QuadTree::from_128x64(&self.buf);
        let len = tree.store_packed(&mut self.leaf_buf)?;

        self.previous_tree = Some(tree);
        self.flush(0, len)?;

        self.leaf_buf.clear();
        Ok(())
    }

    fn encode_p_frame(&mut self) -> IoResult<()> {
        let tree = QuadTree::from_128x64(&self.buf);
        if let Some(prev) = self.previous_tree.take() {
            let diff = tree.diff(&prev);

            let (len_y, len_n) = diff.store_as_diff(&mut self.leaf_buf)?;

            let mut tmp = Vec::with_capacity(1024);
            let full_len = tree.store_packed(&mut tmp)?;

            self.previous_tree = Some(tree);

            if full_len < self.leaf_buf.len() {
                self.cursor = 0;
                self.writer.write_all(&(full_len as u16).to_le_bytes())?;
                self.writer.write_all(&tmp)?;
            } else {
                self.flush(0, len_y)?;
                self.flush(len_y, len_n)?
            }
        }

        self.leaf_buf.clear();
        Ok(())
    }

    fn flush(&mut self, start: usize, len: usize) -> IoResult<()> {
        self.cursor = 0;

        self.writer.write_all(&(len as u16).to_le_bytes())?;
        self.writer.write_all(&self.leaf_buf[start..start + len])?;

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
