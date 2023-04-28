mod expr;
mod item;
mod pat;
mod path;
mod stmt;
mod ty;

use raccoon_ast::{Delimiter, Token, TokenKind, DUMMY_TOKEN};
use raccoon_lexer::Lexer;
use raccoon_span::{Ident, Symbol};

use std::mem::replace;
use thin_vec::ThinVec;

// TODO: Replace this with diagnostic reporting
pub struct ParseError;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    token: Token,
    prev_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut parser = Self {
            lexer: Lexer::new(input),
            token: DUMMY_TOKEN,
            prev_token: DUMMY_TOKEN,
        };

        parser.advance();

        parser
    }

    pub fn advance(&mut self) -> Token {
        let token = self.lexer.advance();
        self.prev_token = replace(&mut self.token, token);
        token
    }

    pub fn advance_int(&mut self) -> Option<Symbol> {
        let token = self.lexer.scan_number(None, false);
        let number = token.lit().unwrap().symbol;

        if number.as_str().is_empty() {
            None
        } else {
            self.prev_token = replace(&mut self.token, token);
            Some(number)
        }
    }

    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if self.token.kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn eat_keyword(&mut self, keyword: Symbol) -> bool {
        self.eat(TokenKind::Ident(keyword))
    }

    pub fn eat_delim(&mut self, delim: Delimiter) -> bool {
        self.eat(TokenKind::OpenDelim(delim))
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        self.token.kind == kind
    }

    pub fn check_keyword(&self, keyword: Symbol) -> bool {
        self.check(TokenKind::Ident(keyword))
    }

    pub fn check_delim(&self, delim: Delimiter) -> bool {
        self.check(TokenKind::OpenDelim(delim))
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.token.kind == kind {
            self.advance();
            Ok(())
        } else {
            Err(ParseError)
        }
    }

    pub fn expect_keyword(&mut self, keyword: Symbol) -> Result<(), ParseError> {
        self.expect(TokenKind::Ident(keyword))
    }

    pub fn expect_semi(&mut self) -> Result<(), ParseError> {
        self.expect(TokenKind::Semi)
    }

    /// Parses a delimited list of items.
    #[inline]
    pub fn parse_terminals<T>(
        &mut self,
        start: TokenKind,
        sep: TokenKind,
        end: TokenKind,
        f: impl Fn(&mut Self) -> Result<T, ParseError>,
    ) -> Result<ThinVec<T>, ParseError> {
        self.expect(start)?;

        let mut items = ThinVec::new();

        loop {
            if self.eat(end) {
                break;
            }

            let item = f(self)?;

            items.push(item);

            if self.eat(end) {
                break;
            }

            self.expect(sep)?;
        }

        Ok(items)
    }

    /// Parses a comma-separated list of items.
    #[inline]
    pub fn parse_comma_separated<T>(
        &mut self,
        start: TokenKind,
        end: TokenKind,
        f: impl Fn(&mut Self) -> Result<T, ParseError>,
    ) -> Result<ThinVec<T>, ParseError> {
        self.parse_terminals(start, TokenKind::Comma, end, f)
    }

    /// Parses a comma-separated list of items, enclosed in parentheses.
    #[inline]
    pub fn parse_parenthesized<T>(
        &mut self,
        f: impl Fn(&mut Self) -> Result<T, ParseError>,
    ) -> Result<ThinVec<T>, ParseError> {
        self.parse_comma_separated(
            TokenKind::OpenDelim(Delimiter::Paren),
            TokenKind::CloseDelim(Delimiter::Paren),
            f,
        )
    }

    /// Parses a comma-separated list of items, enclosed in braces.
    #[inline]
    pub fn parse_braced<T>(
        &mut self,
        f: impl Fn(&mut Self) -> Result<T, ParseError>,
    ) -> Result<ThinVec<T>, ParseError> {
        self.parse_comma_separated(
            TokenKind::OpenDelim(Delimiter::Brace),
            TokenKind::CloseDelim(Delimiter::Brace),
            f,
        )
    }

    /// Parses a comma-separated list of items, enclosed in brackets.
    #[inline]
    pub fn parse_bracketed<T>(
        &mut self,
        f: impl Fn(&mut Self) -> Result<T, ParseError>,
    ) -> Result<ThinVec<T>, ParseError> {
        self.parse_comma_separated(
            TokenKind::OpenDelim(Delimiter::Bracket),
            TokenKind::CloseDelim(Delimiter::Bracket),
            f,
        )
    }

    pub fn parse_ident(&mut self, eat: bool) -> Result<Ident, ParseError> {
        match self.token.ident() {
            Some(ident) if !ident.is_keyword() => {
                if eat {
                    self.advance();
                }

                Ok(ident)
            }
            _ => Err(ParseError),
        }
    }
}
