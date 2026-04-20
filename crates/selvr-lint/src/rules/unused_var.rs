//! SL001 — unused_variable
//!
//! Warns when a `let` binding is declared but never read within its scope.
//! Bindings starting with `_` are exempt.

use selvr_parser::ast::*;
use selvr_lexer::span::Span;
use crate::{LintDiagnostic, LintSpan, Severity};
use std::collections::HashMap;

pub fn check(src: &str, module: &Module, out: &mut Vec<LintDiagnostic>) {
    for item in &module.items {
        if let Item::FnDef(f) = item { check_fn(src, f, out); }
    }
}

fn check_fn(src: &str, f: &FnDef, out: &mut Vec<LintDiagnostic>) {
    let mut bindings: HashMap<String, (Span, usize)> = HashMap::new();
    // Parameters are implicitly "used".
    for p in &f.params {
        bindings.insert(p.name.to_string(), (p.span, 1));
    }
    count_block(&f.body, &mut bindings);
    for (name, (span, count)) in &bindings {
        if *count == 0 && !name.starts_with('_') {
            out.push(LintDiagnostic {
                code:     "SL001",
                name:     "unused_variable",
                severity: Severity::Warning,
                message:  format!("variable `{name}` is declared but never used — prefix with `_` to silence"),
                span:     to_lint_span(src, *span),
                fix:      Some(format!("rename to `_{name}`")),
            });
        }
    }
}

fn count_block(block: &Block, b: &mut HashMap<String, (Span, usize)>) {
    for stmt in &block.stmts { count_stmt(stmt, b); }
    if let Some(tail) = &block.tail { count_expr(tail, b); }
}

fn count_stmt(stmt: &Stmt, b: &mut HashMap<String, (Span, usize)>) {
    match stmt {
        Stmt::Let(l) => {
            if let Some(init) = &l.init { count_expr(init, b); }
            // Register names introduced by the pattern.
            collect_pat_names(&l.pattern, l.span, b);
        }
        Stmt::Expr(e, _) => count_expr(e, b),
        Stmt::Item(_) => {}
    }
}

/// Register every `Ident` name introduced by a pattern (initial use-count = 0).
fn collect_pat_names(pat: &Pattern, span: Span, b: &mut HashMap<String, (Span, usize)>) {
    match &pat.kind {
        PatternKind::Ident { name, .. } => {
            b.entry(name.to_string()).or_insert((span, 0));
        }
        PatternKind::Tuple(elems) | PatternKind::Array(elems) => {
            for e in elems { collect_pat_names(e, span, b); }
        }
        PatternKind::Struct { fields, .. } => {
            for f in fields { if let Some(p) = &f.pat { collect_pat_names(p, span, b); } }
        }
        PatternKind::TupleStruct { elems, .. } => {
            for e in elems { collect_pat_names(e, span, b); }
        }
        PatternKind::Or(pats) => {
            for p in pats { collect_pat_names(p, span, b); }
        }
        _ => {}
    }
}

fn count_expr(e: &Expr, b: &mut HashMap<String, (Span, usize)>) {
    match &e.kind {
        ExprKind::Path(p) => {
            if let Some(seg) = p.segments.first() {
                if let Some(entry) = b.get_mut(seg.name.as_str()) {
                    entry.1 += 1;
                }
            }
        }
        ExprKind::Unary { expr, .. } | ExprKind::Await(expr)
        | ExprKind::Cast { expr, .. } => count_expr(expr, b),

        ExprKind::Binary { lhs, rhs, .. } | ExprKind::Assign { target: lhs, rhs }
        | ExprKind::CompoundAssign { target: lhs, rhs, .. }
        | ExprKind::Index { base: lhs, index: rhs } => {
            count_expr(lhs, b);
            count_expr(rhs, b);
        }
        ExprKind::If { cond, then, else_ } => {
            count_expr(cond, b);
            count_block(then, b);
            if let Some(el) = else_ { count_expr(el, b); }
        }
        ExprKind::While { cond, body } => { count_expr(cond, b); count_block(body, b); }
        ExprKind::For { iter, body, .. } => { count_expr(iter, b); count_block(body, b); }
        ExprKind::Loop(body) => count_block(body, b),
        ExprKind::Match { scrutinee, arms } => {
            count_expr(scrutinee, b);
            for arm in arms {
                if let Some(g) = &arm.guard { count_expr(g, b); }
                count_expr(&arm.body, b);
            }
        }
        ExprKind::Return { value } | ExprKind::Break { value } => {
            if let Some(v) = value { count_expr(v, b); }
        }
        ExprKind::Call { callee, args } => {
            count_expr(callee, b);
            for a in args { count_expr(a, b); }
        }
        ExprKind::MethodCall { receiver, args, .. } => {
            count_expr(receiver, b);
            for a in args { count_expr(a, b); }
        }
        ExprKind::Field { base, .. } => count_expr(base, b),
        ExprKind::ArrayLit(elems) | ExprKind::TupleLit(elems) => {
            for e in elems { count_expr(e, b); }
        }
        ExprKind::ArrayRepeat { elem, len } => { count_expr(elem, b); count_expr(len, b); }
        ExprKind::StructLit { fields, .. } => {
            for f in fields { if let Some(v) = &f.value { count_expr(v, b); } }
        }
        ExprKind::Closure { body, .. } => count_expr(body, b),
        ExprKind::TemplateLit { parts } => {
            for p in parts { if let TemplatePart::Expr(e) = p { count_expr(e, b); } }
        }
        ExprKind::Block(bl) => count_block(bl, b),
        ExprKind::Range { start, end, .. } => {
            if let Some(s) = start { count_expr(s, b); }
            if let Some(e) = end   { count_expr(e, b); }
        }
        _ => {}
    }
}

fn to_lint_span(src: &str, span: Span) -> LintSpan {
    let start = span.start as usize;
    let (line, col) = offset_to_line_col(src, start);
    LintSpan { start: span.start as u32, end: span.end as u32, line: line as u32, col: col as u32 }
}

pub fn offset_to_line_col(src: &str, offset: usize) -> (usize, usize) {
    let slice = &src[..offset.min(src.len())];
    let line  = slice.chars().filter(|&c| c == '\n').count() + 1;
    let col   = slice.rfind('\n').map_or(slice.len(), |p| slice.len() - p - 1) + 1;
    (line, col)
}
