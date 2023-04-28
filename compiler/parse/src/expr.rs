use crate::{path::PathStyle, ParseError, Parser};
use raccoon_ast::{
    Assign, BinOp, BinOpKind, BinOpToken, Binary, Block, Call, CondOpToken, Delimiter, Expr,
    ExprKind, Field, FieldKind, For, If, Index, StructExprField, Token, TokenKind, UnOpToken,
    Unary, UnaryOp, UnaryOpKind, While,
};
use raccoon_span::{kw, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Precedence {
    Any,
    Assign,
    Or,
    And,
    BitOr,
    BitXor,
    BitAnd,
    Compare,
    Shift,
    Arithmetic,
    Term,
}

impl Precedence {
    pub fn from_token(token: &Token) -> Option<Self> {
        use TokenKind::{BinOp, BinOpEq, CondOp};
        Some(match token.kind {
            BinOpEq(_) | TokenKind::Eq => Precedence::Assign,
            BinOp(BinOpToken::Or) => Precedence::Or,
            BinOp(BinOpToken::And) => Precedence::And,
            BinOp(BinOpToken::BitOr) => Precedence::BitOr,
            BinOp(BinOpToken::BitAnd) => Precedence::BitAnd,
            BinOp(BinOpToken::BitXor) => Precedence::BitXor,
            CondOp(CondOpToken::Eq)
            | CondOp(CondOpToken::Ne)
            | CondOp(CondOpToken::Lt)
            | CondOp(CondOpToken::Le)
            | CondOp(CondOpToken::Gt)
            | CondOp(CondOpToken::Ge) => Precedence::Compare,
            BinOp(BinOpToken::Shl) | BinOp(BinOpToken::Shr) => Precedence::Shift,
            BinOp(BinOpToken::Add) | BinOp(BinOpToken::Sub) => Precedence::Arithmetic,
            BinOp(BinOpToken::Mul) | BinOp(BinOpToken::Div) | BinOp(BinOpToken::Rem) => {
                Precedence::Term
            }

            _ => return None,
        })
    }
}

impl Parser<'_> {
    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_unary_expr()?;
        self.parse_bin_rhs(Precedence::Any, lhs)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.token.span;

        if matches!(self.token.kind, TokenKind::UnOp(_)) {
            let op = self.parse_unary_op()?;
            return Ok(Expr {
                kind: ExprKind::Unary(
                    Unary {
                        op,
                        expr: self.parse_unary_expr()?,
                    }
                    .into(),
                ),
                span: start.to(self.prev_token.span),
            });
        };

        let mut expr = self.parse_primary_expr()?;

        loop {
            if self.check_delim(Delimiter::Paren) {
                let args = self.parse_parenthesized(|p| p.parse_expr())?;
                expr = self.mk_expr(
                    ExprKind::Call(Call { callee: expr, args }.into()),
                    start.to(self.prev_token.span),
                );
            } else if self.eat_delim(Delimiter::Brace) {
                let index = self.parse_expr()?;
                expr = self.mk_expr(
                    ExprKind::Index(Index { base: expr, index }.into()),
                    start.to(self.prev_token.span),
                );
                self.eat(TokenKind::CloseDelim(Delimiter::Brace));
            } else if self.eat(TokenKind::Dot) {
                let number = self.advance_int();

                let kind = match number {
                    Some(number) => FieldKind::Unnamed(number),
                    None => FieldKind::Named(self.parse_ident(true)?),
                };
                expr = self.mk_expr(
                    ExprKind::Field(Field { base: expr, kind }.into()),
                    start.to(self.prev_token.span),
                );
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_bin_rhs(&mut self, min_prec: Precedence, mut lhs: Expr) -> Result<Expr, ParseError> {
        let start = self.token.span;

        while let Some(prec) = Precedence::from_token(&self.token) {
            if prec < min_prec {
                break;
            }

            let token = self.token;
            let op = self.parse_bin_op().unwrap();
            let mut rhs = self.parse_unary_expr()?;

            while let Some(next_prec) = Precedence::from_token(&self.token) {
                if next_prec <= prec && prec != Precedence::Assign {
                    break;
                }

                rhs = self.parse_bin_rhs(next_prec, rhs)?;
            }

            match token.kind {
                TokenKind::Eq => {
                    lhs = self.mk_expr(
                        ExprKind::Assign(Assign { lhs, rhs }.into()),
                        start.to(self.prev_token.span),
                    );
                    continue;
                }

                TokenKind::BinOpEq(..) => {
                    lhs = self.mk_expr(
                        ExprKind::AssignOp(Binary { lhs, op, rhs }.into()),
                        start.to(self.prev_token.span),
                    );
                    continue;
                }

                _ => (),
            }

            lhs = self.mk_expr(
                ExprKind::Binary(Binary { lhs, op, rhs }.into()),
                start.to(self.prev_token.span),
            );
        }

        Ok(lhs)
    }

    fn parse_unary_op(&mut self) -> Result<UnaryOp, ParseError> {
        let start = self.token.span;
        use TokenKind::UnOp;
        let op = match self.token.kind {
            UnOp(UnOpToken::Not) => UnaryOpKind::Not,
            UnOp(UnOpToken::Neg) => UnaryOpKind::Neg,
            UnOp(UnOpToken::BitNot) => UnaryOpKind::BitNot,
            _ => return Err(ParseError),
        };

        self.advance();

        Ok(UnaryOp {
            kind: op,
            span: start.to(self.prev_token.span),
        })
    }

    fn parse_bin_op(&mut self) -> Option<BinOp> {
        use TokenKind::{BinOp, BinOpEq, CondOp};
        let op = match self.token.kind {
            BinOp(BinOpToken::BitOr) | BinOpEq(BinOpToken::BitOr) => BinOpKind::BitOr,
            BinOp(BinOpToken::Shl) | BinOpEq(BinOpToken::Shl) => BinOpKind::Shl,
            BinOp(BinOpToken::Shr) | BinOpEq(BinOpToken::Shr) => BinOpKind::Shr,
            BinOp(BinOpToken::Add) | BinOpEq(BinOpToken::Add) => BinOpKind::Add,
            BinOp(BinOpToken::Sub) | BinOpEq(BinOpToken::Sub) => BinOpKind::Sub,
            BinOp(BinOpToken::Mul) | BinOpEq(BinOpToken::Mul) => BinOpKind::Mul,
            BinOp(BinOpToken::Div) | BinOpEq(BinOpToken::Div) => BinOpKind::Div,
            BinOp(BinOpToken::Rem) | BinOpEq(BinOpToken::Rem) => BinOpKind::Rem,
            BinOp(BinOpToken::BitAnd) | BinOpEq(BinOpToken::BitAnd) => BinOpKind::BitAnd,
            BinOp(BinOpToken::BitXor) | BinOpEq(BinOpToken::BitXor) => BinOpKind::BitXor,

            // we just add `==` to skip None
            CondOp(CondOpToken::Eq) | TokenKind::Eq => BinOpKind::Eq,
            CondOp(CondOpToken::Ne) => BinOpKind::Ne,
            CondOp(CondOpToken::Lt) => BinOpKind::Lt,
            CondOp(CondOpToken::Le) => BinOpKind::Le,
            CondOp(CondOpToken::Gt) => BinOpKind::Gt,
            CondOp(CondOpToken::Ge) => BinOpKind::Ge,

            _ => return None,
        };

        Some(raccoon_ast::BinOp {
            kind: op,
            span: self.token.span,
        })
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.token.span;

        let expr = match self.token.kind {
            TokenKind::Lit(lit) => {
                self.advance();
                ExprKind::Lit(lit)
            }
            TokenKind::Ident(ident) => match ident {
                kw::If => self.parse_if_expr()?,
                kw::While => self.parse_while_expr()?,
                kw::For => self.parse_for_expr()?,
                kw::Loop => self.parse_loop_expr()?,
                kw::Return => self.parse_return_expr()?,
                kw::Break => self.parse_break_expr()?,
                kw::Continue => {
                    self.advance();
                    ExprKind::Continue
                }
                _ => ExprKind::Path(self.parse_path(PathStyle::Expr)?),
            },
            TokenKind::OpenDelim(Delimiter::Paren) => {
                let mut exprs = self.parse_parenthesized(|p| p.parse_expr())?;

                if exprs.len() == 1 {
                    ExprKind::Paren(exprs.remove(0).into())
                } else {
                    ExprKind::Tuple(exprs)
                }
            }
            TokenKind::OpenDelim(Delimiter::Brace) => {
                let stmts = self.parse_braced(|p| p.parse_stmt())?;
                ExprKind::Block(
                    Block {
                        stmts,
                        span: start.to(self.prev_token.span),
                    }
                    .into(),
                )
            }

            TokenKind::OpenDelim(Delimiter::Bracket) => {
                ExprKind::Array(self.parse_bracketed(|p| p.parse_expr())?)
            }

            TokenKind::Hash => {
                let fields = self.parse_braced(|p| p.parse_struct_expr_field())?;

                ExprKind::Struct(fields)
            }
            _ => return Err(ParseError),
        };

        Ok(self.mk_expr(expr, start))
    }

    fn parse_struct_expr_field(&mut self) -> Result<StructExprField, ParseError> {
        let start = self.token.span;
        let name = self.parse_ident(true)?;

        let value = if self.eat(TokenKind::Colon) {
            self.parse_expr()?
        } else {
            self.mk_expr(ExprKind::Path(name.into()), start)
        };

        Ok(StructExprField {
            name,
            value,
            span: start.to(self.prev_token.span),
        })
    }

    fn parse_if_expr(&mut self) -> Result<ExprKind, ParseError> {
        self.expect_keyword(kw::If)?;
        let cond = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.eat_keyword(kw::Else) {
            if self.eat_keyword(kw::If) {
                let start = self.token.span;
                let expr = self.parse_if_expr()?;
                Some(self.mk_expr(expr, start.to(self.prev_token.span)))
            } else {
                let start = self.token.span;
                let block = self.parse_block()?.into();
                Some(self.mk_expr(ExprKind::Block(block), start.to(self.prev_token.span)))
            }
        } else {
            None
        };

        Ok(ExprKind::If(
            If {
                cond,
                then_branch,
                else_branch,
            }
            .into(),
        ))
    }

    fn parse_while_expr(&mut self) -> Result<ExprKind, ParseError> {
        Ok(ExprKind::While(
            While {
                cond: self.parse_expr()?,
                body: self.parse_block()?,
            }
            .into(),
        ))
    }

    fn parse_for_expr(&mut self) -> Result<ExprKind, ParseError> {
        self.expect_keyword(kw::For)?;
        let pat = self.parse_pat()?;
        self.expect_keyword(kw::In)?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;

        Ok(ExprKind::For(For { pat, iter, body }.into()))
    }

    fn parse_loop_expr(&mut self) -> Result<ExprKind, ParseError> {
        self.expect_keyword(kw::Loop)?;
        Ok(ExprKind::Loop(self.parse_block()?.into()))
    }

    fn parse_return_expr(&mut self) -> Result<ExprKind, ParseError> {
        self.expect_keyword(kw::Return)?;
        let expr = if !self.eat(TokenKind::Semi) {
            Some(self.parse_expr()?.into())
        } else {
            None
        };

        Ok(ExprKind::Return(expr))
    }

    fn parse_break_expr(&mut self) -> Result<ExprKind, ParseError> {
        self.expect_keyword(kw::Break)?;
        let expr = if !self.eat(TokenKind::Semi) {
            Some(self.parse_expr()?.into())
        } else {
            None
        };

        Ok(ExprKind::Break(expr))
    }
    #[inline]
    fn mk_expr(&self, kind: ExprKind, span: Span) -> Expr {
        Expr { kind, span }
    }
}
