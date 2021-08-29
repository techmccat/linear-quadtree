use crate::Leaf;

use bitvec::prelude::*;
use std::io::{Result as IoResult, Write};

pub mod video;

#[cfg(test)]
pub mod tests;

type BitVecU8 = BitVec<Msb0, u8>;

impl Leaf {
    fn write<W: std::io::Write>(&self, mut w: W) -> IoResult<usize> {
        let depth = self.pos.len();
        let mut data = [0u8; 2];
        let bits = data.view_bits_mut::<Msb0>();

        if depth == 7 {
            for (i, p) in self.pos.iter().enumerate() {
                bits[2..][i * 2..=i * 2 + 1].store(*p);
            }
        } else {
            bits[..1].store(1u8);
            bits[1..4].store(depth);

            for (i, p) in self.pos.iter().enumerate() {
                bits[4..][i * 2..=i * 2 + 1].store(*p)
            }
        }

        if depth > 2 {
            w.write(&data)
        } else {
            w.write(&data[..1])
        }
    }
}

#[derive(Default)]
pub struct LinearQuadTree(Vec<Leaf>);

impl LinearQuadTree {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    pub fn parse_12864(&mut self, buf: &[u8; 1024]) {
        let f = Frame::new(BitVecU8::from_slice(buf).unwrap(), 128);
        let mut parser = BulkParse::new(&mut self.0);
        parser.parse_12864(f)
    }

    pub fn store_packed<W: Write>(&self, mut w: W) -> IoResult<usize> {
        let yes = self.0.iter().filter(|l| l.feature);
        let no = self.0.iter().filter(|l| !l.feature);

        let mut count = 1;

        if yes.clone().count() < no.clone().count() {
            w.write_all(&[1])?;
            for leaf in yes {
                count += leaf.write(&mut w)?;
            }
        } else {
            w.write_all(&[0])?;
            for leaf in no {
                count += leaf.write(&mut w)?;
            }
        }

        Ok(count)
    }
}

struct BulkParse<'a> {
    pos: heapless::Vec<u8, 7>,
    out: &'a mut Vec<Leaf>,
}

impl<'a> BulkParse<'a> {
    pub fn new(out: &'a mut Vec<Leaf>) -> Self {
        Self {
            pos: Default::default(),
            out,
        }
    }

    pub fn parse_12864(&mut self, f: Frame) {
        if f.uniform() {
            self.out.push(Leaf::new(f.color(), heapless::Vec::new()))
        } else {
            let (left, right) = f.split_v();
            self.parse_frame(left, 0);
            self.parse_frame(right, 1);
        }
    }

    pub fn parse_frame(&mut self, f: Frame, pos: u8) {
        self.pos
            .push(pos)
            .expect("Depth exceeded maximum available");

        if f.uniform() {
            self.out.push(Leaf::new(f.color(), self.pos.clone()));
        } else {
            let (tl, tr, bl, br) = f.split_four();
            self.parse_frame(tl, 0);
            self.parse_frame(tr, 1);
            self.parse_frame(bl, 2);
            self.parse_frame(br, 3);
        }

        self.pos.pop();
    }
}

fn compare_bytes(buf: &[u8]) -> bool {
    let mut prev = buf[0];

    if !(prev == u8::MAX || prev == 0) {
        return false;
    }

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
    if buf.len() >= 16 {
        let bytes = buf.as_raw_slice();
        return compare_bytes(bytes);
    }

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

pub struct Frame {
    side: usize,
    buf: BitVecU8,
}

impl Frame {
    pub fn new(buf: BitVecU8, side: usize) -> Self {
        Self { side, buf }
    }

    pub fn uniform(&self) -> bool {
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
