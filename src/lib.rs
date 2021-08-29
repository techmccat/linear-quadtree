#![cfg_attr(not(feature = "std"), no_std)]

use embedded_graphics::prelude::Point;
use heapless::Vec;

#[cfg(feature = "dec")]
pub mod dec;
#[cfg(feature = "enc")]
pub mod enc;

#[cfg(test)]
pub mod tests;

#[derive(Debug, PartialEq, Clone)]
pub struct Leaf {
    pub feature: bool,
    pos: Vec<u8, 7>,
}

impl Leaf {
    pub fn new(feature: bool, pos: Vec<u8, 7>) -> Self {
        Self { feature, pos }
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
}

impl From<Point> for Leaf {
    fn from(p: Point) -> Self {
        let mut pos: Vec<u8, 7> = Vec::new();
        let mut size = 64 ;
        let Point { mut x, mut y } = p;

        while pos.len() < 7 {
            let p = if x < size {
                if y < size {
                    0
                } else {
                    y -= size;
                    1
                }
            } else {
                x -= size;
                if y < size {
                    2
                } else {
                    y -= size;
                    3
                }
            };
            // safe to ignore because depth is checked at the start of the loop
            let _ = pos.push(p);
            size /= 2;
        }

        Self {
            pos,
            feature: false
        }
    }
}
