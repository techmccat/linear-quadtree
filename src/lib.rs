#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "dec")]
pub mod dec;
#[cfg(feature = "enc")]
pub mod enc;

#[cfg(test)]
pub mod tests;

#[derive(Debug, PartialEq, Clone)]
pub struct Leaf {
    pub feature: bool,
    pos: heapless::Vec<u8, 7>,
}

impl Leaf {
    pub fn new(feature: bool, pos: heapless::Vec<u8, 7>) -> Self {
        Self {
            feature,
            pos
        }
    }

    pub fn depth(&self) -> usize {
        self.pos.len()
    }
}
