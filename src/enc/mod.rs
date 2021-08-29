use crate::Leaf;

use bitvec::prelude::*;
use embedded_graphics::{prelude::Point, primitives::Rectangle};
use std::{
    io::{Result as IoResult, Write},
    ops::Index,
};

pub mod video;

#[cfg(test)]
pub mod tests;

type BitSliceU8 = BitSlice<Msb0, u8>;

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
        let buf = z_order(buf, 128);
        let f = Frame::new(buf.view_bits(), 128);
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

impl Index<Point> for LinearQuadTree {
    type Output = Leaf;

    /// Returns the leaf contaning the provided point
    fn index(&self, index: Point) -> &Self::Output {
        let point = index.into();

        let mut iter = self.0.iter().filter(|l| l.contains(&point));
        iter.next().expect(&format!(
            "Index out of range, image is 128x64 but point is {}x{}",
            index.x, index.y
        ))
    }
}

impl Index<Rectangle> for LinearQuadTree {
    type Output = [Leaf];

    /// Returns all the leaves between the top left and bottom right of the triangle (z-order
    /// curve)
    fn index(&self, index: Rectangle) -> &Self::Output {
        let Rectangle {
            top_left: base,
            size,
        } = index;
        let term = base + size;

        let start = self
            .0
            .iter()
            .take_while(|l| !l.contains(&base.into()))
            .count();
        let end = self
            .0
            .iter()
            .take_while(|l| !l.contains(&term.into()))
            .count();

        &self.0[start..end]
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
            let left = Frame::new(&f.buf[..4096], 64);
            let right = Frame::new(&f.buf[4096..], 64);
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
            let [tl, tr, bl, br] = f.split_four();
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

fn compare_bits(buf: &BitSliceU8) -> bool {
    let len = buf.len();

    if len >= 16 {
        let bytes = buf.as_raw_slice();
        return compare_bytes(bytes);
    }

    if len == 1 {
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

pub struct Frame<'a> {
    side: usize,
    buf: &'a BitSliceU8,
}

impl<'a> Frame<'a> {
    /// Assumes buffer is z-ordered
    pub fn new(buf: &'a BitSliceU8, side: usize) -> Self {
        Self { side, buf }
    }

    pub fn uniform(&self) -> bool {
        compare_bits(self.buf)
    }

    pub fn color(&self) -> bool {
        self.buf[0]
    }

    pub fn split_four(&self) -> [Frame; 4] {
        let len = self.buf.len() / 4;
        let side = self.side / 2;

        [
            Frame::new(&self.buf[..len], side),
            Frame::new(&self.buf[len..2 * len], side),
            Frame::new(&self.buf[2 * len..3 * len], side),
            Frame::new(&self.buf[3 * len..4 * len], side),
        ]
    }
}

// 000 001 100 101  0,0 1,0 2,0 3,0
// 010 011 110 111  0,1 1,1 2,1 3,1
fn z_order(source: &[u8], width: usize) -> Vec<u8> {
    let source = source.view_bits::<Msb0>();
    let mut out = vec![0u8; source.len() / 8];
    let out_bits = out.view_bits_mut::<Msb0>();

    for (i, mut cell) in out_bits.iter_mut().enumerate() {
        let x = odd_bits(i);
        let y = even_bits(i);
        let index = y * width + x;
        *cell = source[index];
    }

    out
}

fn even_bits(i: usize) -> usize {
    odd_bits(i >> 1)
}

fn odd_bits(input: usize) -> usize {
    let mut sum = 0;
    let mut offset = 0;

    while 1 << offset <= input {
        sum |= (input & 1 << offset) >> (offset / 2);
        offset += 2;
    }

    sum
}
