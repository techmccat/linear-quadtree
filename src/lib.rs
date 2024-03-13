#![cfg_attr(not(feature = "std"), no_std)]
use core::convert::TryFrom;

#[cfg(feature = "dec")]
pub mod dec;
#[cfg(feature = "enc")]
pub mod enc;

#[cfg(test)]
pub mod tests;

mod utils {
    pub(crate) fn next_pos(pos: &mut crate::Position) -> Option<()> {
        Some(if let Some(p) = pos.last_mut() {
            if *p + 1 > 3 {
                pos.pop()?;
                next_pos(pos);
            } else {
                *p += 1
            }
        })
    }
}

type Position = heapless::Vec<u8, 7>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct FrameMeta {
    /// Determines if active feature is high or low
    pub active_feature: bool,
    /// Tells decoder to not overwwrite the previous framebuffer
    pub partial: bool,
    /// Tells decoder to flush the frambuffer after this frame
    pub display: bool,
}

impl FrameMeta {
    pub fn new(active_feature: bool, partial: bool, display: bool) -> Self {
        Self {
            active_feature,
            partial,
            display,
        }
    }
}

impl Into<u8> for FrameMeta {
    fn into(self) -> u8 {
        self.active_feature as u8 | (self.partial as u8) << 1 | (self.display as u8) << 2
    }
}

impl TryFrom<u8> for FrameMeta {
    type Error = dec::ParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 0b111 {
            let active_feature = value & 1 == 1;
            let partial = (value >> 1) & 1 == 1;
            let display = (value >> 2) & 1 == 1;
            Ok(Self {
                active_feature,
                partial,
                display,
            })
        } else {
            Err(Self::Error::InvalidHeader)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Leaf {
    pub data: LeafData,
    pos: Position,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LeafData {
    Feature(bool),
    Bitmap([u8; 2]),
}

impl Leaf {
    pub fn new(data: LeafData, pos: Position) -> Self {
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
                return false;
            }
        }

        true
    }

    #[cfg_attr(not(feature = "enc"), allow(dead_code))]
    fn feat_or_data(&self, feat: bool) -> bool {
        match self.data {
            LeafData::Bitmap(_) => true,
            LeafData::Feature(f) => f == feat,
        }
    }
}
