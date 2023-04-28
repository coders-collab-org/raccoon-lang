use raccoon_span::{Ident, Span, Symbol};
use thin_vec::{thin_vec, ThinVec};

use crate::Lit;

/// Crate is the root of the AST.
pub struct Crate {
    pub items: ThinVec<Item>,
    pub span: Span,
}

/// An item (e.g. `fn foo() {}`, `struct Bar;`, `extern { ... }`, etc.)
pub struct Item {
    /// Visibility of the item (e.g. `pub`, `pub(crate)`, etc.)
    pub vis: Visibility,

    /// Name of the item (e.g. `foo` for `fn foo() {}`)
    pub ident: Ident,

    /// Kind of the item (e.g. `Fn`, `Struct`, etc.)
    pub kind: ItemKind,

    /// span of the entire item
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A visibility qualifier (e.g. `pub`, `pub(crate)`, etc.)
pub enum Visibility {
    /// `pub`
    Public,

    /// `pub(crate)`
    Crate,

    /// No visibility qualifier
    Inherited,
}

/// The kind of an item (e.g. `Fn`, `Struct`, `ExternCrate`, etc.)
pub enum ItemKind {
    /// A module declaration.
    Mod(Box<ModKind>),

    /// A use declaration.
    Use(Box<UseTree>),

    /// A function declaration.
    Fn(Box<Fn>),

    /// A struct declaration.
    Struct(Box<Struct>),

    /// An enum declaration.
    Enum(Box<Enum>),
}

/// A module declaration.
pub enum ModKind {
    /// A module with a body.
    Loaded(ThinVec<Item>, Inline, Span),

    /// A module with an external body.
    Unloaded,
}

/// A use declaration.
pub struct UseTree {
    /// The path of the use tree.
    pub base: Path,

    /// The use tree's kind.
    pub kind: UseTreeKind,

    /// span of the entire use tree
    pub span: Span,
}

/// A use tree kind (e.g. `use foo::bar`, `use foo::bar as baz`, etc.)
pub enum UseTreeKind {
    /// A use tree with a simple path (e.g. `use foo::bar` or `use foo::bar as baz`)
    Single(Option<Ident>),

    /// A use tree with a nested use tree (e.g. `use foo::{bar, baz}`)
    Nested(ThinVec<UseTree>),

    /// A use tree with a glob (e.g. `use foo::*`)
    Glob,
}

pub enum Inline {
    Yes,
    No,
}

/// A function declaration.
pub struct Fn {
    /// The function's signature.
    pub sig: FnSig,

    /// The function's body.
    pub body: Block,
}

/// A function signature.
pub struct FnSig {
    /// The function's parameters.
    pub params: ThinVec<Param>,

    /// The function's return type.
    pub ret_ty: Option<Ty>,

    /// span of the entire function signature
    pub span: Span,
}

/// A function parameter.
pub struct Param {
    /// The parameter's name.
    pub ident: Pat,

    /// The parameter's type.
    pub ty: Ty,

    /// span of the entire parameter
    pub span: Span,
}

/// A type (e.g. `int`, `str`, etc.)
pub struct Ty {
    /// The type's kind.
    pub kind: TyKind,

    /// span of the entire type
    pub span: Span,
}

/// A type kind (e.g. `int`, `str[]`, etc.)
pub enum TyKind {
    /// An Array type (e.g. `int[]`)
    Array(Box<Ty>),

    /// A Tuple type (e.g. `(int, str)`)
    Tuple(ThinVec<Ty>),

    /// We will use this type for
    Paren(Box<Ty>),

    /// Represents an inferred type.
    Infer,

    /// Represents a method that has an implicit `self` parameter.
    ImplicitSelf,

    /// A Path type (e.g. `std::vec::Vec`)
    Path(Path),

    /// an unit type (e.g. `()`)
    Unit,
}

/// A block of statements.
pub struct Block {
    /// The block's statements.
    pub stmts: ThinVec<Stmt>,

