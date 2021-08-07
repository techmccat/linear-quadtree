use std::io::{Result, Write};

use bitvec::prelude::*;

pub mod video;

#[cfg(test)]
pub mod tests;

type BitVecU8 = BitVec<Msb0, u8>;

pub struct LinearQuadTree<W: Write = Vec<u8>> {
    data: W,
    count: usize,
    position: Vec<u8>,
}

impl<W: Write> LinearQuadTree<W> {
    pub fn new(data: W) -> Self {
        Self {
            data,
            count: 0,
            position: Vec::new(),
        }
    }

    pub fn parse_slice_12864(&mut self, slice: &[u8]) -> Result<usize> {
        if compare_bits(slice.view_bits()) {
            if slice[0] == u8::MAX {
                self.count += self.data.write(&[0b1_000_00_00])?;
            }
        } else {
            let top = Frame::new(BitVecU8::from_slice(slice).unwrap(), 128);
            let (left, right) = top.split_v();
            self.parse_frame(left, 0)?;
            self.parse_frame(right, 1)?;
        }

        Ok(self.count)
    }

    fn parse_frame(&mut self, f: Frame, pos: u8) -> Result<()> {
        //println!("Parsing frame of {} bytes", f.buf.len());
        self.position.push(pos);

        if f.uniform() {
            if f.color() {
                let depth = self.position.len();
                let mut data = [0u8; 2];
                let bits = data.view_bits_mut::<Msb0>();

                if depth > 7 {
                    panic!("Depth exceeded maximum available")
                }
                if depth == 7 {
                    for (i, p) in self.position.iter().enumerate() {
                        bits[2..][i * 2..=i * 2 + 1].store(*p);
                    }
                } else {
                    bits[..1].store(1u8);
                    bits[1..4].store(depth);

                    for (i, p) in self.position.iter().enumerate() {
                        bits[4..][i * 2..=i * 2 + 1].store(*p)
                    }
                }

                if depth > 2 {
                    self.count += self.data.write(&data)?;
                } else {
                    self.count += self.data.write(&data[..1])?;
                }
            }

            self.position.pop();
        } else {
            let (tl, tr, bl, br) = f.split_four();
            self.parse_frame(tl, 0)?;
            self.parse_frame(tr, 1)?;
            self.parse_frame(bl, 2)?;
            self.parse_frame(br, 3)?;

            self.position.pop();
        }

        Ok(())
    }
}

fn compare_bytes(buf: &[u8]) -> bool {
    let mut prev = buf[0];
    for b in buf {
        if *b != prev {
            return false;
        } else {
            prev = *b
        }
    }

    true
}

fn compare_bits(buf: &BitSlice<Msb0, u8>) -> bool {
    if buf.len() == 1 {
        return true;
    }

    let mut prev = buf[0];
    for b in buf {
        if *b != prev {
            return false;
        } else {
            prev = *b
        }
    }

    true
}

struct Frame {
    side: usize,
    buf: BitVecU8,
}

impl Frame {
    pub fn new(buf: BitVecU8, side: usize) -> Self {
        Self { side, buf }
    }

    pub fn uniform(&self) -> bool {
        if self.buf.len() >= 16 {
            let bytes = self.buf.as_raw_slice();
            if bytes[0] == u8::MAX {
                return compare_bytes(self.buf.as_raw_slice())
            }
        }
        compare_bits(self.buf.as_ref())
    }

    pub fn color(&self) -> bool {
        self.buf[0]
    }

    fn split_h(&self) -> (Frame, Frame) {
        let half = self.buf.len() / 2;
        let (top, bot) = (self.buf[0..half].to_bitvec(), self.buf[half..].to_bitvec());
        (Self::new(top, self.side), Self::new(bot, self.side))
    }

    fn split_v(&self) -> (Frame, Frame) {
        let new_side = self.side / 2;
        let new_cap = self.buf.len() / 2;
        let (mut left, mut right) = (
            BitVecU8::with_capacity(new_cap),
            BitVecU8::with_capacity(new_cap),
        );

        for i in 0..new_side {
            let start = i * self.side;
            let mid = start + new_side;
            let end = start + self.side;

            left.extend_from_bitslice(&self.buf[start..mid]);
            right.extend_from_bitslice(&self.buf[mid..end]);
        }

        (Self::new(left, new_side), Self::new(right, new_side))
    }

    pub fn split_four(&self) -> (Frame, Frame, Frame, Frame) {
        let (top, bot) = self.split_h();
        let (tl, tr) = top.split_v();
        let (bl, br) = bot.split_v();
        (tl, tr, bl, br)
    }
}
