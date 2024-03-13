use super::{BitVecU8, QuadTree};

use std::{
    cmp::min,
    io::{Result as IoResult, Write},
};

pub trait Encode: Sized + Default {
    fn encode_i_frame<W: Write>(&mut self, buf: &[u8; 1024], w: W) -> IoResult<()>;
    fn encode_p_frame<W: Write>(&mut self, buf: &[u8; 1024], w: W) -> IoResult<()>;
}

pub struct VideoEncoder<W, E> {
    writer: W,
    encoder: E,
    buf: [u8; 1024],
    cursor: usize,
    i_frame_interval: u16,
    frame_counter: u16,
}

#[derive(Debug, Default)]
pub struct EncoderV1 {
    previous_tree: Option<QuadTree>,
}

impl Encode for EncoderV1 {
    fn encode_i_frame<W: Write>(&mut self, buf: &[u8; 1024], mut w: W) -> IoResult<()> {
        let tree = QuadTree::from_128x64(buf, true);
        let mut leaf_buf = Vec::with_capacity(1024);
        let len = tree.store_packed(&mut leaf_buf)?;
        self.previous_tree = Some(tree);

        w.write_all(&(len as u16).to_le_bytes())?;
        w.write_all(&leaf_buf)
    }
    fn encode_p_frame<W: Write>(&mut self, buf: &[u8; 1024], mut w: W) -> IoResult<()> {
        let tree = QuadTree::from_128x64(buf, true);
        Ok(if let Some(prev) = self.previous_tree.take() {
            let diff = tree.diff(&prev);
            let mut leaf_buf = Vec::with_capacity(1024);

            let (len_y, len_n) = diff.store_as_diff(&mut leaf_buf)?;

            let mut tmp = Vec::with_capacity(1024);
            let full_len = tree.store_packed(&mut tmp)?;


            let res = if full_len < leaf_buf.len() {
                w.write_all(&(full_len as u16).to_le_bytes())?;
                w.write_all(&tmp)?;
            } else {
                w.write_all(&(len_y as u16).to_le_bytes())?;
                w.write_all(&leaf_buf[..len_y])?;
                w.write_all(&(len_n as u16).to_le_bytes())?;
                w.write_all(&leaf_buf[len_y..len_y + len_n])?;
            };
            self.previous_tree = Some(tree);
            res
        } else {
            self.encode_p_frame(buf, w)?;
        })
    }
}

#[derive(Debug, Default)]
pub struct EncoderV2 {
    previous_tree: Option<QuadTree>,
}

fn write_bits(bits: &BitVecU8, mut w: impl Write) -> IoResult<()> {
    let bytes = bits.as_raw_slice();
    let len = bytes.len();
    w.write_all(&(len as u16).to_le_bytes())?;
    w.write_all(bytes)
}

impl Encode for EncoderV2 {
    fn encode_i_frame<W: Write>(&mut self, buf: &[u8; 1024], w: W) -> IoResult<()> {
        let tree = QuadTree::from_128x64(buf, false);
        write_bits(&tree.collect_compact().unwrap(), w)?;
        self.previous_tree = Some(tree);
        Ok(())
    }
    fn encode_p_frame<W: Write>(&mut self, buf: &[u8; 1024], w: W) -> IoResult<()> {
        if let Some(prev) = self.previous_tree.take() {
            let tree = QuadTree::from_128x64(buf, false);
            let diff = tree.diff(&prev);
            self.previous_tree = Some(tree);
            write_bits(&diff.collect_compact().unwrap(), w)
        } else {
            eprintln!("iframe fallback");
            self.encode_i_frame(buf, w)
        }
    }
}

impl<W: Write, E: Encode> VideoEncoder<W, E> {
    pub fn new(writer: W, i_frame_interval: u16) -> Self {
        Self {
            writer,
            encoder: Default::default(),
            buf: [0; 1024],
            cursor: 0,
            i_frame_interval,
            frame_counter: i_frame_interval,
        }
    }

    fn encode_buf(&mut self) -> IoResult<()> {
        self.cursor = 0;
        if self.frame_counter < self.i_frame_interval {
            self.frame_counter += 1;
            self.encoder.encode_p_frame(&self.buf, &mut self.writer)
        } else {
            self.frame_counter = 1;
            self.encoder.encode_i_frame(&self.buf, &mut self.writer)
        }
    }
}

impl<W: Write, E: Encode> Write for VideoEncoder<W, E> {
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
