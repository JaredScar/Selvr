//! Hindley-Milner style type inference with unification.
//!
//! Algorithm W / Algorithm M hybrid:
//!  - Fresh type variables are generated for each expression.
//!  - `unify(a, b)` adds a constraint; the union-find structure resolves them.
//!  - After inference, `zonk(ty)` substitutes all solved variables.

use std::collections::HashMap;
use crate::ty::{Ty, TyVarId};
use crate::error::TypeError;
use selvr_lexer::span::Span;

/// Union-find substitution table.
pub struct Unifier {
    /// Maps a `TyVarId` to what it has been unified with (or itself if free).
    subst: HashMap<u32, Ty>,
    next_var: u32,
    pub errors: Vec<TypeError>,
}

impl Unifier {
    pub fn new() -> Self {
        Self { subst: HashMap::new(), next_var: 0, errors: Vec::new() }
    }

    /// Allocate a fresh, unconstrained type variable.
    pub fn fresh(&mut self) -> Ty {
        let id = TyVarId(self.next_var);
        self.next_var += 1;
        Ty::Var(id)
    }

    /// Follow the chain until we hit a non-variable or a free variable.
    pub fn find(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::Var(id) => match self.subst.get(&id.0) {
                Some(t) => self.find(t),
                None => ty.clone(),
            },
            _ => ty.clone(),
        }
    }

    /// Unify two types, recording any mismatch as an error.
    pub fn unify(&mut self, a: &Ty, b: &Ty, span: Span) {
        let a = self.find(a);
        let b = self.find(b);
        match (&a, &b) {
            // Two identical concrete types — always fine.
            (Ty::I32, Ty::I32) | (Ty::I64, Ty::I64) | (Ty::F32, Ty::F32)
            | (Ty::F64, Ty::F64) | (Ty::Bool, Ty::Bool) | (Ty::Str, Ty::Str)
            | (Ty::Char, Ty::Char) | (Ty::Void, Ty::Void) => {}

            // Bind a free variable.
            (Ty::Var(id), other) | (other, Ty::Var(id)) => {
                if !self.occurs(id.0, other) {
                    self.subst.insert(id.0, other.clone());
                } else {
                    self.errors.push(TypeError::InfiniteType { span });
                }
            }

            // Structural recursion for tuples.
            (Ty::Tuple(as_), Ty::Tuple(bs)) if as_.len() == bs.len() => {
                let pairs: Vec<_> = as_.iter().zip(bs.iter()).map(|(a, b)| (a.clone(), b.clone())).collect();
                for (a, b) in pairs { self.unify(&a, &b, span); }
            }

            // Generic application — both must have same ctor and arity.
            (Ty::App { ctor: ca, args: aa }, Ty::App { ctor: cb, args: ab })
                if ca == cb && aa.len() == ab.len() =>
            {
                let pairs: Vec<_> = aa.iter().zip(ab.iter()).map(|(a, b)| (a.clone(), b.clone())).collect();
                for (a, b) in pairs { self.unify(&a, &b, span); }
            }

            // Error sentinel — silently propagate without a second report.
            (Ty::Error, _) | (_, Ty::Error) => {}

            // Mismatch.
            _ => {
                self.errors.push(TypeError::TypeMismatch {
                    expected: a.display(),
                    found: b.display(),
                    span,
                });
            }
        }
    }

    /// Occurs check — prevents `T = Option<T>`.
    fn occurs(&self, id: u32, ty: &Ty) -> bool {
        match ty {
            Ty::Var(v) => v.0 == id,
            Ty::Tuple(ts) => ts.iter().any(|t| self.occurs(id, t)),
            Ty::App { args, .. } => args.iter().any(|t| self.occurs(id, t)),
            Ty::Array { elem, .. } => self.occurs(id, elem),
            Ty::Fn { params, ret, .. } => {
                params.iter().any(|t| self.occurs(id, t)) || self.occurs(id, ret)
            }
            _ => false,
        }
    }

    /// Substitute all solved variables in a type.
    pub fn zonk(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::Var(id) => match self.subst.get(&id.0) {
                Some(t) => self.zonk(t),
                None => ty.clone(),
            },
            Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(|t| self.zonk(t)).collect()),
            Ty::App { ctor, args } => Ty::App {
                ctor: ctor.clone(),
                args: args.iter().map(|t| self.zonk(t)).collect(),
            },
            Ty::Array { elem, len } => Ty::Array { elem: Box::new(self.zonk(elem)), len: *len },
            Ty::Fn { params, ret, is_async } => Ty::Fn {
                params: params.iter().map(|t| self.zonk(t)).collect(),
                ret: Box::new(self.zonk(ret)),
                is_async: *is_async,
            },
            _ => ty.clone(),
        }
    }
}