    /// span of the entire block
    pub span: Span,
}

/// A statement (e.g. `let x = 1;`, `x = 2;`, etc.)
pub struct Stmt {
    /// The statement's kind.
    pub kind: StmtKind,

    /// span of the entire statement
    pub span: Span,
}

/// A statement kind (e.g. `let x = 1;`, `x = 2;`, etc.)
pub enum StmtKind {
    /// A item statement (e.g. `fn foo() {}`).
    Item(Box<Item>),

    /// A let statement (e.g. `let x = 1;`).
    Let(Box<Let>),

    /// An expression statement (e.g. `x = 2;`).
    Expr(Box<Expr>),

    /// A semi-colon statement (e.g. `x;`).
    Semi(Box<Expr>),

    /// An empty statement (e.g. `;`).
    Empty,
}

/// A let statement (e.g. `let x = 1;`).
pub struct Let {
    /// The let statement's pattern.
    pub pat: Pat,

    /// The let statement's type.
    pub ty: Option<Ty>,

    /// The let statement's initializer.
    pub init: Option<Expr>,

    /// span of the entire let statement
    pub span: Span,
}

/// A pattern (e.g. `x`, `(x, y)`, `Foo { x, y }`, etc.)
pub struct Pat {
    /// The pattern's kind.
    pub kind: PatKind,

    /// span of the entire pattern
    pub span: Span,
}

/// A pattern kind (e.g. `x`, `(x, y)`, `Foo { x, y }`, etc.)
pub enum PatKind {
    /// A variable pattern (e.g. `x`).
    Ident(Ident),

    /// A tuple pattern (e.g. `(x, y)`).
    Tuple(ThinVec<Pat>),

    /// A path pattern (e.g. `Foo::Bar`).
    Path(Path),

    /// A struct pattern (e.g. `Foo { x, y }`).
    Struct(Box<StructPat>),

    /// A enum pattern (e.g. `Foo::Bar(x, y)`).
    Enum(Box<EnumPat>),

    /// A slice pattern (e.g. `[x, y]`).
    Slice(ThinVec<Pat>),
}

/// A struct pattern (e.g. `Foo { x, y }`).
pub struct StructPat {
    /// The struct pattern's path.
    pub path: Path,

    /// The struct pattern's fields.
    pub fields: ThinVec<StructPatField>,
}
/// A struct pattern field (e.g. `x` in `Foo { x, y }`).
pub struct StructPatField {
    /// The struct pattern field's name.
    pub ident: Ident,

    /// The struct pattern field's pattern.
    pub pat: Option<Pat>,

    /// span of the entire struct pattern field
    pub span: Span,
}

/// An enum pattern (e.g. `Foo::Bar(x, y)`).
pub struct EnumPat {
    /// The enum pattern's path.
    pub path: Path,

    /// The enum pattern's fields.
    pub fields: ThinVec<Pat>,
}

/// An expression (e.g. `1`, `x + 1`, etc.)
pub struct Expr {
    /// The expression's kind.
    pub kind: ExprKind,

    /// span of the entire expression
    pub span: Span,
}

/// An expression kind (e.g. `1`, `x + 1`, etc.)
pub enum ExprKind {
    /// A literal expression (e.g. `1`).
    Lit(Lit),

    /// A binary expression (e.g. `x + 1`).
    Binary(Box<Binary>),

    /// An assignment expression (e.g. `x = 1`).
    Assign(Box<Assign>),

    /// An assignment operation (e.g. `x += 1`).
    AssignOp(Box<Binary>),

    /// A unary expression (e.g. `!x`).
    Unary(Box<Unary>),

    /// A path expression (e.g. `foo::bar`).
    Path(Path),

    /// A call expression (e.g. `foo(1)`).
    Call(Box<Call>),

    /// an indexing expression (e.g. `foo[1]`).
    Index(Box<Index>),

