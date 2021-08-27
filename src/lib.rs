#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "dec")]
pub mod dec;
#[cfg(feature = "enc")]
pub mod enc;

#[cfg(test)]
pub mod tests;

#[derive(Debug, PartialEq)]
pub struct Leaf {
    pub feature: bool,
    pos: heapless::Vec<u8, 7>,
}

impl Leaf {
    pub fn new(feature: bool) -> Self {
        Self {
            feature,
            pos: heapless::Vec::new()
        }
    }

    pub fn depth(&self) -> usize {
        self.pos.len()
    }
}
