//! Abstract Syntax Tree node definitions for Selvr.
//!
//! Design goals:
//!  - Every node carries a `Span` for error reporting.
//!  - Names are interned as `SmolStr` (stack-allocated for ≤23 bytes, heap otherwise).
//!  - The tree is immutable after parsing; all mutation goes through a HIR lowering pass.

use smol_str::SmolStr;
use selvr_lexer::span::Span;

// ── Top-level ─────────────────────────────────────────────────────────────────

/// A single `.self` source file.
#[derive(Debug, Clone)]
pub struct Module {
    pub items: Vec<Item>,
    pub span: Span,
}

/// Top-level items that can appear in a module.
#[derive(Debug, Clone)]
pub enum Item {
    FnDef(FnDef),
    StructDef(StructDef),
    EnumDef(EnumDef),
    TraitDef(TraitDef),
    ImplBlock(ImplBlock),
    TypeAlias(TypeAlias),
    ImportDecl(ImportDecl), // import { X } from "module"
    ModDecl(ModDecl),
    Const(ConstItem),
    MacroDef(MacroDef),
}

// ── Visibility ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Default — visible only in the current module.
    Private,
    /// Declared with `export` — visible to importers.
    Public,
}

// ── Attributes ────────────────────────────────────────────────────────────────

/// A compile-time attribute applied to a function: `#[name]` or `#[name(arg)]`.
#[derive(Debug, Clone)]
pub struct Attr {
    pub name: SmolStr,
    pub args: Vec<SmolStr>, // e.g. #[cfg(target = "wasm")] → args = ["target = \"wasm\""]
    pub span: Span,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FnDef {
    pub vis: Visibility,
    pub name: SmolStr,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_ty: Option<Type>,
    pub body: Block,
    pub is_async: bool,
    pub attrs: Vec<Attr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: SmolStr,
    pub ty: Type,
    pub span: Span,
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Type {
    /// Named type, e.g. `i32`, `MyStruct`, `Option<T>`
    Named { name: SmolStr, args: Vec<Type>, span: Span },
    /// Tuple type, e.g. `(i32, str)`
    Tuple { elems: Vec<Type>, span: Span },
    /// Array type, e.g. `[i32; 4]`
    Array { elem: Box<Type>, len: Option<Box<Expr>>, span: Span },
    /// Function type, e.g. `fn(i32, str) -> bool`
    Fn { params: Vec<Type>, ret: Box<Type>, span: Span },
    /// `void` — no meaningful return value
    Void(Span),
    /// Inferred — written as `_` in the source
    Infer(Span),
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Named { span, .. } => *span,
            Type::Tuple { span, .. } => *span,
            Type::Array { span, .. } => *span,
            Type::Fn { span, .. }    => *span,
            Type::Void(s)            => *s,
            Type::Infer(s)           => *s,
        }
    }
}

// ── Generics ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: SmolStr,
    pub bounds: Vec<Type>,
    pub span: Span,
}

// ── Structs & Enums ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StructDef {
    pub vis: Visibility,
    pub name: SmolStr,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub vis: Visibility,
    pub name: SmolStr,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub vis: Visibility,
    pub name: SmolStr,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: SmolStr,
    pub kind: VariantKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum VariantKind {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<StructField>),
}

// ── Traits & Impls ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TraitDef {
    pub vis: Visibility,
    pub name: SmolStr,
    pub generics: Vec<GenericParam>,
    pub items: Vec<TraitItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TraitItem {
    FnSig(FnSig),
    FnDef(FnDef),
    TypeAssoc { name: SmolStr, bounds: Vec<Type>, span: Span },
}

#[derive(Debug, Clone)]
pub struct FnSig {
    pub name: SmolStr,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_ty: Option<Type>,
    pub is_async: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub generics: Vec<GenericParam>,
    pub trait_: Option<Type>,
    pub self_ty: Type,
    pub items: Vec<ImplItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ImplItem {
    Fn(FnDef),
    Const(ConstItem),
    TypeAssoc { name: SmolStr, ty: Type, span: Span },
}

// ── Type alias ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub vis: Visibility,
    pub name: SmolStr,
    pub generics: Vec<GenericParam>,
    pub ty: Type,
    pub span: Span,
}

// ── Import ────────────────────────────────────────────────────────────────────

/// `import { X, Y as Z } from "module"`
/// `import * from "module"`
#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub items: ImportItems,
    pub source: SmolStr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ImportItems {
    /// `import { X, Y as Z } from "..."`
    Named(Vec<ImportedName>),
    /// `import * from "..."`
    Glob,
}

