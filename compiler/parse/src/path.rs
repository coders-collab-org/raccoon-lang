use raccoon_ast::{BinOpToken, Delimiter, Path, PathSegment, TokenKind};
use thin_vec::thin_vec;

use crate::{ParseError, Parser};

/// The style of path being parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathStyle {
    /// A path in expression position, e.g. `foo::bar::<Baz>`
    Expr,

    /// A path in type position, e.g. `foo::bar<Baz>`
    Type,

    /// A path in module declaration position, e.g. `foo::bar::Baz`
    Mod,
}

impl<'a> Parser<'a> {
    pub fn parse_path(&mut self, style: PathStyle) -> Result<Path, ParseError> {
        let mut segments = thin_vec![PathSegment {
            ident: self.parse_ident(true)?,
            span: self.prev_token.span,
        }];

        while self.check(TokenKind::DoubleColon) {
            if style == PathStyle::Mod
                && matches!(
                    self.token.kind,
                    TokenKind::OpenDelim(Delimiter::Brace) | TokenKind::BinOp(BinOpToken::Mul)
                )
            {
                break;
            }

            self.advance();

            let ident = match self.token.ident() {
                Some(ident) if ident.is_path_segment_keyword() => ident,
                _ => self.parse_ident(true)?,
            };

            segments.push(PathSegment {
                ident,
                span: self.prev_token.span,
            });
        }

        Ok(Path { segments })
    }
}
