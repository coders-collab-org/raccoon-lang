//! This module defines the [Symbol] type, which is used to represent identifiers, keywords and other strings.

#![allow(non_upper_case_globals)]

use std::{fmt::Display, ops::Deref, sync::Mutex};

use fxhash::FxHashMap;
use raccoon_macros::symbols;
use typed_arena::Arena;

use crate::{Span, DUMMY_SP, GLOBAL_SESSION};

symbols! {
    {
        // Special tokens
        Empty: "",
        Wildcard: "_",

        // Keywords
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
        Mod: "mod",
        Use: "use",
        Fn: "fn",
        Struct: "struct",
        Enum: "enum",
        Pub: "pub",
        True: "true",
        False: "false",
        As: "as",
        Crate: "crate",
        SelfLower: "self",
        SelfUpper: "Self",
        Super: "super",
    }
}

/// A `Symbol` is an interned string that is used to represent identifiers and keywords.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Symbol(u32);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// An identifier (e.g. `foo`).
pub struct Ident {
    /// The name of the identifier.
    pub name: Symbol,

    /// span of the identifier
    pub span: Span,
}

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

    #[inline]
    pub fn is_keyword(self) -> bool {
        self <= kw::Pub && self >= kw::Let
    }

    pub fn is_path_segment_keyword(self) -> bool {
        self == kw::SelfLower || self == kw::Super || self == kw::SelfUpper || self == kw::Crate
    }
}

impl Ident {
    pub fn new(name: Symbol, span: Span) -> Self {
        Ident { name, span }
    }

    pub fn empty() -> Self {
        Ident {
            name: kw::Empty,
            span: DUMMY_SP,
        }
    }
}

impl Deref for Ident {
    type Target = Symbol;

    fn deref(&self) -> &Symbol {
        &self.name
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
    pub fn prefill(strings: &[&'static str]) -> Self {
        Interner(Mutex::new(InternerInner {
            arena: Arena::new(),
            strings: strings.to_owned(),
            symbols: strings.iter().copied().zip((0..).map(Symbol)).collect(),
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
