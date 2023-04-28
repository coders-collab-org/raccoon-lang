use crate::{path::PathStyle, ParseError, Parser};
use raccoon_ast::{
    BinOpToken, Block, Crate, Delimiter, Enum, EnumVariant, EnumVariantFields, Fn, FnSig, Inline,
    Item, ItemKind, ModKind, Param, Struct, StructFields, TokenKind, Ty, UseTree, UseTreeKind,
    Visibility,
};
use raccoon_span::{kw, Ident, Span};
use thin_vec::{thin_vec, ThinVec};

impl Parser<'_> {
    pub fn parse_crate(&mut self) -> Result<Crate, ParseError> {
        let (items, span) = self.parse_mod()?;

        Ok(Crate { items, span })
    }

    pub fn parse_item(&mut self) -> Result<Option<Item>, ParseError> {
        let vis = self.parse_visibility()?;
        let start = self.token.span;
        let item_type = self.parse_ident(false)?;

        let (kind, ident) = match item_type.name {
            kw::Fn => self.parse_fn_item()?,
            kw::Struct => self.parse_struct_item()?,
            kw::Enum => self.parse_enum_item()?,
            kw::Mod => self.parse_mod_item()?,
            kw::Use => self.parse_use_item()?,

            _ => {
                if vis == Visibility::Inherited {
                    return Ok(None);
                } else {
                    return Err(ParseError);
                }
            }
        };

        Ok(Some(Item {
            vis,
            kind,
            ident,
            span: start.to(self.prev_token.span),
        }))
    }

    fn parse_visibility(&mut self) -> Result<Visibility, ParseError> {
        if !self.eat_keyword(kw::Pub) {
            return Ok(Visibility::Inherited);
        }

        // TODO: Parse `pub(crate)`
        Ok(Visibility::Public)
    }
}

type ItemInfo = (ItemKind, Ident);

// parse functions and methods
impl Parser<'_> {
    pub fn parse_fn_item(&mut self) -> Result<ItemInfo, ParseError> {
        let start = self.token.span;
        if !self.eat_keyword(kw::Fn) {
            return Err(ParseError);
        }
        let ident = self.parse_ident(true)?;

        let sig = {
            let params = self.parse_parenthesized(|p| p.parse_param())?;

            let ret_ty = if self.eat(TokenKind::RArrow) {
                Some(self.parse_ty()?)
            } else {
                None
            };

            FnSig {
                params,
                ret_ty,
                span: start.to(self.prev_token.span),
            }
        };

        Ok((
            ItemKind::Fn(
                Fn {
                    sig,
                    body: self.parse_body()?,
                }
                .into(),
            ),
            ident,
        ))
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let start = self.token.span;
        let ident = self.parse_pat()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_ty()?;
        Ok(Param {
            ident,
            ty,
            span: start.to(self.prev_token.span),
        })
    }

    fn parse_body(&mut self) -> Result<Block, ParseError> {
        self.parse_block()
    }
}

/// parse structs and enums;
impl Parser<'_> {
    pub fn parse_struct_item(&mut self) -> Result<ItemInfo, ParseError> {
        if !self.eat_keyword(kw::Struct) {
            return Err(ParseError);
        }

        let ident = self.parse_ident(true)?;

        let fields = if self.check_delim(Delimiter::Brace) {
            StructFields::Struct(self.parse_braced(|p| p.parse_struct_field())?)
        } else if self.check_delim(Delimiter::Paren) {
            let fields = StructFields::Tuple(self.parse_parenthesized(|p| p.parse_tuple_field())?);
            self.expect_semi()?;
            fields
        } else {
            self.expect_semi()?;
            StructFields::Unit
        };

        Ok((ItemKind::Struct(Struct { fields }.into()), ident))
    }
    fn parse_struct_field(&mut self) -> Result<(Visibility, Ident, Ty), ParseError> {
        let vis = self.parse_visibility()?;
        let ident = self.parse_ident(true)?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_ty()?;
        Ok((vis, ident, ty))
    }

    fn parse_tuple_field(&mut self) -> Result<(Visibility, Ty), ParseError> {
        let vis = self.parse_visibility()?;
        let ty = self.parse_ty()?;
        Ok((vis, ty))
    }

    pub fn parse_enum_item(&mut self) -> Result<ItemInfo, ParseError> {
        if !self.eat_keyword(kw::Enum) {
            return Err(ParseError);
        }

        let ident = self.parse_ident(true)?;

        let variants = if self.check_delim(Delimiter::Brace) {
            Some(self.parse_braced(|p| p.parse_variant())?)
        } else {
            self.expect_semi()?;
            None
        };

        Ok((ItemKind::Enum(Enum { variants }.into()), ident))
    }

    fn parse_variant(&mut self) -> Result<EnumVariant, ParseError> {
        let ident = self.parse_ident(true)?;

        let fields = if self.check_delim(Delimiter::Brace) {
            EnumVariantFields::Struct(self.parse_braced(|p| p.parse_struct_field())?)
        } else if self.check_delim(Delimiter::Paren) {
            EnumVariantFields::Tuple(self.parse_parenthesized(|p| p.parse_tuple_field())?)
        } else {
            EnumVariantFields::Unit
        };

        Ok(EnumVariant { ident, fields })
    }
}

/// parse modules and use statements
impl Parser<'_> {
    pub fn parse_mod_item(&mut self) -> Result<ItemInfo, ParseError> {
        if !self.eat_keyword(kw::Mod) {
            return Err(ParseError);
        }

        let ident = self.parse_ident(true)?;

        let kind = if self.eat(TokenKind::Semi) {
            ModKind::Unloaded
        } else {
            let (items, span) = self.parse_mod()?;
            ModKind::Loaded(items, Inline::Yes, span)
        };

        Ok((ItemKind::Mod(kind.into()), ident))
    }

    pub fn parse_mod(&mut self) -> Result<(ThinVec<Item>, Span), ParseError> {
        let start = self.token.span;
        let mut items = thin_vec![];
        while !self.lexer.is_eof() {
            let Some(item) = self.parse_item()? else {
                continue;
            };
            items.push(item);
        }
        Ok((items, start.to(self.prev_token.span)))
    }

    pub fn parse_use_item(&mut self) -> Result<ItemInfo, ParseError> {
        if !self.eat_keyword(kw::Use) {
            return Err(ParseError);
        }

        let tree = self.parse_use_tree()?;

        Ok((ItemKind::Use(tree.into()), Ident::empty()))
    }

    fn parse_use_tree(&mut self) -> Result<UseTree, ParseError> {
        let start = self.token.span;
        let base = self.parse_path(PathStyle::Mod)?;

        let kind = if self.eat(TokenKind::DoubleColon) {
            if self.eat(TokenKind::BinOp(BinOpToken::Mul)) {
                UseTreeKind::Glob
            } else {
                UseTreeKind::Nested(self.parse_braced(|p| p.parse_use_tree())?)
            }
        } else {
            UseTreeKind::Single(if self.eat_keyword(kw::As) {
                Some(self.parse_ident(true)?)
            } else {
                None
            })
        };

        Ok(UseTree {
            base,
            kind,
            span: start.to(self.prev_token.span),
        })
    }
}
