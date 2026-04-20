//! Completion item generation.
//!
//! Sources:
//!  1. Selvr keywords.
//!  2. Top-level function / type names from the current file's AST.
//!  3. Built-in types.
//!  4. Compiler attributes.

use lsp_types::{CompletionItem, CompletionItemKind, Position};
use selvr_lexer::Lexer;
use selvr_parser::Parser;
use selvr_parser::ast::Item;

/// Selvr keywords.
const KEYWORDS: &[(&str, &str)] = &[
    ("fn",       "Define a function"),
    ("const",    "Immutable binding"),
    ("let",      "Mutable binding"),
    ("return",   "Return from function"),
    ("if",       "Conditional"),
    ("else",     "Else branch"),
    ("while",    "Loop while condition is true"),
    ("for",      "Iterate over a range or iterator"),
    ("in",       "Used in for…in"),
    ("break",    "Exit a loop"),
    ("continue", "Skip to next iteration"),
    ("match",    "Pattern match"),
    ("struct",   "Define a struct"),
    ("enum",     "Define an enum"),
    ("trait",    "Define a trait"),
    ("impl",     "Implement a trait or add methods"),
    ("type",     "Type alias"),
    ("import",   "Import from a module"),
    ("export",   "Make an item public"),
    ("mod",      "Declare a sub-module"),
    ("async",    "Mark function as async"),
    ("await",    "Await an async expression"),
    ("true",     "Boolean true"),
    ("false",    "Boolean false"),
    ("null",     "Null value"),
    ("void",     "Void return type"),
    ("as",       "Type cast"),
];

/// Built-in primitive types.
const TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128",
    "u8", "u16", "u32", "u64", "u128",
    "f32", "f64",
    "bool", "string", "char", "void",
    "Option", "Result",
];

/// Compiler attributes.
const ATTRS: &[(&str, &str)] = &[
    ("#[wasm]",   "Force this function to run in WebAssembly"),
    ("#[js]",     "Force this function to run in JavaScript"),
    ("#[test]",   "Mark as a test function"),
    ("#[inline]", "Hint to inline this function"),
    ("#[gpu]",    "Route to WebGPU compute (stretch goal)"),
];

pub fn completions(src: &str, _pos: Position) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = Vec::new();

    // 1. Keywords
    for (kw, doc) in KEYWORDS {
        items.push(CompletionItem {
            label:  kw.to_string(),
            kind:   Some(CompletionItemKind::KEYWORD),
            detail: Some(doc.to_string()),
            ..Default::default()
        });
    }

    // 2. Built-in types
    for ty in TYPES {
        items.push(CompletionItem {
            label:  ty.to_string(),
            kind:   Some(CompletionItemKind::CLASS),
            detail: Some("built-in type".into()),
            ..Default::default()
        });
    }

    // 3. Attributes (only meaningful at the start of a line)
    for (attr, doc) in ATTRS {
        items.push(CompletionItem {
            label:  attr.to_string(),
            kind:   Some(CompletionItemKind::SNIPPET),
            detail: Some(doc.to_string()),
            ..Default::default()
        });
    }

    // 4. User-defined symbols from the AST
    let (tokens, _) = Lexer::new(src, 0).tokenize();
    let (module, _) = Parser::new(tokens, 0).parse();

    for item in &module.items {
        match item {
            Item::FnDef(f) => items.push(CompletionItem {
                label:  f.name.to_string(),
                kind:   Some(CompletionItemKind::FUNCTION),
                detail: Some(fn_signature(f)),
                ..Default::default()
            }),
            Item::StructDef(s) => items.push(CompletionItem {
                label:  s.name.to_string(),
                kind:   Some(CompletionItemKind::STRUCT),
                detail: Some("struct".into()),
                ..Default::default()
            }),
            Item::EnumDef(e) => items.push(CompletionItem {
                label:  e.name.to_string(),
                kind:   Some(CompletionItemKind::ENUM),
                detail: Some("enum".into()),
                ..Default::default()
            }),
            Item::TraitDef(t) => items.push(CompletionItem {
                label:  t.name.to_string(),
                kind:   Some(CompletionItemKind::INTERFACE),
                detail: Some("trait".into()),
                ..Default::default()
            }),
            Item::Const(c) => items.push(CompletionItem {
                label:  c.name.to_string(),
                kind:   Some(CompletionItemKind::CONSTANT),
                detail: Some("const".into()),
                ..Default::default()
            }),
            _ => {}
        }
    }

    items
}

fn fn_signature(f: &selvr_parser::ast::FnDef) -> String {
    let params: Vec<String> = f.params.iter()
        .map(|p| format!("{}: {}", p.name, type_str(&p.ty)))
        .collect();
    let ret = f.return_ty.as_ref().map(|t| format!(": {}", type_str(t))).unwrap_or_default();
    format!("fn({}){}", params.join(", "), ret)
}

fn type_str(ty: &selvr_parser::ast::Type) -> String {
    use selvr_parser::ast::Type;
    match ty {
        Type::Named { name, args, .. } if args.is_empty() => name.to_string(),
        Type::Named { name, args, .. } => {
            let inner: Vec<_> = args.iter().map(type_str).collect();
            format!("{}<{}>", name, inner.join(", "))
        }
        Type::Void(_)  => "void".into(),
        Type::Infer(_) => "_".into(),
        Type::Tuple { elems, .. } => {
            let inner: Vec<_> = elems.iter().map(type_str).collect();
            format!("({})", inner.join(", "))
        }
        _ => "?".into(),
    }
}
