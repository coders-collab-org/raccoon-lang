use raccoon_ast::{Block, Let, Stmt, StmtKind, TokenKind};
use raccoon_span::kw;

use crate::{ParseError, Parser};

impl<'a> Parser<'a> {
    pub fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.token.span;
        let stmt = self.parse_stmt_kind()?;
        Ok(Stmt {
            kind: stmt,
            span: start.to(self.prev_token.span),
        })
    }

    fn parse_stmt_kind(&mut self) -> Result<StmtKind, ParseError> {
        if self.check_keyword(kw::Let) {
            return Ok(StmtKind::Let(self.parse_let_stmt()?.into()));
        };

        if let Some(item) = self.parse_item()? {
            return Ok(StmtKind::Item(item.into()));
        }
        if self.eat(TokenKind::Semi) {
            return Ok(StmtKind::Empty);
        }

        Ok(StmtKind::Expr(self.parse_expr()?.into()))
    }

    pub fn parse_let_stmt(&mut self) -> Result<Let, ParseError> {
        let start = self.token.span;
        self.expect_keyword(kw::Let)?;
        let pat = self.parse_pat()?;

        let ty = if self.eat(TokenKind::Colon) {
            Some(self.parse_ty()?)
        } else {
            None
        };

        let init = if self.eat(TokenKind::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::Semi)?;

        Ok(Let {
            pat,
            ty,
            init,
            span: start.to(self.prev_token.span),
        })
    }

    pub fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.token.span;
        let stmts = self.parse_braced(|p| p.parse_stmt())?;
        Ok(Block {
            stmts,
            span: start.to(self.prev_token.span),
        })
    }
}
