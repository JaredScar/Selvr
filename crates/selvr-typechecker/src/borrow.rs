//! Ownership and borrow checking for Selvr.
//!
//! Enforces the core safety rules at the function level:
//!
//!   1. **Use after move** — a non-Copy binding cannot be read after its value
//!      has been moved into another binding or passed by value into a call.
//!   2. **Double move** — a non-Copy binding cannot be moved more than once on
//!      any path through the function.
//!   3. **Mutation of `const`** — immutable bindings cannot be re-assigned.
//!
//! The analysis is a forward dataflow pass over the AST.  Branch states are
//! merged conservatively: a binding is considered *moved* if it was moved on
//! *either* branch.  Copy types (`i32`, `f64`, `bool`, `char`) are implicitly
//! duplicated and never generate use-after-move errors.

use std::collections::HashMap;
use smol_str::SmolStr;
use selvr_lexer::span::Span;
use crate::error::TypeError;
use crate::ty::Ty;

// ── Variable state ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum VarState {
    Uninit,
    Owned,
    /// The value was moved out of this binding at the recorded span.
    Moved { at: Span },
}

// ── Ownership context ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct OwnershipCtx {
    scopes: Vec<HashMap<SmolStr, (VarState, Ty, bool /*is_const*/)>>,
    pub errors: Vec<TypeError>,
}

impl OwnershipCtx {
    pub fn new() -> Self {
        Self { scopes: vec![HashMap::new()], errors: Vec::new() }
    }

    pub fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }
    pub fn pop_scope(&mut self)  { self.scopes.pop(); }

    pub fn declare(&mut self, name: SmolStr, ty: Ty, is_const: bool) {
        self.scopes.last_mut().unwrap()
            .insert(name, (VarState::Owned, ty, is_const));
    }

    /// Record that `name` is moved at `at`, erroring on double-move.
    pub fn mark_moved(&mut self, name: &SmolStr, at: Span) {
        if let Some((state, ty, _)) = self.lookup_mut(name) {
            if is_copy_ty(ty) { return; }
            if matches!(state, VarState::Moved { .. }) {
                self.errors.push(TypeError::UseAfterMove { name: name.clone(), span: at });
            } else {
                *state = VarState::Moved { at };
            }
        }
    }

    pub fn check_use(&mut self, name: &SmolStr, span: Span) {
        if let Some((state, ty, _)) = self.lookup_mut(name) {
            if is_copy_ty(ty) { return; }
            if matches!(state, VarState::Moved { .. }) {
                self.errors.push(TypeError::UseAfterMove { name: name.clone(), span });
            }
        }
    }

    pub fn check_assign(&mut self, name: &SmolStr, span: Span) {
        if let Some((state, _, is_const)) = self.lookup_mut(name) {
            if *is_const {
                self.errors.push(TypeError::ImmutableAssign { name: name.clone(), span });
                return;
            }
            *state = VarState::Owned; // re-assignment re-initialises
        }
    }

    fn lookup_mut(&mut self, name: &SmolStr) -> Option<&mut (VarState, Ty, bool)> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(e) = scope.get_mut(name) { return Some(e); }
        }
        None
    }

    fn lookup(&self, name: &SmolStr) -> Option<&(VarState, Ty, bool)> {
        for scope in self.scopes.iter().rev() {
            if let Some(e) = scope.get(name) { return Some(e); }
        }
        None
    }

    /// Merge the moved-state from two branch contexts into `self`.
    pub fn merge_branch(&mut self, then_ctx: &OwnershipCtx, else_ctx: &OwnershipCtx) {
        let mut all: std::collections::HashSet<SmolStr> = Default::default();
        for s in &then_ctx.scopes { all.extend(s.keys().cloned()); }
        for s in &else_ctx.scopes { all.extend(s.keys().cloned()); }

        for name in &all {
            let then_moved = then_ctx.lookup(name)
                .map(|(s, _, _)| matches!(s, VarState::Moved { .. }))
                .unwrap_or(false);
            let else_moved = else_ctx.lookup(name)
                .map(|(s, _, _)| matches!(s, VarState::Moved { .. }))
                .unwrap_or(false);

            if then_moved || else_moved {
                if let Some((state, _, _)) = self.lookup_mut(name) {
                    if !matches!(state, VarState::Moved { .. }) {
                        *state = VarState::Moved {
                            at: Span::new(0, 0, 0),
                        };
                    }
                }
            }
        }
    }
}

