//! Server capability declaration and main message loop.

use std::collections::HashMap;
use lsp_server::{Connection, Message, Request, Response, Notification};
use lsp_types::*;
use serde_json::Value;

type Url = lsp_types::Uri;

use crate::{diagnostics, completion, hover};

// ── Capability declaration ────────────────────────────────────────────────────

pub fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        // Sync: full document text on every change.
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::FULL,
        )),
        // Trigger completions on `_`, `.`, `:`, `(` in addition to alpha.
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![
                ".".into(), ":".into(), "(".into(), "_".into(),
            ]),
            resolve_provider: Some(false),
            ..Default::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        // Full-document formatting via `selvr fmt`.
        document_formatting_provider: Some(OneOf::Left(true)),
        // Definition jump (best-effort within the same file).
        definition_provider: Some(OneOf::Left(true)),
        // Document symbols for outline view.
        document_symbol_provider: Some(OneOf::Left(true)),
        // Code-action provider (quick fixes for lint diagnostics).
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        ..Default::default()
    }
}

// ── Server state ──────────────────────────────────────────────────────────────

struct ServerState {
    /// In-memory snapshot of every open document: URI → text.
    documents: HashMap<Url, String>,
}

impl ServerState {
    fn new() -> Self { Self { documents: HashMap::new() } }
}

// ── Main loop ─────────────────────────────────────────────────────────────────