#[derive(Debug, Clone)]
pub struct ImportedName {
    pub name: SmolStr,
    pub alias: Option<SmolStr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ModDecl {
    pub vis: Visibility,
    pub name: SmolStr,
    pub body: Option<Module>,
    pub span: Span,
}

// ── Constants ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ConstItem {
    pub vis: Visibility,
    pub name: SmolStr,
    pub ty: Option<Type>,
    pub value: Expr,
    pub span: Span,
}

// ── Macros ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MacroDef {
    pub name: SmolStr,
    pub body: TokenTree,
    pub span: Span,
}

/// Opaque placeholder — macro bodies are stored as token trees and expanded
/// during the HIR lowering phase.
#[derive(Debug, Clone)]
pub struct TokenTree {
    pub span: Span,
}

// ── Statements ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    /// The optional trailing expression (value the block evaluates to).
    pub tail: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Expr(Expr, Span),   // expression followed by `;`
    Item(Item),
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
    pub span: Span,
}

// ── Expressions ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    // Literals
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    StrLit(SmolStr),
    CharLit(char),

    // Name resolution
    Path(Path),

    // Operators
    Unary  { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Assign { target: Box<Expr>, rhs: Box<Expr> },
    CompoundAssign { op: BinaryOp, target: Box<Expr>, rhs: Box<Expr> },

    // Control flow
    If { cond: Box<Expr>, then: Block, else_: Option<Box<Expr>> },
    Match { scrutinee: Box<Expr>, arms: Vec<MatchArm> },
    Loop(Block),
    While { cond: Box<Expr>, body: Block },
    For   { pat: Pattern, iter: Box<Expr>, body: Block },
    Break { value: Option<Box<Expr>> },
    Continue,
    Return { value: Option<Box<Expr>> },

    // Calls & access
    Call { callee: Box<Expr>, args: Vec<Expr> },
    MethodCall { receiver: Box<Expr>, method: SmolStr, args: Vec<Expr> },
    Field { base: Box<Expr>, field: SmolStr },
    Index { base: Box<Expr>, index: Box<Expr> },

    // Constructors
    StructLit { ty: Path, fields: Vec<FieldInit> },
    TupleLit(Vec<Expr>),
    ArrayLit(Vec<Expr>),
    ArrayRepeat { elem: Box<Expr>, len: Box<Expr> },

    // Closures — `(x: i32, y: i32): i32 => expr`
    // params may omit types when inferable; body may be a block or single expression.
    Closure { params: Vec<ClosureParam>, ret: Option<Type>, body: Box<Expr> },

    // Template literal — `Hello ${name}, you are ${age} years old`
    // Segments and interpolated expressions are interleaved:
    //   [Segment("Hello "), Expr(name), Segment(", you are "), Expr(age), Segment(" years old")]
    TemplateLit { parts: Vec<TemplatePart> },

    // Async
    Await(Box<Expr>),

    // Type ascription / cast
    Cast { expr: Box<Expr>, ty: Type },

    // Block expression
    Block(Block),

    // Range
    Range { start: Option<Box<Expr>>, end: Option<Box<Expr>>, inclusive: bool },

    // Macro invocation
    MacroCall { name: SmolStr, args: TokenTree },
}

// ── Paths ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PathSegment {
    pub name: SmolStr,
    pub args: Vec<Type>,
    pub span: Span,
}

// ── Patterns ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum PatternKind {
    Wildcard,
    Ident { name: SmolStr, mutable: bool },
    Literal(LitPat),
    Tuple(Vec<Pattern>),
    Array(Vec<Pattern>),
    Struct { path: Path, fields: Vec<FieldPat>, rest: bool },
    TupleStruct { path: Path, elems: Vec<Pattern> },
    Or(Vec<Pattern>),
    Range { start: Option<Box<Pattern>>, end: Option<Box<Pattern>>, inclusive: bool },
}

#[derive(Debug, Clone)]
pub enum LitPat {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(SmolStr),
    Char(char),
}

#[derive(Debug, Clone)]
pub struct FieldPat {
    pub name: SmolStr,
    pub pat: Option<Pattern>,
    pub span: Span,
}

// ── Match arms ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
    pub span: Span,
}

// ── Struct field initializer ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FieldInit {
    pub name: SmolStr,
    pub value: Option<Expr>,
    pub span: Span,
}

// ── Closure params ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClosureParam {
    pub pat: Pattern,
    pub ty: Option<Type>,
    pub span: Span,
}

// ── Template literal parts ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum TemplatePart {
    /// A literal text segment (already un-escaped).
    Str(SmolStr),
    /// An interpolated expression `${expr}`.
    Expr(Box<Expr>),
}

// ── Operators ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp { Neg, Not, Deref }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Rem,
    BitAnd, BitOr, BitXor, Shl, Shr,
    And, Or,
    Eq, Ne, Lt, Le, Gt, Ge,
}
