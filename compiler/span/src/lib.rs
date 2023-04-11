//! This crate defines the [Span] and [BytePos] types, which are used
//! to represent a contiguous region of source text.
//! The [Span] type is used to represent a contiguous region of source text.
//! The [BytePos] type is used to represent a single byte position in the source text.
//!
mod symbol;

use std::ops::{Add, Sub};

use lazy_static::lazy_static;
pub use symbol::*;

pub const DUMMY_SP: Span = Span::new(BytePos(0), BytePos(0));

/// A `Span` represents a contiguous region of source text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub lo: BytePos,
    pub hi: BytePos,
}

/// A `BytePos` is a byte offset into the source text.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BytePos(pub u32);

/// A `GlobalSession` contains global data that is shared across all compilation sessions.
pub struct GlobalSession {
    pub symbols: Interner,
}

impl Span {
    pub const fn new(lo: BytePos, hi: BytePos) -> Span {
        Span { lo, hi }
    }

    pub fn is_dummy(&self) -> bool {
        *self == DUMMY_SP
    }
}

impl GlobalSession {
    pub fn new() -> GlobalSession {
        GlobalSession {
            symbols: Interner::new(),
        }
    }
}
impl Default for GlobalSession {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    pub static ref GLOBAL_SESSION: GlobalSession = GlobalSession::new();
}

impl Add for BytePos {
    type Output = BytePos;

    fn add(self, other: BytePos) -> BytePos {
        BytePos(self.0 + other.0)
    }
}

impl Sub for BytePos {
    type Output = BytePos;

    fn sub(self, other: BytePos) -> BytePos {
        BytePos(self.0 - other.0)
    }
}

impl Add<u32> for BytePos {
    type Output = BytePos;

    fn add(self, other: u32) -> BytePos {
        BytePos(self.0 + other)
    }
}

impl From<u32> for BytePos {
    fn from(pos: u32) -> BytePos {
        BytePos(pos)
    }
}

impl From<usize> for BytePos {
    fn from(pos: usize) -> BytePos {
        BytePos(pos as u32)
    }
}
