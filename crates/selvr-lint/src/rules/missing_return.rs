//! SL003 — missing_return
//!
//! Warns when a non-void function might not return a value.

use selvr_parser::ast::*;
use crate::{LintDiagnostic, LintSpan, Severity};
use crate::rules::unused_var::offset_to_line_col;

pub fn check(src: &str, module: &Module, out: &mut Vec<LintDiagnostic>) {
    for item in &module.items {
        if let Item::FnDef(f) = item { check_fn(src, f, out); }
    }
}

fn check_fn(src: &str, f: &FnDef, out: &mut Vec<LintDiagnostic>) {
    let returns_value = match &f.return_ty {
        None                => false,
        Some(Type::Void(_)) => false,
        Some(_)             => true,
    };
    if !returns_value { return; }
    if !block_always_returns(&f.body) {
        let (line, col) = offset_to_line_col(src, f.span.end.saturating_sub(1));
        out.push(LintDiagnostic {
            code:     "SL003",
            name:     "missing_return",
            severity: Severity::Error,
            message:  format!("function `{}` may not return a value on all paths", f.name),
            span:     LintSpan {
                start: f.span.end.saturating_sub(1) as u32,
                end:   f.span.end as u32,
                line:  line as u32,
                col:   col as u32,
            },
            fix: Some("ensure every code path has an explicit `return`".into()),
        });
    }
}

fn block_always_returns(b: &Block) -> bool {
    if b.tail.is_some() { return true; }
    b.stmts.iter().any(stmt_always_returns)
}

fn stmt_always_returns(s: &Stmt) -> bool {
    match s {
        Stmt::Expr(e, _) => expr_always_returns(e),
        _ => false,
    }
}

fn expr_always_returns(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Return { .. } | ExprKind::Break { .. } | ExprKind::Continue => true,
        ExprKind::If { then, else_: Some(el), .. } => {
            block_always_returns(then) && expr_always_returns(el)
        }
        ExprKind::Block(b) => block_always_returns(b),
        _ => false,
    }
}