    /// A field access expression (e.g. `foo.bar` or `foo.0`).
    Field(Box<Field>),

    /// A struct expression (e.g. `#{ x, y }`).
    Struct(ThinVec<StructExprField>),

    /// A tuple expression (e.g. `(x, y)`).
    Tuple(ThinVec<Expr>),

    /// A slice expression (e.g. `[x, y]`).
    Array(ThinVec<Expr>),

    /// A block expression (e.g. `{ let x = 1; x }`).
    Block(Box<Block>),

    /// An if expression (e.g. `if x { 1 } else { 2 }`).
    If(Box<If>),

    /// A loop expression (e.g. `loop { x += 1; }`).
    Loop(Box<Block>),

    /// A while expression (e.g. `while x { x += 1; }`).
    While(Box<While>),

    /// A for expression (e.g. `for x in y { x += 1; }`).
    For(Box<For>),

    /// A match expression (e.g. `match x { 1 => 2, _ => 3 }`).
    Match(Box<Match>),

    /// A return expression (e.g. `return 1`).
    Return(Option<Box<Expr>>),

    /// A break expression (e.g. `break`).
    Break(Option<Box<Expr>>),

    /// A continue expression (e.g. `continue`).
    Continue,

    /// A parenthesized expression (e.g. `(x * y)`).
    Paren(Box<Expr>),
}

/// An assignment expression (e.g. `x = 1`).
pub struct Assign {
    /// The assignment expression's left-hand side.
    pub lhs: Expr,

    /// The assignment expression's right-hand side.
    pub rhs: Expr,
}

/// A binary expression (e.g. `x + 1`).
pub struct Binary {
    /// The binary expression's left-hand side.
    pub lhs: Expr,

    /// The binary expression's operator.
    pub op: BinOp,

    /// The binary expression's right-hand side.
    pub rhs: Expr,
}

/// A binary operator (e.g. `+`).
pub struct BinOp {
    /// The binary operator's kind.
    pub kind: BinOpKind,

    /// span of the entire binary operator
    pub span: Span,
}

/// A binary operator kind (e.g. `+`).
pub enum BinOpKind {
    /// A `+` operator.
    Add,

    /// A `-` operator.
    Sub,

    /// A `*` operator.
    Mul,

    /// A `/` operator.
    Div,

    /// A `%` operator.
    Rem,

    /// A `&` operator.
    BitAnd,

    /// A `|` operator.
    BitOr,

    /// A `^` operator.
    BitXor,

    /// A `<<` operator.
    Shl,

    /// A `>>` operator.
    Shr,

    /// A `&&` operator.
    And,

    /// A `||` operator.
    Or,

    /// A `==` operator.
    Eq,

    /// A `!=` operator.
    Ne,

    /// A `<` operator.
    Lt,

    /// A `<=` operator.
    Le,

    /// A `>` operator.
    Gt,

    /// A `>=` operator.
    Ge,
}

/// A unary expression (e.g. `!x`).
pub struct Unary {
    /// The unary expression's operator.
    pub op: UnaryOp,

    /// The unary expression's operand.
    pub expr: Expr,
}

/// A unary operator (e.g. `!`).
pub struct UnaryOp {
    /// The unary operator's kind.
    pub kind: UnaryOpKind,

    /// span of the entire unary operator
    pub span: Span,
}

/// A unary operator kind (e.g. `!`).
pub enum UnaryOpKind {
    /// A `!` operator.
    Not,

    /// A `-` operator.
    Neg,

    /// A `~` operator.
    BitNot,
}

/// A call expression (e.g. `foo(1)`).
pub struct Call {
    /// The call expression's function.
    pub callee: Expr,

    /// The call expression's arguments.
    pub args: ThinVec<Expr>,
}

/// A field access expression (e.g. `foo.bar` or `foo.0`).
pub struct Field {
    /// The field access expression's base.
    pub base: Expr,

