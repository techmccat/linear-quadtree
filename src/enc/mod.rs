use crate::{Leaf, LeafData};

use bitvec::prelude::*;
use std::io::{Result as IoResult, Write};

pub mod video;

#[cfg(test)]
pub mod tests;

type BitSliceU8 = BitSlice<u8, Msb0>;
type BitVecU8 = BitVec<u8, Msb0>;
type Position = heapless::Vec<u8, 7>;

#[derive(Clone, Debug)]
pub enum Node {
    Empty,
    Leaf(LeafData),
    Branch(Box<[Node; 4]>),
}

impl Node {
    pub fn empty_branch() -> Self {
        // Box isn't Copy :)
        Self::Branch(Box::new([
            Node::Empty,
            Node::Empty,
            Node::Empty,
            Node::Empty,
        ]))
    }

    /// Get a reference to this node's child nodes, if there are any.
    pub fn children(&self) -> Option<&[Self; 4]> {
        if let Node::Branch(branches) = self {
            Some(branches)
        } else {
            None
        }
    }

    /// Get a mutable reference to this node's child nodes, if there are any.
    pub fn children_mut(&mut self) -> Option<&mut [Self; 4]> {
        if let Node::Branch(branches) = self {
            Some(branches)
        } else {
            None
        }
    }

    /// Returns the current value, leaving Node::Empty in its place.
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::Empty)
    }

    /// Parse a monochrome bitmap into Self.
    fn from_sector(sec: Frame) -> Self {
        if sec.uniform() {
            Self::Leaf(LeafData::Feature(sec.color()))
        } else if sec.side > 4 {
            let [tl, tr, bl, br] = sec.split_four();
            Self::Branch(Box::new([
                Self::from_sector(tl),
                Self::from_sector(tr),
                Self::from_sector(bl),
                Self::from_sector(br),
            ]))
        } else {
            // convert from z-curve and store after the leaf
            let mut bitmap = [0u8; 2];
            bitmap[0] = sec.buf[0..=1].load::<u8>() << 6
                | sec.buf[4..=5].load::<u8>() << 4
                | sec.buf[2..=3].load::<u8>() << 2
                | sec.buf[6..=7].load::<u8>();
            bitmap[1] = sec.buf[8..=9].load::<u8>() << 6
                | sec.buf[12..=13].load::<u8>() << 4
                | sec.buf[10..=11].load::<u8>() << 2
                | sec.buf[14..=15].load::<u8>();
            Self::Leaf(LeafData::Bitmap(bitmap))
        }
    }
}

#[derive(Clone, Debug)]
pub struct QuadTree {
    head: Node,
}

impl QuadTree {
    pub fn new() -> Self {
        Self { head: Node::Empty }
    }

    /// Builds a new tree from a 128x64 monochrome framebuffer.
    pub fn from_128x64(buf: &[u8; 1024]) -> Self {
        let mut z_curve: BitVecU8 = BitVec::with_capacity(buf.len() * 8);
        z_order_2to1(buf, &mut z_curve, 128);

        if Frame::new(&z_curve, 128).uniform() {
            return QuadTree {
                head: Node::Leaf(LeafData::Feature(buf[0] == 255))
            }
        }

        let mut out = Self {
            head: Node::empty_branch(),
        };
        let [left, right, _, _] = out.head.children_mut().unwrap();

        *left = Node::from_sector(Frame::new(&z_curve[..4096], 64));
        *right = Node::from_sector(Frame::new(&z_curve[4096..], 64));

        out
    }

    pub fn iter(&self) -> QuadTreeIterator {
        let mut stack = Vec::with_capacity(5 * 4);
        stack.push(&self.head);
        QuadTreeIterator {
            stack,
            position: Default::default(),
        }
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
    ///
    /// When depth is more than 5, the 4x4 bitmap is stored to save space.
    pub fn store_packed<W: Write>(&self, mut w: W) -> IoResult<usize> {
        let yes = self.iter().filter(|l| l.feat_or_data(true));
        let no = self.iter().filter(|l| l.feat_or_data(false));

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

#[derive(Clone, Debug)]
pub struct QuadTreeIterator<'a> {
    stack: Vec<&'a Node>,
    position: Position,
}

impl QuadTreeIterator<'_> {
    fn leaf_met(&mut self) {
        self.position.last_mut().map(|pos| *pos += 1);
        if self.position.last() == Some(&4) {
            self.position.pop();
            self.leaf_met();
        }
    }
}

impl Iterator for QuadTreeIterator<'_> {
    type Item = Leaf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            if let Some(nodes) = node.children() {
                // println!("Current depth: {}", self.position.len());
                self.position
                    .push(0)
                    .expect("Max depth exceeded during traversal");
                for node in nodes.into_iter().rev() {
                    self.stack.push(node);
                }
            } else {
                let pos = self.position.clone();
                self.leaf_met();
                if let Node::Leaf(feat) = node {
                    return Some(Leaf::new(
                        *feat,
                        heapless::Vec::from_slice(&pos.get(..5).unwrap_or(&pos)).unwrap(),
                    ));
                }
            }
        }
        None
    }
}

impl Leaf {
    fn write<W: std::io::Write>(&self, mut w: W) -> IoResult<usize> {
        let depth = self.pos.len();
        let mut data = [0u8; 2];
        let bits = data.view_bits_mut::<Msb0>();

        bits[..1].store(1u8);
        if let LeafData::Feature(_) = self.data {
            bits[1..4].store(depth);
        } else {
            bits[1..4].store(6u8);
        }

        for (i, p) in self.pos.iter().enumerate() {
            bits[4..][i * 2..=i * 2 + 1].store(*p)
        }

        let mut written = 0;
        if depth > 2 {
            w.write_all(&data)?;
            written += 2;
        } else {
            w.write_all(&data[..1])?;
            written += 1;
        }
        if let LeafData::Bitmap(b) = self.data {
            w.write_all(&b)?;
            written += 2;
        };

        Ok(written)
    }
}

fn compare_bytes(buf: &[u8]) -> bool {
    let mut prev = buf[0];

    if !(prev == u8::MAX || prev == 0) {
        return false;
    }

    for b in buf.iter().skip(1) {
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
        let (_, bytes, _) = buf.domain().region().unwrap();
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
fn z_order_2to1(source: &[u8], dest: &mut BitVecU8, mut width: usize) {
    width /= 2;
    z_order(source, dest, width, 0, 0);
    z_order(source, dest, width, width, 0);
}

fn z_order(source: &[u8], dest: &mut BitVecU8, mut width: usize, x: usize, y: usize) {
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
