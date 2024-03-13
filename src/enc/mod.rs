use crate::{FrameMeta, Leaf, LeafData, Position};

use bitvec::prelude::*;
use std::{
    io::{Result as IoResult, Write},
    iter::repeat,
};

pub mod video;

#[cfg(test)]
pub mod tests;

type BitSliceU8 = BitSlice<u8, Msb0>;
type BitVecU8 = BitVec<u8, Msb0>;

#[derive(Clone, Debug, PartialEq)]
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
    fn from_sector(sec: Frame, use_bitmap: bool) -> Self {
        if sec.uniform() {
            Self::Leaf(LeafData::Feature(sec.color()))
        } else if sec.side > 4 || !use_bitmap {
            let [tl, tr, bl, br] = sec.split_four();
            Self::Branch(Box::new([
                Self::from_sector(tl, use_bitmap),
                Self::from_sector(tr, use_bitmap),
                Self::from_sector(bl, use_bitmap),
                Self::from_sector(br, use_bitmap),
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

    pub fn diff(&self, other: &Self) -> Self {
        match (self.children(), other.children()) {
            (Some(a), Some(b)) => {
                let nodes = [
                    a[0].diff(&b[0]),
                    a[1].diff(&b[1]),
                    a[2].diff(&b[2]),
                    a[3].diff(&b[3]),
                ];
                if nodes.iter().eq(repeat(&Self::Empty).take(4)) {
                    Self::Empty
                } else {
                    Self::Branch(Box::new(nodes))
                }
            }
            (None, None) => {
                if self == other {
                    Self::Empty
                } else {
                    self.clone()
                }
            }
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct QuadTree {
    pub head: Node,
}

impl QuadTree {
    pub fn new() -> Self {
        Self { head: Node::Empty }
    }

    /// Builds a new tree from a 128x64 monochrome framebuffer.
    pub fn from_128x64(buf: &[u8; 1024], use_bitmap: bool) -> Self {
        let mut z_curve: BitVecU8 = BitVec::with_capacity(buf.len() * 8);
        z_order_2to1(buf, &mut z_curve, 128);

        if Frame::new(&z_curve, 128).uniform() {
            return QuadTree {
                head: Node::Leaf(LeafData::Feature(buf[0] == 255)),
            };
        }

        let mut out = Self {
            head: Node::empty_branch(),
        };
        let [left, right, _, _] = out.head.children_mut().unwrap();

        *left = Node::from_sector(Frame::new(&z_curve[..4096], 64), use_bitmap);
        *right = Node::from_sector(Frame::new(&z_curve[4096..], 64), use_bitmap);

        out
    }

    pub fn leaves(&self) -> QuadTreeIterator {
        QuadTreeIterator {
            inner: self.nodes(),
            position: Default::default(),
        }
    }

    pub fn nodes(&self) -> QuadTreeTraverser {
        QuadTreeTraverser {
            stack: vec![&self.head],
        }
    }

    pub fn diff(&self, other: &Self) -> Self {
        Self {
            head: self.head.diff(&other.head),
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
        let yes = self.leaves().filter(|l| l.feat_or_data(true));
        let no = self.leaves().filter(|l| l.feat_or_data(false));

        let mut count = 1;

        if yes.clone().count() < no.clone().count() {
            w.write_all(&[FrameMeta::new(true, false, true).into()])?;
            for leaf in yes {
                count += leaf.write(&mut w)?;
            }
        } else {
            w.write_all(&[FrameMeta::new(false, false, true).into()])?;
            for leaf in no {
                count += leaf.write(&mut w)?;
            }
        }

        Ok(count)
    }
    pub fn collect_compact(&self) -> Result<BitVecU8, &str> {
        let mut out = BitVecU8::new();
        for n in self.nodes() {
            match n {
                Node::Branch(_) => out.extend([false, true]),
                Node::Empty => out.extend([false, false]),
                Node::Leaf(LeafData::Feature(val)) => out.extend([true, *val]),
                Node::Leaf(LeafData::Bitmap(_)) => return Err("Bitmap leaves are not supported by the compact representation"),
            }
        }
        Ok(out)
    }

    /// Same as `store_packed`, but keeps both features and tells the decoder not to clear the
    /// framebuffer with the inactive feature before drawing, used in p-frames.
    pub fn store_as_diff<W: Write>(&self, mut w: W) -> IoResult<(usize, usize)> {
        let mut count_yes = 1;
        let mut count_no = 1;

        w.write_all(&[FrameMeta::new(true, true, false).into()])?;
        for leaf in self.leaves().filter(|l| l.feat_or_data(true)) {
            count_yes += leaf.write(&mut w)?;
        }
        w.write_all(&[FrameMeta::new(false, true, true).into()])?;
        for leaf in self.leaves().filter(|l| l.feat_or_data(false)) {
            count_no += leaf.write(&mut w)?;
        }

        Ok((count_yes, count_no))
    }
}

/// Depth-first traversal returning every node
#[derive(Clone, Debug)]
pub struct QuadTreeTraverser<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> Iterator for QuadTreeTraverser<'a> {
    type Item = &'a Node;
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|node| {
            if let Some(nodes) = node.children() {
                self.stack.extend(nodes.into_iter().rev());
            }
            node
        })
    }
}

/// Depth-first traversal returning leaves
#[derive(Clone, Debug)]
pub struct QuadTreeIterator<'a> {
    inner: QuadTreeTraverser<'a>,
    position: Position,
}

pub(crate) fn next_pos(pos: &mut Position) -> Option<()> {
    Some(if let Some(p) = pos.last_mut() {
        if *p + 1 > 3 {
            pos.pop()?;
            next_pos(pos);
        } else {
            *p += 1
        }
    })
}

impl Iterator for QuadTreeIterator<'_> {
    type Item = Leaf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.inner.next() {
            match node {
                Node::Leaf(data) => {
                    let pos = self.position.clone();
                    next_pos(&mut self.position);
                    return Some(Leaf::new(
                        *data,
                        pos.clone(),
                    ));
                }
                Node::Empty => next_pos(&mut self.position).unwrap(),
                Node::Branch(_) => self.position.push(0).expect("Max depth exceeded"),
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

        if depth > 5 {
            panic!("Depth not supposed to exceed 5")
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