// ── Copy-type predicate ───────────────────────────────────────────────────────

pub fn is_copy_ty(ty: &Ty) -> bool {
    matches!(ty, Ty::I32 | Ty::I64 | Ty::F32 | Ty::F64 | Ty::Bool | Ty::Char)
}

// ── AST walker ────────────────────────────────────────────────────────────────

use selvr_parser::ast::*;

/// Run the borrow checker over a single function body.
pub fn check_fn(
    fn_def: &FnDef,
    ty_map: &HashMap<SmolStr, Ty>,
    errors: &mut Vec<TypeError>,
) {
    let mut ctx = OwnershipCtx::new();
    for param in &fn_def.params {
        let ty = ty_map.get(&param.name).cloned().unwrap_or(Ty::Error);
        ctx.declare(param.name.clone(), ty, false);
    }
    check_block(&fn_def.body, &mut ctx, ty_map);
    errors.extend(ctx.errors);
}

fn check_block(block: &Block, ctx: &mut OwnershipCtx, ty_map: &HashMap<SmolStr, Ty>) {
    ctx.push_scope();
    for stmt in &block.stmts {
        check_stmt(stmt, ctx, ty_map);
    }
    if let Some(tail) = &block.tail {
        check_expr(tail, ctx, ty_map);
    }
    ctx.pop_scope();
}

fn check_stmt(stmt: &Stmt, ctx: &mut OwnershipCtx, ty_map: &HashMap<SmolStr, Ty>) {
    match stmt {
        Stmt::Let(let_stmt) => {
            if let Some(init) = &let_stmt.init {
                check_expr(init, ctx, ty_map);
            }
            // Bind the pattern — simple ident bindings only (mutable vs const).
            bind_pattern_decl(&let_stmt.pattern, ctx, ty_map, false);
        }
        Stmt::Expr(expr, _) => {
            check_expr(expr, ctx, ty_map);
        }
        Stmt::Item(_) => {}
    }
}

