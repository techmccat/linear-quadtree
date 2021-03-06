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

/// A quadtree stored as a contiguous vector of leaves.
///
/// It doesn't support inserting yet, building is done by calling the `parse_12864` method.
#[derive(Default)]
pub struct LinearQuadTree(Vec<Leaf>);

impl LinearQuadTree {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates a new `LinearQuadTree`, allocating space for `cap` leaves.
    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    /// Populates the internal leaf store with a 128x64 bit framebuffer.
    pub fn parse_12864(&mut self, buf: &[u8; 1024]) {
        let mut z_curve: BitVec<Msb0, u8> = BitVec::with_capacity(buf.len() * 8);
        z_order_2to1(buf, &mut z_curve, 128);

        let mut parser = BulkParse::new(&mut self.0);

        let f = Frame::new(z_curve.as_ref(), 128);
        parser.parse_12864(f)
    }

    /// Stores the leaves as packed bytes into a writer.
    ///
    /// The packed format is as follows:  
    /// `1 010 01 00`  
    /// `^` 1  
    /// `  ^^^` 2  
    /// `      ^^^^^` 3
    ///
    /// 1: discriminant bit.
    /// If set bits 1 through 3 are treated as depth of the node.
    /// When not set depth is assumed to be 7  
    ///
    /// 2: depth.
    /// If the discriminant bit is not set, bit 1 is padding and position starts at bit 2.   
    ///
    /// 3: position.
    /// Groups of two bits that represent the position of the node in the quadtree.
    ///
    /// When the depth is less than or equal to 2, the leaf is represented as a single byte.
    /// It otherwise takes up two bytes.
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

/// A wrapper for a slice representing a square
struct Frame<'a> {
    side: usize,
    buf: &'a BitSliceU8,
}

impl<'a> Frame<'a> {
    /// Assumes buffer is z-ordered
    pub fn new(buf: &'a BitSliceU8, side: usize) -> Self {
        Self { side, buf }
    }

    /// Checks if all the bits in the buffer are set or unset
    pub fn uniform(&self) -> bool {
        compare_bits(self.buf)
    }

    /// Returns the first bit of the buffer
    pub fn color(&self) -> bool {
        self.buf[0]
    }

    /// Splits the frame into four frames.
    pub fn split_four(self) -> [Frame<'a>; 4] {
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

// calls z_order on the two top squares
fn z_order_2to1(source: &[u8], dest: &mut BitVec<Msb0, u8>, mut width: usize) {
    width /= 2;
    z_order(source, dest, width, 0, 0);
    z_order(source, dest, width, width, 0);
}

fn z_order(source: &[u8], dest: &mut BitVec<Msb0, u8>, mut width: usize, x: usize, y: usize) {
    if width == 1 {
        dest.push(source.view_bits::<Msb0>()[x + y * 128])
    } else {
        width /= 2;
        z_order(source, dest, width, x, y);
        z_order(source, dest, width, x + width, y);
        z_order(source, dest, width, x, y + width);
        z_order(source, dest, width, x + width, y + width);
    }
}
