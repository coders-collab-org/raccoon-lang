//! This module defines the [Symbol] type, which is used to represent identifiers, keywords and other strings.

#![allow(non_upper_case_globals)]

use std::{fmt::Display, sync::Mutex};

use fxhash::FxHashMap;
use typed_arena::Arena;

use crate::GLOBAL_SESSION;

macro_rules! keywords {
    ($($name:ident: $string:expr),* $(,)?) => {
        pub mod kw {
            lazy_static::lazy_static! {
                $(
                    pub static ref $name: $crate::Symbol = $crate::Symbol::intern($string);
                )*
            }
        }
    };
}

keywords! {
    Let: "let",
    Const: "const",
    If: "if",
    Else: "else",
    While: "while",
    For: "for",
    In: "in",
    Loop: "loop",
    Break: "break",
    Continue: "continue",
    Return: "return",
    Fn: "fn",
    Struct: "struct",
    Enum: "enum",

}

/// A `Symbol` is an interned string that is used to represent identifiers and keywords.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Symbol(u32);

impl Symbol {
    #[inline]
    pub fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn as_str(self) -> &'static str {
        GLOBAL_SESSION.symbols.get(self)
    }

    #[inline]
    pub fn intern(string: &str) -> Symbol {
        GLOBAL_SESSION.symbols.intern(string)
    }
}

/// An `Interner` is used to intern strings into [Symbol]s.
pub struct Interner(Mutex<InternerInner>);

pub struct InternerInner {
    arena: Arena<u8>,
    symbols: FxHashMap<&'static str, Symbol>,
    strings: Vec<&'static str>,
}

impl Interner {
    #[inline]
    pub fn new() -> Interner {
        Interner(Mutex::new(InternerInner {
            arena: Arena::new(),
            symbols: FxHashMap::default(),
            strings: Vec::new(),
        }))
    }

    pub fn intern(&self, string: &str) -> Symbol {
        let mut inner = self.0.lock().unwrap();
        if let Some(&symbol) = inner.symbols.get(string) {
            return symbol;
        }

        let symbol = Symbol(inner.strings.len() as u32);

        // SAFETY: The string is guaranteed to be valid for the lifetime of the arena.
        // The arena is never freed.
        let string: &'static str = unsafe { &*(inner.arena.alloc_str(string) as *const str) };

        inner.symbols.insert(string, symbol);
        inner.strings.push(string);

        symbol
    }

    pub fn get(&self, symbol: Symbol) -> &'static str {
        let inner = self.0.lock().unwrap();
        inner.strings[symbol.as_usize()]
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}
