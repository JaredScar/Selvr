//! Hover information — type, targeting decision, and doc comments.

use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};
use selvr_lexer::Lexer;
use selvr_parser::Parser;
use selvr_parser::ast::{Item, Type};
use selvr_ir::lower_module;
use selvr_target::{infer_targets, propagate_targets, Target};
use crate::server::{position_to_offset, span_to_range};

pub fn hover(src: &str, pos: Position) -> Option<Hover> {
    // Locate the word under the cursor.
    let offset = position_to_offset(src, pos)?;
    let word   = word_at(src, offset)?;

    // Parse + analyse.
    let (tokens, _) = Lexer::new(src, 0).tokenize();
    let (module, _) = Parser::new(tokens, 0).parse();
    let mut ir  = lower_module(&module);
    let mut map = infer_targets(&mut ir);
    propagate_targets(&mut ir, &mut map);

    // Look for a matching top-level item.
    for item in &module.items {
        match item {
            Item::FnDef(f) if f.name.as_str() == word => {
                let mut md = format!("```selvr\nfn {}(", f.name);
                let params: Vec<String> = f.params.iter()
                    .map(|p| format!("{}: {}", p.name, type_str(&p.ty)))
                    .collect();
                md.push_str(&params.join(", "));
                md.push(')');
                if let Some(ret) = &f.return_ty {
                    md.push_str(&format!(": {}", type_str(ret)));
                }
                md.push_str("\n```\n\n");

                if let Some(rec) = map.fns.get(f.name.as_str()) {
                    let target_str = match rec.target {
                        Target::Wasm => "⚙ **WASM** — routes to WebAssembly",
                        Target::Js   => "⚡ **JS** — runs as plain JavaScript",
                        Target::Auto => "◌ **Auto** — target not yet determined",
                    };
                    md.push_str(&format!(
                        "{target_str}  \nCompiler score: `{}`  \nReason: {}\n",
                        rec.score, rec.reason
                    ));
                    if rec.forced {
                        md.push_str("\n> Targeting forced by attribute override.\n");
                    }
                }

                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind:  MarkupKind::Markdown,
                        value: md,
                    }),
                    range: Some(span_to_range(src, f.span)),
                });
            }

            Item::StructDef(s) if s.name.as_str() == word => {
                let mut md = format!("```selvr\nstruct {} {{\n", s.name);
                for field in &s.fields {
                    md.push_str(&format!("    {}: {},\n", field.name, type_str(&field.ty)));
                }
                md.push_str("}\n```");
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown, value: md,
                    }),
                    range: Some(span_to_range(src, s.span)),
                });
            }

            Item::EnumDef(e) if e.name.as_str() == word => {
                let md = format!("```selvr\nenum {}\n```", e.name);
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown, value: md,
                    }),
                    range: Some(span_to_range(src, e.span)),
                });
            }

            _ => {}
        }
    }

    // Keyword hover.
    if let Some(doc) = keyword_doc(word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind:  MarkupKind::Markdown,
                value: format!("**`{word}`** — {doc}"),
            }),
            range: None,
        });
    }

    None
}

fn type_str(ty: &Type) -> String {
    match ty {
        Type::Named { name, args, .. } if args.is_empty() => name.to_string(),
        Type::Named { name, args, .. } => {
            let inner: Vec<_> = args.iter().map(type_str).collect();
            format!("{}<{}>", name, inner.join(", "))
        }
        Type::Void(_)  => "void".into(),
        Type::Infer(_) => "_".into(),
        _ => "?".into(),
    }
}

fn word_at(src: &str, offset: usize) -> Option<&str> {
    let bytes  = src.as_bytes();
    let is_id  = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let start  = (0..=offset).rev().take_while(|&i| bytes.get(i).map_or(false, |&b| is_id(b))).last()?;
    let end    = (offset..src.len()).take_while(|&i| bytes.get(i).map_or(false, |&b| is_id(b))).last().map(|i| i+1)?;
    Some(&src[start..end])
}

fn keyword_doc(kw: &str) -> Option<&'static str> {
    match kw {
        "fn"       => Some("Define a function"),
        "const"    => Some("Declare an immutable binding"),
        "let"      => Some("Declare a mutable binding"),
        "return"   => Some("Return a value from the current function"),
        "if"       => Some("Conditional expression"),
        "while"    => Some("Loop while a condition holds"),
        "for"      => Some("Iterate over a range or iterable"),
        "match"    => Some("Exhaustive pattern matching"),
        "struct"   => Some("Define a named record type"),
        "enum"     => Some("Define a sum type with variants"),
        "trait"    => Some("Define a set of methods a type can implement"),
        "impl"     => Some("Implement a trait or add methods to a type"),
        "import"   => Some("Import symbols from another module"),
        "export"   => Some("Make an item visible to importers"),
        "async"    => Some("Mark a function as asynchronous"),
        "await"    => Some("Suspend until an async value resolves"),
        _ => None,
    }
}
