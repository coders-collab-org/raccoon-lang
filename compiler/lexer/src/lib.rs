//! The lexer module provides a [Lexer] struct that can be used to lex tokens from a source code string.

use std::str::Chars;

use raccoon_ast::{BinOpToken, CondOpToken, Delimiter, Lit, Token, TokenKind, UnOpToken};
use raccoon_span::{kw, BytePos, Span, Symbol};
use TokenKind::*;

/// A `Cursor` is a wrapper around a [Chars] iterator that provides some additional methods
/// for lexing purposes and tracks the current position in the source code for error reporting.
#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    input: Chars<'a>,
    len: usize,
}

/// A `Lexer` is a wrapper around a [Cursor] that provides methods for lexing tokens
/// from the source code and tracks the current position in the source code for error reporting
pub struct Lexer<'a> {
    pub cursor: Cursor<'a>,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            input: input.chars(),
            len: input.len(),
        }
    }

    #[inline]
    pub fn peek(&self, ch: char) -> bool {
        self.peek_char() == Some(ch)
    }

    #[inline]
    pub fn peek_char(&self) -> Option<char> {
        self.input.clone().next()
    }

    #[inline]
    pub fn bump(&mut self) {
        self.next();
    }

    #[inline]
    pub fn bump_by(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }

    #[inline]
    pub fn pos(&self) -> BytePos {
        BytePos((self.len - self.input.as_str().len()) as u32)
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.input.as_str().is_empty()
    }
}