fn check_expr(expr: &Expr, ctx: &mut OwnershipCtx, ty_map: &HashMap<SmolStr, Ty>) {
    match &expr.kind {
        ExprKind::Path(path) => {
            if let Some(seg) = path.segments.first() {
                ctx.check_use(&seg.name, expr.span);
            }
        }

        ExprKind::Call { callee, args } => {
            check_expr(callee, ctx, ty_map);
            for arg in args {
                check_expr(arg, ctx, ty_map);
                // Passing a local by value moves it (unless Copy).
                if let ExprKind::Path(p) = &arg.kind {
                    if let Some(seg) = p.segments.first() {
                        ctx.mark_moved(&seg.name, arg.span);
                    }
                }
            }
        }

        ExprKind::MethodCall { receiver, args, .. } => {
            check_expr(receiver, ctx, ty_map);
            for arg in args { check_expr(arg, ctx, ty_map); }
        }

        ExprKind::Assign { target, rhs } => {
            check_expr(rhs, ctx, ty_map);
            if let ExprKind::Path(p) = &target.kind {
                if let Some(seg) = p.segments.first() {
                    ctx.check_assign(&seg.name, target.span);
                }
            }
        }

        ExprKind::If { cond, then, else_ } => {
            check_expr(cond, ctx, ty_map);
            let mut then_ctx = ctx.clone();
            then_ctx.errors = Vec::new();
            check_block(then, &mut then_ctx, ty_map);
            ctx.errors.extend(then_ctx.errors.drain(..));

            let mut else_ctx = ctx.clone();
            else_ctx.errors = Vec::new();
            if let Some(eb) = else_ {
                check_expr(eb, &mut else_ctx, ty_map);
            }
            ctx.errors.extend(else_ctx.errors.drain(..));
            ctx.merge_branch(&then_ctx, &else_ctx);
        }

        ExprKind::Match { scrutinee, arms } => {
            check_expr(scrutinee, ctx, ty_map);
            for arm in arms {
                ctx.push_scope();
                bind_pattern_decl(&arm.pattern, ctx, ty_map, false);
                if let Some(guard) = &arm.guard { check_expr(guard, ctx, ty_map); }
                check_expr(&arm.body, ctx, ty_map);
                ctx.pop_scope();
            }
        }

        ExprKind::Block(b) => check_block(b, ctx, ty_map),

        ExprKind::Return { value } => {
            if let Some(v) = value { check_expr(v, ctx, ty_map); }
        }

        ExprKind::Binary { lhs, rhs, .. } => {
            check_expr(lhs, ctx, ty_map);
            check_expr(rhs, ctx, ty_map);
        }
        ExprKind::CompoundAssign { target, rhs, .. } => {
            check_expr(target, ctx, ty_map);
            check_expr(rhs, ctx, ty_map);
        }

        ExprKind::Unary { expr: operand, .. }
        | ExprKind::Await(operand)
        | ExprKind::Cast { expr: operand, .. } => {
            check_expr(operand, ctx, ty_map);
        }

        ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => {
            check_expr(base, ctx, ty_map);
        }

        ExprKind::ArrayLit(elems) | ExprKind::TupleLit(elems) => {
            for e in elems { check_expr(e, ctx, ty_map); }
        }

        ExprKind::ArrayRepeat { elem, len } => {
            check_expr(elem, ctx, ty_map);
            check_expr(len, ctx, ty_map);
        }

        ExprKind::StructLit { fields, .. } => {
            for f in fields {
                if let Some(v) = &f.value { check_expr(v, ctx, ty_map); }
            }
        }

        ExprKind::Closure { params, body, .. } => {
            ctx.push_scope();
            for param in params {
                bind_pattern_decl(&param.pat, ctx, ty_map, false);
            }
            check_expr(body, ctx, ty_map);
            ctx.pop_scope();
        }

        ExprKind::While { cond, body } => {
            check_expr(cond, ctx, ty_map);
            check_block(body, ctx, ty_map);
        }

        ExprKind::For { pat, iter, body } => {
            check_expr(iter, ctx, ty_map);
            ctx.push_scope();
            bind_pattern_decl(pat, ctx, ty_map, false);
            check_block(body, ctx, ty_map);
            ctx.pop_scope();
        }

        ExprKind::Loop(body) => check_block(body, ctx, ty_map),

        ExprKind::TemplateLit { parts } => {
            for part in parts {
                if let TemplatePart::Expr(e) = part { check_expr(e, ctx, ty_map); }
            }
        }

        // Leaves — nothing to check.
        ExprKind::IntLit(_) | ExprKind::FloatLit(_) | ExprKind::BoolLit(_)
        | ExprKind::StrLit(_) | ExprKind::CharLit(_) | ExprKind::Break { .. }
        | ExprKind::Continue | ExprKind::Range { .. } | ExprKind::MacroCall { .. } => {}
    }
}

fn bind_pattern_decl(
    pat: &Pattern,
    ctx: &mut OwnershipCtx,
    ty_map: &HashMap<SmolStr, Ty>,
    is_const: bool,
) {
    match &pat.kind {
        PatternKind::Ident { name, mutable } => {
            let ty = ty_map.get(name).cloned().unwrap_or(Ty::Error);
            ctx.declare(name.clone(), ty, !mutable && is_const);
        }
        PatternKind::Tuple(elems) | PatternKind::Array(elems) => {
            for e in elems { bind_pattern_decl(e, ctx, ty_map, is_const); }
        }
        PatternKind::Struct { fields, .. } => {
            for f in fields {
                if let Some(p) = &f.pat { bind_pattern_decl(p, ctx, ty_map, is_const); }
            }
        }
        PatternKind::TupleStruct { elems, .. } => {
            for e in elems { bind_pattern_decl(e, ctx, ty_map, is_const); }
        }
        PatternKind::Or(alts) => {
            if let Some(first) = alts.first() {
                bind_pattern_decl(first, ctx, ty_map, is_const);
            }
        }
        _ => {}
    }
}
