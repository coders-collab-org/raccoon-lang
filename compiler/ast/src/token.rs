//! A token is a lexical unit of the source code.

use raccoon_span::{Span, Symbol, DUMMY_SP};

pub const DUMMY_TOKEN: Token = Token::new(TokenKind::Dummy, DUMMY_SP);

/// A token is a lexical unit of the source code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Token {
        Token { kind, span }
    }

    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }
}

/// A token kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// A conditional operator.
    CondOp(CondOpToken),

    /// A binary operator.
    BinOp(BinOpToken),

    /// A binary operator with an assignment.
    BinOpEq(BinOpToken),

    /// A unary operator.
    UnOp(UnOpToken),

    /// `=`.
    Eq,

    /// `.`.
    Dot,

    /// `,`.
    Comma,

    /// `;`.
    Semi,

    /// `:`.
    Colon,

    /// `::`.
    ColonColon,

    /// `"`.
    Quote,

    /// A literal.
    Lit(Lit),

    /// An opening delimiter e.g. `(`.
    OpenDelim(Delimiter),

    /// A closing delimiter e.g. `)`.
    CloseDelim(Delimiter),

    /// An identifier.
    Ident(Symbol),

    /// An end-of-file token.
    Eof,

    /// A dummy token.
    Dummy,
}

/// A delimiter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delimiter {
    /// `(...)`.
    Parenthesis,

    /// `[...]`.
    Bracket,

    /// `{...}`.
    Brace,
}

/// A literal value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lit {
    pub kind: LitKind,
    pub symbol: Symbol,
}
impl Lit {
    pub const fn new(kind: LitKind, symbol: Symbol) -> Lit {
        Lit { kind, symbol }
    }

    pub fn new_str(symbol: Symbol) -> Lit {
        Lit::new(LitKind::Str, symbol)
    }

    pub fn new_int(symbol: Symbol) -> Lit {
        Lit::new(LitKind::Int, symbol)
    }

    pub fn new_float(symbol: Symbol) -> Lit {
        Lit::new(LitKind::Float, symbol)
    }
}

/// A literal kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LitKind {
    /// An integer literal e.g. `1`.
    Int,

    /// A floating point literal e.g. `1.0`.
    Float,

    /// A string literal e.g. `"hello"`.
    Str,

    /// A boolean literal e.g. `true`.
    Bool,
}

/// A conditional operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CondOpToken {
    /// `==`
    Eq,

    /// `!=`
    Ne,

    /// `<`
    Lt,

    /// `<=`
    Le,

    /// `>`
    Gt,

    /// `>=`
    Ge,

    /// `&&`
    And,

    /// `||`
    Or,
}

/// A binary operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOpToken {
    /// `+`
    Add,

    /// `-`
    Sub,

    /// `*`
    Mul,

    /// `/`
    Div,

    /// `%`
    Rem,

    /// `&`
    And,

    /// `|`
    Or,

    /// `^`
    Xor,

    /// `<<`
    Shl,

    /// `>>`
    Shr,
}

/// A unary operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOpToken {
    /// `!`
    Not,

    /// `~`
    NotBitwise,

    /// `-`
    Neg,
}
