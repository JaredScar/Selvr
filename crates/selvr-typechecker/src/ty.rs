//! Semantic type representation used by the type checker.
//!
//! `Ty` is what the inference engine works with — distinct from the syntactic
//! `ast::Type` nodes produced by the parser.

use smol_str::SmolStr;
use indexmap::IndexMap;

/// A unique type variable ID produced during inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TyVarId(pub u32);

/// A fully-resolved semantic type.
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    // ── Primitive ─────────────────────────────────────────────────────────────
    I32,
    I64,
    F32,
    F64,
    Bool,
    Str,
    Char,
    Void,

    // ── Compound ──────────────────────────────────────────────────────────────
    Tuple(Vec<Ty>),
    Array { elem: Box<Ty>, len: Option<usize> },
    Fn    { params: Vec<Ty>, ret: Box<Ty>, is_async: bool },
    Struct { name: SmolStr, fields: IndexMap<SmolStr, Ty> },
    Enum   { name: SmolStr, variants: IndexMap<SmolStr, VariantTy> },

    // ── Generic application ───────────────────────────────────────────────────
    /// e.g. `Option<i32>`, `Vec<str>`
    App { ctor: SmolStr, args: Vec<Ty> },

    // ── Inference variable ────────────────────────────────────────────────────
    Var(TyVarId),

    // ── Error sentinel (propagated silently after first report) ───────────────
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariantTy {
    Unit,
    Tuple(Vec<Ty>),
    Struct(IndexMap<SmolStr, Ty>),
}

impl Ty {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::I32 | Ty::I64 | Ty::F32 | Ty::F64)
    }

    pub fn is_integral(&self) -> bool {
        matches!(self, Ty::I32 | Ty::I64)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Ty::Error)
    }

    /// Human-readable name for error messages.
    pub fn display(&self) -> String {
        match self {
            Ty::I32   => "i32".into(),
            Ty::I64   => "i64".into(),
            Ty::F32   => "f32".into(),
            Ty::F64   => "f64".into(),
            Ty::Bool  => "bool".into(),
            Ty::Str   => "str".into(),
            Ty::Char  => "char".into(),
            Ty::Void  => "void".into(),
            Ty::Tuple(ts) => {
                let inner = ts.iter().map(|t| t.display()).collect::<Vec<_>>().join(", ");
                format!("({inner})")
            }
            Ty::App { ctor, args } if args.is_empty() => ctor.to_string(),
            Ty::App { ctor, args } => {
                let inner = args.iter().map(|t| t.display()).collect::<Vec<_>>().join(", ");
                format!("{ctor}<{inner}>")
            }
            Ty::Var(id) => format!("?T{}", id.0),
            Ty::Error => "<error>".into(),
            Ty::Struct { name, .. } => name.to_string(),
            Ty::Enum   { name, .. } => name.to_string(),
            Ty::Array { elem, len: Some(n) } => format!("[{}; {n}]", elem.display()),
            Ty::Array { elem, len: None }     => format!("[{}]", elem.display()),
            Ty::Fn { params, ret, .. } => {
                let ps = params.iter().map(|t| t.display()).collect::<Vec<_>>().join(", ");
                format!("fn({ps}) -> {}", ret.display())
            }
        }
    }
}
