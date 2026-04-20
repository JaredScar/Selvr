//! SL002 — dead_code
//!
//! Warns on non-exported functions never called within the same module.

use selvr_parser::ast::*;
use crate::{LintDiagnostic, LintSpan, Severity};
use crate::rules::unused_var::offset_to_line_col;
use std::collections::HashSet;

pub fn check(src: &str, module: &Module, out: &mut Vec<LintDiagnostic>) {
    let private_fns: Vec<(&str, usize, usize)> = module.items.iter()
        .filter_map(|i| match i {
            Item::FnDef(f) if f.vis == Visibility::Private => {
                let exempt = f.name == "main"
                    || f.attrs.iter().any(|a| a.name == "test");
                if exempt { None } else { Some((f.name.as_str(), f.span.start, f.span.end)) }
            }
            _ => None,
        })
        .collect();

    if private_fns.is_empty() { return; }

    let mut called: HashSet<String> = HashSet::new();
    for item in &module.items { collect_calls_item(item, &mut called); }

    for (name, start, end) in &private_fns {
        if !called.contains(*name) {
            let (line, col) = offset_to_line_col(src, *start);
            out.push(LintDiagnostic {
                code:     "SL002",
                name:     "dead_code",
                severity: Severity::Warning,
                message:  format!("function `{name}` is never called"),
                span:     LintSpan { start: *start as u32, end: *end as u32, line: line as u32, col: col as u32 },
                fix:      Some(format!("add `export` to expose `{name}`, or delete it")),
            });
        }
    }
}

fn collect_calls_item(item: &Item, out: &mut HashSet<String>) {
    match item {
        Item::FnDef(f)    => collect_calls_block(&f.body, out),
        Item::Const(c)    => collect_calls_expr(&c.value, out),
        Item::ImplBlock(i) => {
            for it in &i.items {
                if let ImplItem::Fn(f) = it { collect_calls_block(&f.body, out); }
            }
        }
        _ => {}
    }
}

fn collect_calls_block(b: &Block, out: &mut HashSet<String>) {
    for s in &b.stmts { collect_calls_stmt(s, out); }
    if let Some(tail) = &b.tail { collect_calls_expr(tail, out); }
}

fn collect_calls_stmt(s: &Stmt, out: &mut HashSet<String>) {
    match s {
        Stmt::Let(l) => { if let Some(i) = &l.init { collect_calls_expr(i, out); } }
        Stmt::Expr(e, _) => collect_calls_expr(e, out),
        Stmt::Item(_) => {}
    }
}

fn collect_calls_expr(e: &Expr, out: &mut HashSet<String>) {
    match &e.kind {
        ExprKind::Call { callee, args } => {
            if let ExprKind::Path(p) = &callee.kind {
                if let Some(seg) = p.segments.first() {
                    out.insert(seg.name.to_string());
                }
            }
            collect_calls_expr(callee, out);
            for a in args { collect_calls_expr(a, out); }
        }
        ExprKind::MethodCall { receiver, args, .. } => {
            collect_calls_expr(receiver, out);
            for a in args { collect_calls_expr(a, out); }
        }
        ExprKind::Binary { lhs, rhs, .. } | ExprKind::Assign { target: lhs, rhs }
        | ExprKind::CompoundAssign { target: lhs, rhs, .. }
        | ExprKind::Index { base: lhs, index: rhs } => {
            collect_calls_expr(lhs, out);
            collect_calls_expr(rhs, out);
        }
        ExprKind::Unary { expr, .. } | ExprKind::Await(expr)
        | ExprKind::Cast { expr, .. } | ExprKind::Field { base: expr, .. } => {
            collect_calls_expr(expr, out);
        }
        ExprKind::If { cond, then, else_ } => {
            collect_calls_expr(cond, out);
            collect_calls_block(then, out);
            if let Some(el) = else_ { collect_calls_expr(el, out); }
        }
        ExprKind::While { cond, body } => {
            collect_calls_expr(cond, out); collect_calls_block(body, out);
        }
        ExprKind::For { iter, body, .. } => {
            collect_calls_expr(iter, out); collect_calls_block(body, out);
        }
        ExprKind::Loop(b) => collect_calls_block(b, out),
        ExprKind::Match { scrutinee, arms } => {
            collect_calls_expr(scrutinee, out);
            for arm in arms {
                if let Some(g) = &arm.guard { collect_calls_expr(g, out); }
                collect_calls_expr(&arm.body, out);
            }
        }
        ExprKind::Return { value } | ExprKind::Break { value } => {
            if let Some(v) = value { collect_calls_expr(v, out); }
        }
        ExprKind::ArrayLit(elems) | ExprKind::TupleLit(elems) => {
            for e in elems { collect_calls_expr(e, out); }
        }
        ExprKind::ArrayRepeat { elem, len } => {
            collect_calls_expr(elem, out); collect_calls_expr(len, out);
        }
        ExprKind::StructLit { fields, .. } => {
            for f in fields { if let Some(v) = &f.value { collect_calls_expr(v, out); } }
        }
        ExprKind::Closure { body, .. } => collect_calls_expr(body, out),
        ExprKind::TemplateLit { parts } => {
            for p in parts { if let TemplatePart::Expr(e) = p { collect_calls_expr(e, out); } }
        }
        ExprKind::Block(b) => collect_calls_block(b, out),
        ExprKind::Range { start, end, .. } => {
            if let Some(s) = start { collect_calls_expr(s, out); }
            if let Some(e) = end   { collect_calls_expr(e, out); }
        }
        _ => {}
    }
}
