//! SL004 — shadow_variable
//!
//! Warns when a `let` binding shadows an outer-scope binding of the same name.

use selvr_parser::ast::*;
use crate::{LintDiagnostic, LintSpan, Severity};
use crate::rules::unused_var::offset_to_line_col;

pub fn check(src: &str, module: &Module, out: &mut Vec<LintDiagnostic>) {
    for item in &module.items {
        if let Item::FnDef(f) = item { check_fn(src, f, out); }
    }
}

fn check_fn(src: &str, f: &FnDef, out: &mut Vec<LintDiagnostic>) {
    let outer: Vec<String> = f.params.iter().map(|p| p.name.to_string()).collect();
    check_block(src, &f.body, &outer, out);
}

fn check_block(src: &str, b: &Block, outer: &[String], out: &mut Vec<LintDiagnostic>) {
    let mut scope: Vec<String> = outer.to_vec();
    for stmt in &b.stmts {
        if let Stmt::Let(bind) = stmt {
            // Check each name introduced by the pattern.
            for name in pat_names(&bind.pattern) {
                if scope.contains(&name) {
                    let (line, col) = offset_to_line_col(src, bind.span.start);
                    out.push(LintDiagnostic {
                        code:     "SL004",
                        name:     "shadow_variable",
                        severity: Severity::Warning,
                        message:  format!("`{name}` shadows an outer binding"),
                        span:     LintSpan {
                            start: bind.span.start as u32,
                            end:   bind.span.end   as u32,
                            line:  line as u32,
                            col:   col  as u32,
                        },
                        fix: Some("rename this binding".into()),
                    });
                }
                scope.push(name);
            }
        }
        // Recurse into nested blocks inside expressions.
        if let Stmt::Expr(e, _) = stmt {
            check_nested_blocks(src, e, &scope, out);
        }
    }
    if let Some(tail) = &b.tail {
        check_nested_blocks(src, tail, &scope, out);
    }
}

fn check_nested_blocks(src: &str, e: &Expr, scope: &[String], out: &mut Vec<LintDiagnostic>) {
    match &e.kind {
        ExprKind::If { then, else_, .. } => {
            check_block(src, then, scope, out);
            if let Some(el) = else_ { check_nested_blocks(src, el, scope, out); }
        }
        ExprKind::While { body, .. } | ExprKind::For { body, .. } | ExprKind::Loop(body) => {
            check_block(src, body, scope, out);
        }
        ExprKind::Block(b) => check_block(src, b, scope, out),
        ExprKind::Match { arms, .. } => {
            for arm in arms { check_nested_blocks(src, &arm.body, scope, out); }
        }
        _ => {}
    }
}

fn pat_names(pat: &Pattern) -> Vec<String> {
    let mut names = Vec::new();
    collect_names(pat, &mut names);
    names
}

fn collect_names(pat: &Pattern, out: &mut Vec<String>) {
    match &pat.kind {
        PatternKind::Ident { name, .. } => out.push(name.to_string()),
        PatternKind::Tuple(elems) | PatternKind::Array(elems) => {
            for e in elems { collect_names(e, out); }
        }
        PatternKind::Struct { fields, .. } => {
            for f in fields { if let Some(p) = &f.pat { collect_names(p, out); } }
        }
        PatternKind::TupleStruct { elems, .. } => {
            for e in elems { collect_names(e, out); }
        }
        PatternKind::Or(pats) => {
            for p in pats { collect_names(p, out); }
        }
        _ => {}
    }
}