impl Iterator for Cursor<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next()
    }
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            cursor: Cursor::new(input),
        }
    }

    pub fn advance(&mut self) -> Token {
        let start = self.cursor.pos();
        let Some(ch) = self.cursor.next() else {
            return Token::new(TokenKind::Eof, Span::new(start, start));
        };

        // A macro to make it easier to write if-else chains that return early
        macro_rules! if_cond {
            ($char:expr,$if:expr, $else:expr) => {
                if self.cursor.peek($char) {
                    self.cursor.bump();
                    $if
                } else {
                    $else
                }
            };
        }

        let kind = match ch {
            '=' => if_cond!('=', CondOp(CondOpToken::Eq), Eq),
            '+' => if_cond!('=', BinOpEq(BinOpToken::Add), BinOp(BinOpToken::Add)),
            '-' => if_cond!(
                '=',
                BinOpEq(BinOpToken::Sub),
                if_cond!('>', RArrow, BinOp(BinOpToken::Sub))
            ),
            '*' => if_cond!('=', BinOpEq(BinOpToken::Mul), BinOp(BinOpToken::Mul)),
            '^' => if_cond!('=', BinOpEq(BinOpToken::BitXor), BinOp(BinOpToken::BitXor)),
            '%' => if_cond!('=', BinOpEq(BinOpToken::Rem), BinOp(BinOpToken::Rem)),
            '!' => if_cond!('=', CondOp(CondOpToken::Ne), UnOp(UnOpToken::Not)),
            ':' => if_cond!(':', DoubleColon, Colon),
            '.' => Dot,
            ',' => Comma,
            ';' => Semi,
            '(' => OpenDelim(Delimiter::Paren),
            ')' => CloseDelim(Delimiter::Paren),
            '{' => OpenDelim(Delimiter::Brace),
            '}' => CloseDelim(Delimiter::Brace),
            '[' => OpenDelim(Delimiter::Bracket),
            ']' => CloseDelim(Delimiter::Bracket),

            '/' => if_cond!(
                '/',
                return self.skip_inline_comment(),
                if_cond!(
                    '*',
                    return self.skip_block_comment(),
                    if_cond!('=', BinOpEq(BinOpToken::Div), BinOp(BinOpToken::Div))
                )
            ),
            '&' => if_cond!(
                '&',
                if_cond!('=', BinOpEq(BinOpToken::And), BinOp(BinOpToken::And)),
                if_cond!('=', BinOpEq(BinOpToken::BitAnd), BinOp(BinOpToken::BitAnd))
            ),
            '|' => if_cond!(
                '|',
                if_cond!('=', BinOpEq(BinOpToken::Or), BinOp(BinOpToken::Or)),
                if_cond!('=', BinOpEq(BinOpToken::BitOr), BinOp(BinOpToken::BitOr))
            ),
            '>' => if_cond!(
                '=',
                CondOp(CondOpToken::Ge),
                if_cond!(
                    '>',
                    if_cond!('=', BinOpEq(BinOpToken::Shr), BinOp(BinOpToken::Shr)),
                    CondOp(CondOpToken::Gt)
                )
            ),
            '<' => if_cond!(
                '=',
                CondOp(CondOpToken::Le),
                if_cond!(
                    '<',
                    if_cond!('=', BinOpEq(BinOpToken::Shl), BinOp(BinOpToken::Shl)),
                    CondOp(CondOpToken::Lt)
                )
            ),
            '"' => return self.scan_string(true),
            '0'..='9' => return self.scan_number(Some(ch), true),
            'a'..='z' | 'A'..='Z' | '_' => return self.scan_ident(Some(ch)),
            ch if ch.is_whitespace() => return self.skip_whitespace(),

            // TODO: handle invalid character
            _ => panic!("unexpected character: {}", ch),
        };

        Token::new(kind, Span::new(start, self.cursor.pos()))
    }

    #[inline]
    pub fn skip_whitespace(&mut self) -> Token {
        while let Some(ch) = self.cursor.peek_char() {
            if !ch.is_whitespace() {
                break;
            }
            self.cursor.bump();
        }

        self.advance()
    }

    pub fn scan_ident(&mut self, first: Option<char>) -> Token {
        let start = self.cursor.pos() - BytePos(if first.is_some() { 1 } else { 0 });
        let mut buf = if let Some(ch) = first {
            ch.to_string()
        } else {
            String::new()
        };

        while let Some(ch) = self.cursor.peek_char() {
            match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => buf.push(ch),
                _ => break,
            }
            self.cursor.bump();
        }

        let sym = Symbol::intern(&buf);

        if matches!(sym, kw::True | kw::False) {
            return Token::new(
                TokenKind::Lit(Lit::new_bool(sym)),
                Span::new(start, self.cursor.pos()),
            );
        }

        Token::new(TokenKind::Ident(sym), Span::new(start, self.cursor.pos()))
    }

    pub fn scan_number(&mut self, first: Option<char>, scan_float: bool) -> Token {
        let start = self.cursor.pos() - BytePos(if first.is_some() { 1 } else { 0 });
        let mut buf = if let Some(ch) = first {
            ch.to_string()
        } else {
            String::new()
        };
        let mut scanned_float = false;

        while let Some(ch) = self.cursor.peek_char() {
            match ch {
                '0'..='9' => buf.push(ch),
                '.' if scan_float && !scanned_float => {
                    buf.push(ch);
                    scanned_float = true;
                }
                _ => break,
            }
            self.cursor.bump();
        }

        let lit = if scanned_float {
            Lit::new_float(Symbol::intern(&buf))
        } else {
            Lit::new_int(Symbol::intern(&buf))
        };

        Token::new(TokenKind::Lit(lit), Span::new(start, self.cursor.pos()))
    }

    #[inline]
    pub fn scan_string(&mut self, inside_next: bool) -> Token {
        let start = self.cursor.pos() - BytePos(if inside_next { 1 } else { 0 });
        let mut buf = String::new();
        let mut terminated = false;
        for ch in &mut self.cursor {
            if ch == '"' {
                terminated = true;
                break;
            }
            buf.push(ch);
        }
        if !terminated {
            // TODO: update it to error handling
            panic!("Unterminated string literal");
        }

        let sym = Lit::new_str(Symbol::intern(&buf));

        Token::new(TokenKind::Lit(sym), Span::new(start, self.cursor.pos()))
    }

    #[inline]
    pub fn skip_inline_comment(&mut self) -> Token {
        for ch in &mut self.cursor {
            if ch == '\n' {
                break;
            }
        }
        self.advance()
    }

    #[inline]
    pub fn skip_block_comment(&mut self) -> Token {
        let mut terminated = false;
        while let Some(ch) = self.cursor.next() {
            if ch == '*' && self.cursor.peek('/') {
                terminated = true;
                self.cursor.bump();
                break;
            }
        }
        if !terminated {
            // TODO: update it to error handling
            panic!("Unterminated block comment");
        }

        self.advance()
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.cursor.is_eof()
    }

    #[inline]
    pub fn pos(&self) -> BytePos {
        self.cursor.pos()
    }
}
