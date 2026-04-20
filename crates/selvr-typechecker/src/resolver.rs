//! Name resolution — maps every identifier reference to its definition site.
//!
//! Operates in two passes:
//!   1. Collect all top-level names (functions, structs, enums, type aliases).
//!   2. Walk each function body, resolving local bindings depth-first.
//!
//! Produces a `ResolutionMap` consumed by the inference engine.

use indexmap::IndexMap;
use smol_str::SmolStr;
use selvr_lexer::span::Span;
use crate::error::TypeError;

/// A stable ID for every definition site in a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(pub u32);

/// Maps each identifier `Span` to its resolved `DefId`.
pub type ResolutionMap = IndexMap<Span, DefId>;

/// All definitions collected from a module.
#[derive(Debug, Default)]
pub struct DefTable {
    pub defs: IndexMap<DefId, DefInfo>,
    next_id: u32,
}

impl DefTable {
    pub fn fresh(&mut self) -> DefId {
        let id = DefId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn insert(&mut self, name: SmolStr, span: Span, kind: DefKind) -> DefId {
        let id = self.fresh();
        self.defs.insert(id, DefInfo { name, span, kind });
        id
    }
}

#[derive(Debug, Clone)]
pub struct DefInfo {
    pub name: SmolStr,
    pub span: Span,
    pub kind: DefKind,
}

#[derive(Debug, Clone)]
pub enum DefKind {
    Fn,
    Struct,
    Enum,
    Trait,
    TypeAlias,
    Local { mutable: bool },
    Param,
    Const,
    Variant,
}

/// Lexical scope stack — each frame holds bindings introduced in that block.
pub struct Scope {
    frames: Vec<IndexMap<SmolStr, DefId>>,
}

impl Scope {
    pub fn new() -> Self {
        Self { frames: vec![IndexMap::new()] }
    }

    pub fn push(&mut self) {
        self.frames.push(IndexMap::new());
    }

    pub fn pop(&mut self) {
        self.frames.pop();
    }

    pub fn define(&mut self, name: SmolStr, id: DefId) {
        self.frames.last_mut().unwrap().insert(name, id);
    }

    pub fn lookup(&self, name: &str) -> Option<DefId> {
        for frame in self.frames.iter().rev() {
            if let Some(&id) = frame.get(name) {
                return Some(id);
            }
        }
        None
    }
}

/// The resolver — to be driven by the typechecker pipeline.
pub struct Resolver {
    pub defs: DefTable,
    pub resolution_map: ResolutionMap,
    pub errors: Vec<TypeError>,
    scope: Scope,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            defs: DefTable::default(),
            resolution_map: ResolutionMap::new(),
            errors: Vec::new(),
            scope: Scope::new(),
        }
    }

    pub fn resolve_name(&mut self, name: &SmolStr, span: Span) -> Option<DefId> {
        match self.scope.lookup(name) {
            Some(id) => {
                self.resolution_map.insert(span, id);
                Some(id)
            }
            None => {
                self.errors.push(TypeError::UnresolvedName { name: name.clone(), span });
                None
            }
        }
    }

    pub fn define_local(&mut self, name: SmolStr, span: Span, mutable: bool) -> DefId {
        let id = self.defs.insert(name.clone(), span, DefKind::Local { mutable });
        self.scope.define(name, id);
        id
    }

    pub fn push_scope(&mut self) { self.scope.push(); }
    pub fn pop_scope(&mut self)  { self.scope.pop(); }
}
