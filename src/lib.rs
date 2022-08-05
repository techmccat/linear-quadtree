#![cfg_attr(not(feature = "std"), no_std)]

use heapless::Vec;

#[cfg(feature = "dec")]
pub mod dec;
#[cfg(feature = "enc")]
pub mod enc;

#[cfg(test)]
pub mod tests;

#[derive(Debug, PartialEq, Clone)]
pub struct Leaf {
    pub data: LeafData,
    pos: Vec<u8, 5>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LeafData {
    Feature(bool),
    Bitmap([u8; 2])
}

impl Leaf {
    pub fn new(data: LeafData, pos: Vec<u8, 5>) -> Self {
        Self { data, pos }
    }

    pub fn depth(&self) -> usize {
        self.pos.len()
    }

    pub fn contains(&self, other: &Self) -> bool {
        if self.depth() > other.depth() {
            return false;
        };
        let iter = other.pos.iter().take(self.depth()).zip(self.pos.iter());
        for (other, this) in iter {
            if other != this {
                return false
            }
        }

        true
    }

    fn feat_or_data(&self, feat: bool) -> bool {
        match self.data {
            LeafData::Bitmap(_) => true,
            LeafData::Feature(f) => f == feat,
        }
    }
}