pub fn run(
    connection: Connection,
    _init: lsp_types::InitializeParams,
) -> anyhow::Result<()> {
    let mut state = ServerState::new();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? { break; }
                handle_request(&connection, &mut state, req)?;
            }
            Message::Notification(n) => {
                handle_notification(&connection, &mut state, n)?;
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

// ── Request dispatch ──────────────────────────────────────────────────────────

fn handle_request(
    conn:  &Connection,
    state: &mut ServerState,
    req:   Request,
) -> anyhow::Result<()> {
    use lsp_types::request::*;
    let id = req.id.clone();

    let resp = match req.method.as_str() {
        Completion::METHOD => {
            let p: CompletionParams = serde_json::from_value(req.params)?;
            let uri = &p.text_document_position.text_document.uri;
            let pos = p.text_document_position.position;
            let items = if let Some(src) = state.documents.get(uri) {
                completion::completions(src, pos)
            } else { vec![] };
            let result = CompletionResponse::Array(items);
            ok_response(id, result)
        }

        HoverRequest::METHOD => {
            let p: HoverParams = serde_json::from_value(req.params)?;
            let uri = &p.text_document_position_params.text_document.uri;
            let pos = p.text_document_position_params.position;
            let result = if let Some(src) = state.documents.get(uri) {
                hover::hover(src, pos)
            } else { None };
            ok_response(id, result)
        }

        Formatting::METHOD => {
            let p: DocumentFormattingParams = serde_json::from_value(req.params)?;
            let uri = &p.text_document.uri;
            let edits = if let Some(src) = state.documents.get(uri) {
                format_document(src)
            } else { vec![] };
            ok_response(id, edits)
        }

        GotoDefinition::METHOD => {
            // Best-effort single-file definition.
            let p: GotoDefinitionParams = serde_json::from_value(req.params)?;
            let uri = &p.text_document_position_params.text_document.uri;
            let pos = p.text_document_position_params.position;
            let result: Option<GotoDefinitionResponse> = if let Some(src) = state.documents.get(uri) {
                find_definition(src, uri, pos)
            } else { None };
            ok_response(id, result)
        }

        DocumentSymbolRequest::METHOD => {
            let p: DocumentSymbolParams = serde_json::from_value(req.params)?;
            let uri = &p.text_document.uri;
            let symbols: Vec<DocumentSymbol> = if let Some(src) = state.documents.get(uri) {
                document_symbols(src)
            } else { vec![] };
            ok_response(id, DocumentSymbolResponse::Nested(symbols))
        }

        CodeActionRequest::METHOD => {
            ok_response(id, serde_json::json!(null))
        }

        _ => {
            Response::new_err(id, lsp_server::ErrorCode::MethodNotFound as i32, "method not found".into())
        }
    };

    conn.sender.send(Message::Response(resp))?;
    Ok(())
}

// ── Notification dispatch ─────────────────────────────────────────────────────

fn handle_notification(
    conn:  &Connection,
    state: &mut ServerState,
    notif: Notification,
) -> anyhow::Result<()> {
    use lsp_types::notification::*;

    match notif.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let p: DidOpenTextDocumentParams = serde_json::from_value(notif.params)?;
            let uri = p.text_document.uri.clone();
            let text = p.text_document.text.clone();
            publish_diagnostics(conn, &uri, &text)?;
            state.documents.insert(uri, text);
        }

        DidChangeTextDocument::METHOD => {
            let p: DidChangeTextDocumentParams = serde_json::from_value(notif.params)?;
            let uri = p.text_document.uri.clone();
            if let Some(change) = p.content_changes.into_iter().last() {
                let text = change.text;
                publish_diagnostics(conn, &uri, &text)?;
                state.documents.insert(uri, text);
            }
        }

        DidSaveTextDocument::METHOD => {
            let p: DidSaveTextDocumentParams = serde_json::from_value(notif.params)?;
            let uri = p.text_document.uri;
            if let Some(src) = state.documents.get(&uri) {
                publish_diagnostics(conn, &uri, src)?;
            }
        }

        DidCloseTextDocument::METHOD => {
            let p: DidCloseTextDocumentParams = serde_json::from_value(notif.params)?;
            state.documents.remove(&p.text_document.uri);
        }

        _ => {}
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn ok_response<T: serde::Serialize>(id: lsp_server::RequestId, value: T) -> Response {
    Response {
        id,
        result: Some(serde_json::to_value(value).unwrap_or(Value::Null)),
        error:  None,
    }
}

fn publish_diagnostics(conn: &Connection, uri: &Url, src: &str) -> anyhow::Result<()> {
    let diags = diagnostics::lint_diagnostics(src);
    let params = PublishDiagnosticsParams {
        uri:         uri.clone(),
        diagnostics: diags,
        version:     None,
    };
    let notif = Notification::new(
        <lsp_types::notification::PublishDiagnostics as lsp_types::notification::Notification>::METHOD.into(),
        params,
    );
    conn.sender.send(Message::Notification(notif))?;
    Ok(())
}

fn format_document(src: &str) -> Vec<TextEdit> {
    let fmt = selvr_fmt::Formatter::new();
    match fmt.format_src(src) {
        Ok(formatted) if formatted != src => {
            // Replace the entire document.
            let end_line = src.lines().count().saturating_sub(1) as u32;
            let end_col  = src.lines().last().map(|l| l.len()).unwrap_or(0) as u32;
            vec![TextEdit {
                range:    Range {
                    start: Position { line: 0, character: 0 },
                    end:   Position { line: end_line, character: end_col },
                },
                new_text: formatted,
            }]
        }
        _ => vec![],
    }
}

fn find_definition(src: &str, uri: &Url, pos: Position) -> Option<GotoDefinitionResponse> {
    use selvr_lexer::Lexer;
    use selvr_parser::Parser;
    use selvr_parser::ast::*;

    let (tokens, _) = Lexer::new(src, 0).tokenize();
    let (module, _) = Parser::new(tokens, 0).parse();

    // Find word at cursor position.
    let offset = position_to_offset(src, pos)?;
    let word   = word_at(src, offset)?;

    // Search module top-level for a function/struct with that name.
    for item in &module.items {
        let (name, span): (&str, selvr_lexer::span::Span) = match item {
            Item::FnDef(f)     => (f.name.as_str(), f.span),
            Item::StructDef(s) => (s.name.as_str(), s.span),
            Item::EnumDef(e)   => (e.name.as_str(), e.span),
            Item::TraitDef(t)  => (t.name.as_str(), t.span),
            _ => continue,
        };
        if name == word {
            let loc = Location {
                uri:   uri.clone(),
                range: span_to_range(src, span),
            };
            return Some(GotoDefinitionResponse::Scalar(loc));
        }
    }
    None
}

fn document_symbols(src: &str) -> Vec<DocumentSymbol> {
    use selvr_lexer::Lexer;
    use selvr_parser::Parser;
    use selvr_parser::ast::*;

    let (tokens, _) = Lexer::new(src, 0).tokenize();
    let (module, _) = Parser::new(tokens, 0).parse();

    module.items.iter().filter_map(|item| {
        let (name, kind, span): (String, SymbolKind, selvr_lexer::span::Span) = match item {
            Item::FnDef(f)     => (f.name.to_string(), SymbolKind::FUNCTION,  f.span),
            Item::StructDef(s) => (s.name.to_string(), SymbolKind::STRUCT,    s.span),
            Item::EnumDef(e)   => (e.name.to_string(), SymbolKind::ENUM,      e.span),
            Item::TraitDef(t)  => (t.name.to_string(), SymbolKind::INTERFACE, t.span),
            Item::Const(c)     => (c.name.to_string(), SymbolKind::CONSTANT,  c.span),
            _ => return None,
        };
        let range = span_to_range(src, span);
        #[allow(deprecated)]
        Some(DocumentSymbol {
            name,
            detail: None,
            kind,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: None,
        })
    }).collect()
}

// ── Span / position utilities ─────────────────────────────────────────────────

pub fn position_to_offset(src: &str, pos: Position) -> Option<usize> {
    let mut line = 0u32;
    let mut col  = 0u32;
    for (i, c) in src.char_indices() {
        if line == pos.line && col == pos.character { return Some(i); }
        if c == '\n' { line += 1; col = 0; } else { col += 1; }
    }
    None
}

pub fn span_to_range(src: &str, span: selvr_lexer::span::Span) -> Range {
    let (sl, sc) = offset_to_line_col(src, span.start as usize);
    let (el, ec) = offset_to_line_col(src, span.end   as usize);
    Range {
        start: Position { line: sl as u32, character: sc as u32 },
        end:   Position { line: el as u32, character: ec as u32 },
    }
}

fn offset_to_line_col(src: &str, offset: usize) -> (usize, usize) {
    let slice = &src[..offset.min(src.len())];
    let line  = slice.chars().filter(|&c| c == '\n').count();
    let col   = slice.rfind('\n').map_or(slice.len(), |p| slice.len() - p - 1);
    (line, col)
}

fn word_at(src: &str, offset: usize) -> Option<&str> {
    let bytes = src.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let start = (0..=offset).rev().take_while(|&i| bytes.get(i).map_or(false, |&b| is_ident(b))).last()?;
    let end   = (offset..src.len()).take_while(|&i| bytes.get(i).map_or(false, |&b| is_ident(b))).last().map(|i| i+1)?;
    Some(&src[start..end])
}
