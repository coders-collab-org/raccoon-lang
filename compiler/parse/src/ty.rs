use crate::{path::PathStyle, ParseError, Parser};
use raccoon_ast::{Delimiter, TokenKind, Ty, TyKind};
use raccoon_span::kw;

impl<'a> Parser<'a> {
    pub fn parse_ty(&mut self) -> Result<Ty, ParseError> {
        let start = self.token.span;
        let kind = self.parse_ty_kind()?;
        let ty = Ty {
            kind,
            span: start.to(self.prev_token.span),
        };

        if self.eat_delim(Delimiter::Bracket) {
            self.expect(TokenKind::CloseDelim(Delimiter::Bracket))?;
            Ok(Ty {
                kind: TyKind::Array(ty.into()),
                span: start.to(self.prev_token.span),
            })
        } else {
            Ok(ty)
        }
    }

    fn parse_ty_kind(&mut self) -> Result<TyKind, ParseError> {
        let kind = if self.check_delim(Delimiter::Paren) {
            let mut types = self.parse_parenthesized(|p| p.parse_ty())?;

            if types.len() == 1 {
                TyKind::Paren(types.remove(0).into())
            } else if types.is_empty() {
                TyKind::Unit
            } else {
                TyKind::Tuple(types)
            }
        } else if self.check_keyword(kw::Wildcard) {
            TyKind::Infer
        } else {
            TyKind::Path(self.parse_path(PathStyle::Type)?)
        };

        Ok(kind)
    }
}
