use raccoon_ast::{Delimiter, EnumPat, Pat, PatKind, StructPat, StructPatField, TokenKind};

use crate::{path::PathStyle, ParseError, Parser};

impl<'a> Parser<'a> {
    pub fn parse_pat(&mut self) -> Result<Pat, ParseError> {
        let start = self.token.span;
        let pat = self.parse_pat_kind()?;
        Ok(Pat {
            kind: pat,
            span: start.to(self.prev_token.span),
        })
    }

    fn parse_pat_kind(&mut self) -> Result<PatKind, ParseError> {
        let pat = match self.token.kind {
            TokenKind::Ident(_) => {
                let path = self.parse_path(PathStyle::Expr)?;

                match self.token.kind {
                    TokenKind::OpenDelim(Delimiter::Brace) => {
                        let fields = self.parse_braced(|p| p.parse_pat_field())?;
                        PatKind::Struct(StructPat { path, fields }.into())
                    }
                    TokenKind::OpenDelim(Delimiter::Paren) => {
                        let fields = self.parse_parenthesized(|p| p.parse_pat())?;
                        PatKind::Enum(EnumPat { path, fields }.into())
                    }

                    _ => PatKind::Path(path),
                }
            }
            TokenKind::OpenDelim(Delimiter::Paren) => {
                PatKind::Tuple(self.parse_parenthesized(|p| p.parse_pat())?)
            }

            TokenKind::OpenDelim(Delimiter::Bracket) => {
                PatKind::Slice(self.parse_bracketed(|p| p.parse_pat())?)
            }

            _ => return Err(ParseError),
        };

        Ok(pat)
    }

    fn parse_pat_field(&mut self) -> Result<StructPatField, ParseError> {
        let start = self.token.span;
        let ident = self.parse_ident(true)?;
        let pat = if self.eat(TokenKind::Colon) {
            Some(self.parse_pat()?)
        } else {
            None
        };

        Ok(StructPatField {
            ident,
            pat,
            span: start.to(self.prev_token.span),
        })
    }
}