    /// The field access expression's field.
    pub kind: FieldKind,
}

pub enum FieldKind {
    /// A named field (e.g. `foo.bar`).
    Named(Ident),

    /// An unnamed field (e.g. `foo.0`).
    Unnamed(Symbol),
}

/// A struct expression field (e.g. `x`).
pub struct StructExprField {
    /// The struct expression field's name.
    pub name: Ident,

    /// The struct expression field's value.
    pub value: Expr,

    /// The span of the entire struct expression field.
    pub span: Span,
}

/// An if expression (e.g. `if x { 1 } else { 2 }`).
pub struct If {
    /// The if expression's condition.
    pub cond: Expr,

    /// The if expression's then block.
    pub then_branch: Block,

    /// The if expression's else block.
    pub else_branch: Option<Expr>,
}

/// A while expression (e.g. `while x { x += 1; }`).
pub struct While {
    /// The while expression's condition.
    pub cond: Expr,

    /// The while expression's body.
    pub body: Block,
}

/// A for expression (e.g. `for x in y { x += 1; }`).
pub struct For {
    /// The for expression's variable.
    pub pat: Pat,

    /// The for expression's iterator.
    pub iter: Expr,

    /// The for expression's body.
    pub body: Block,
}

/// A match expression (e.g. `match x { 1 => 2, _ => 3 }`).
pub struct Match {
    /// The match expression's discriminant.
    pub discriminant: Expr,

    /// The match expression's arms.
    pub arms: ThinVec<MatchArm>,
}

/// A match expression arm (e.g. `1 => 2`).
pub struct MatchArm {
    /// The match expression arm's pattern.
    pub pattern: Pat,

    /// The match expression arm's body.
    pub body: Expr,
}

/// A index expression (e.g. `foo[1]`).
pub struct Index {
    /// The index expression's base.
    pub base: Expr,

    /// The index expression's index.
    pub index: Expr,
}

/// A path (e.g. `std::mem::replace`).
pub struct Path {
    /// The path's segments.
    pub segments: ThinVec<PathSegment>,
}

/// A path segment (e.g. `mem` in `std::mem::replace`).
pub struct PathSegment {
    /// The path segment's identifier.
    pub ident: Ident,

    /// The path segment's span.
    pub span: Span,
}

/// A struct declaration.
pub struct Struct {
    /// The struct's fields.
    pub fields: StructFields,
}

pub enum StructFields {
    /// A tuple struct (e.g. `struct Foo(u32, u32)`).
    Tuple(ThinVec<(Visibility, Ty)>),

    /// A struct (e.g. `struct Foo { x: u32, y: u32 }`).
    Struct(ThinVec<(Visibility, Ident, Ty)>),

    /// A unit struct (e.g. `struct Foo`).
    Unit,
}

/// A enum declaration.
pub struct Enum {
    /// The enum's variants.
    pub variants: Option<ThinVec<EnumVariant>>,
}

/// A enum variant (e.g. `Foo` in `enum Foo { ... }`).
pub struct EnumVariant {
    /// The enum variant's name.
    pub ident: Ident,

    /// The enum variant's fields.
    pub fields: EnumVariantFields,
}

/// A enum variant fields (e.g. `Foo(u32, u32)`).
pub enum EnumVariantFields {
    /// A tuple enum variant (e.g. `Foo(u32, u32)`).
    Tuple(ThinVec<(Visibility, Ty)>),

    /// A struct enum variant (e.g. `Foo { x: u32, y: u32 }`).
    Struct(ThinVec<(Visibility, Ident, Ty)>),

    /// A unit enum variant (e.g. `Foo`).
    Unit,
}

impl From<Ident> for Path {
    fn from(ident: Ident) -> Path {
        Path {
            segments: thin_vec![PathSegment {
                ident,
                span: ident.span,
            }],
        }
    }
}
