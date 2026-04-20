//! Recursive-descent parser for Selvr.
//!
//! Key TypeScript-like syntax rules:
//!   - Return types:    `fn foo(x: i32): i32 { ... }`   (colon, not `->`)
//!   - Immutable:       `const x = 5;`
//!   - Mutable:         `let x = 5;`
//!   - Closures:        `(x: i32) => x * 2`
//!   - Template lits:   `` `Hello ${name}!` ``
//!   - Visibility:      `export fn` / `export const` / `export struct`
//!   - Imports:         `import { X, Y } from "module"`
//!   - `this` in impl:  implicit receiver — no `self` parameter
//!   - Strict equality: `===` / `!==`
//!   - Enum variants:   `Enum.Variant` (dot, not `::`)
//!   - Struct fields:   semicolon-terminated, not comma-separated
//!
//! Operator precedence (low → high):
//!   1. Assignment   (`=`, `+=`, …)
//!   2. Range        (`..`, `..=`)
//!   3. Or           (`||`)
//!   4. And          (`&&`)
//!   5. Equality     (`===`, `!==`)
//!   6. Comparison   (`<`, `>`, `<=`, `>=`)
//!   7. Additive     (`+`, `-`)
//!   8. Multiplicative (`*`, `/`, `%`)
//!   9. Unary        (`!`, `-`)
//!  10. Postfix      (call, field, index, await, as, ?)
//!  11. Primary      (literals, paths, blocks, closures, template lits)

use smol_str::SmolStr;
use selvr_lexer::{span::{Span, Spanned}, token::Token};
use crate::{
    ast::*,
    error::ParseError,
};

pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    pos: usize,
    errors: Vec<ParseError>,
    file_id: u32,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned<Token>>, file_id: u32) -> Self {
        let tokens: Vec<_> = tokens.into_iter().filter(|t| !t.node.is_trivia()).collect();
        Self { tokens, pos: 0, errors: Vec::new(), file_id }
    }

    pub fn parse(mut self) -> (Module, Vec<ParseError>) {
        let start = self.current_span();
        let mut items = Vec::new();
        while !self.at_eof() {
            match self.parse_item() {
                Some(item) => items.push(item),
                None => { self.advance(); }
            }
        }
        let span = start.merge(self.current_span());
        (Module { items, span }, self.errors)
    }

    // ── Token navigation ─────────────────────────────────────────────────────

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).map(|s| &s.node).unwrap_or(&Token::Eof)
    }

    fn peek_span(&self) -> Span {
        self.tokens.get(self.pos).map(|s| s.span).unwrap_or_else(|| {
            let last = self.tokens.last().map(|s| s.span.end).unwrap_or(0);
            Span::new(last, last, self.file_id)
        })
    }

    fn current_span(&self) -> Span { self.peek_span() }

    fn advance(&mut self) -> &Token {
        let tok = self.tokens.get(self.pos).map(|s| &s.node).unwrap_or(&Token::Eof);
        if self.pos < self.tokens.len() { self.pos += 1; }
        tok
    }

    fn at_eof(&self) -> bool { matches!(self.peek(), Token::Eof) }

    fn check(&self, tok: &Token) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(tok)
    }

    fn eat(&mut self, tok: &Token) -> bool {
        if self.check(tok) { self.advance(); true } else { false }
    }

    fn expect(&mut self, tok: &Token, label: &'static str) -> bool {
        if self.eat(tok) { true } else {
            let span = self.peek_span();
            self.errors.push(ParseError::expected(label, self.peek(), span));
            false
        }
    }

    fn expect_ident(&mut self) -> Option<SmolStr> {
        match self.peek().clone() {
            Token::Ident(name) => { self.advance(); Some(name) }
            _ => {
                let span = self.peek_span();
                self.errors.push(ParseError::expected("identifier", self.peek(), span));
                None
            }
        }
    }

    // `export` replaces `pub` as the visibility keyword.
    fn eat_visibility(&mut self) -> Visibility {
        if self.eat(&Token::Export) { Visibility::Public } else { Visibility::Private }
    }

    // ── Items ─────────────────────────────────────────────────────────────────

    fn parse_item(&mut self) -> Option<Item> {
        // Collect any leading `#[attr]` annotations before the item keyword.
        let attrs = self.parse_attrs();

        let vis = self.eat_visibility();

        // async fn
        if self.check(&Token::Async) && {
            // peek ahead — is the next token `fn`?
            self.tokens.get(self.pos + 1).map(|t| matches!(t.node, Token::Fn)).unwrap_or(false)
        } {
            self.advance(); // consume `async`
            return self.parse_fn_def(vis, true, attrs).map(Item::FnDef);
        }

        match self.peek() {
            Token::Fn     => self.parse_fn_def(vis, false, attrs).map(Item::FnDef),
            Token::Struct  => self.parse_struct_def(vis).map(Item::StructDef),
            Token::Enum    => self.parse_enum_def(vis).map(Item::EnumDef),
            Token::Trait   => self.parse_trait_def(vis).map(Item::TraitDef),
            Token::Impl    => self.parse_impl_block().map(Item::ImplBlock),
            Token::Type    => self.parse_type_alias(vis).map(Item::TypeAlias),
            Token::Import  => self.parse_import_decl().map(Item::ImportDecl),
            Token::Mod     => self.parse_mod_decl(vis).map(Item::ModDecl),
            Token::Const   => self.parse_const_item(vis).map(Item::Const),
            _ => {
                let span = self.peek_span();
                self.errors.push(ParseError::expected("item", self.peek(), span));
                None
            }
        }
    }

    // ── Attributes: #[name] or #[name(arg1, arg2)] ────────────────────────────

    fn parse_attrs(&mut self) -> Vec<Attr> {
        let mut attrs = Vec::new();
        while self.check(&Token::Hash) {
            let span_start = self.peek_span();
            self.advance(); // consume `#`
            if !self.eat(&Token::LBracket) { break; }
            let name = match self.expect_ident() {
                Some(n) => n,
                None    => {
                    while !matches!(self.peek(), Token::RBracket | Token::Eof) { self.advance(); }
                    self.eat(&Token::RBracket);
                    continue;
                }
            };
            // Optional `(arg, …)`
            let mut args = Vec::new();
            if self.eat(&Token::LParen) {
                while !matches!(self.peek(), Token::RParen | Token::Eof) {
                    // Collect tokens as a raw string until `)` or `,`
                    let mut raw = String::new();
                    while !matches!(self.peek(), Token::RParen | Token::Comma | Token::Eof) {
                        raw.push_str(&format!("{:?}", self.peek()));
                        self.advance();
                    }
                    args.push(SmolStr::new(raw));
                    if !self.eat(&Token::Comma) { break; }
                }
                self.expect(&Token::RParen, "`)`");
            }
            let span = span_start.merge(self.current_span());
            self.expect(&Token::RBracket, "`]`");
            attrs.push(Attr { name, args, span });
        }
        attrs
    }

    // ── fn ────────────────────────────────────────────────────────────────────

    fn parse_fn_def(&mut self, vis: Visibility, is_async: bool, attrs: Vec<Attr>) -> Option<FnDef> {
        let start = self.peek_span();
        self.expect(&Token::Fn, "`fn`");
        let name = self.expect_ident()?;
        let generics = self.parse_generic_params();
        let params = self.parse_fn_params();
        // Return type: `: Type`  (TypeScript style)
        let return_ty = if self.eat(&Token::Colon) { self.parse_type() } else { None };
        let body = self.parse_block()?;
        let span = start.merge(body.span);
        Some(FnDef { vis, name, generics, params, return_ty, body, is_async, attrs, span })
    }

    fn parse_fn_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        self.expect(&Token::LParen, "`(`");
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            let start = self.peek_span();
            let Some(name) = self.expect_ident() else { break };
            self.expect(&Token::Colon, "`:`");
            let Some(ty) = self.parse_type() else { break };
            let span = start.merge(ty.span());
            params.push(Param { name, ty, span });
            if !self.eat(&Token::Comma) { break; }
        }
        self.expect(&Token::RParen, "`)`");
        params
    }

    fn parse_generic_params(&mut self) -> Vec<GenericParam> {
        if !self.eat(&Token::Lt) { return Vec::new(); }
        let mut params = Vec::new();
        while !matches!(self.peek(), Token::Gt | Token::Eof) {
            let start = self.peek_span();
            let Some(name) = self.expect_ident() else { break };
            let bounds = if self.eat(&Token::Colon) {
                let mut bs = Vec::new();
                if let Some(t) = self.parse_type() { bs.push(t); }
                while self.eat(&Token::Amp) {
                    if let Some(t) = self.parse_type() { bs.push(t); }
                }
                bs
            } else { Vec::new() };
            let span = start.merge(self.current_span());
            params.push(GenericParam { name, bounds, span });
            if !self.eat(&Token::Comma) { break; }
        }
        self.expect(&Token::Gt, "`>`");
        params
    }

    // ── struct ────────────────────────────────────────────────────────────────

    fn parse_struct_def(&mut self, vis: Visibility) -> Option<StructDef> {
        let start = self.peek_span();
        self.expect(&Token::Struct, "`struct`");
        let name = self.expect_ident()?;
        let generics = self.parse_generic_params();
        let mut fields = Vec::new();
        self.expect(&Token::LBrace, "`{`");
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let fstart = self.peek_span();
            let fvis = self.eat_visibility();
            let Some(fname) = self.expect_ident() else { break };
            self.expect(&Token::Colon, "`:`");
            let Some(fty) = self.parse_type() else { break };
            // Struct fields are semicolon-terminated (TypeScript-like)
            self.eat(&Token::Semi);
            let fspan = fstart.merge(fty.span());
            fields.push(StructField { vis: fvis, name: fname, ty: fty, span: fspan });
        }
        self.expect(&Token::RBrace, "`}`");
        let span = start.merge(self.current_span());
        Some(StructDef { vis, name, generics, fields, span })
    }

    // ── enum ──────────────────────────────────────────────────────────────────

    fn parse_enum_def(&mut self, vis: Visibility) -> Option<EnumDef> {
        let start = self.peek_span();
        self.advance(); // `enum`
        let name = self.expect_ident()?;
        let generics = self.parse_generic_params();
        self.expect(&Token::LBrace, "`{`");
        let mut variants = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let vstart = self.peek_span();
            let Some(vname) = self.expect_ident() else { break };
            let kind = if self.eat(&Token::LParen) {
                let mut tys = Vec::new();
                while !matches!(self.peek(), Token::RParen | Token::Eof) {
                    if let Some(t) = self.parse_type() { tys.push(t); }
                    if !self.eat(&Token::Comma) { break; }
                }
                self.expect(&Token::RParen, "`)`");
                VariantKind::Tuple(tys)
            } else if self.check(&Token::LBrace) {
                self.advance();
                let mut fs = Vec::new();
                while !matches!(self.peek(), Token::RBrace | Token::Eof) {
                    let fstart2 = self.peek_span();
                    let fvis = self.eat_visibility();
                    let Some(fn_) = self.expect_ident() else { break };
                    self.expect(&Token::Colon, "`:`");
                    let Some(ft) = self.parse_type() else { break };
                    self.eat(&Token::Semi);
                    let fspan = fstart2.merge(ft.span());
                    fs.push(StructField { vis: fvis, name: fn_, ty: ft, span: fspan });
                }
                self.expect(&Token::RBrace, "`}`");
                VariantKind::Struct(fs)
            } else {
                VariantKind::Unit
            };
            let vspan = vstart.merge(self.current_span());
            variants.push(EnumVariant { name: vname, kind, span: vspan });
            if !self.eat(&Token::Comma) { break; }
        }
        self.expect(&Token::RBrace, "`}`");
        let span = start.merge(self.current_span());
        Some(EnumDef { vis, name, generics, variants, span })
    }

    // ── trait ─────────────────────────────────────────────────────────────────

    fn parse_trait_def(&mut self, vis: Visibility) -> Option<TraitDef> {
        let start = self.peek_span();
        self.advance();
        let name = self.expect_ident()?;
        let generics = self.parse_generic_params();
        self.expect(&Token::LBrace, "`{`");
        let mut items = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let is_async = if self.check(&Token::Async) {
                self.tokens.get(self.pos + 1).map(|t| matches!(t.node, Token::Fn)).unwrap_or(false)
            } else { false };
            if is_async { self.advance(); }
            if self.check(&Token::Fn) {
                let fvis = Visibility::Public;
                let inner_attrs = self.parse_attrs();
                if let Some(f) = self.parse_fn_def(fvis, is_async, inner_attrs) {
                    items.push(TraitItem::FnDef(f));
                }
            } else { self.advance(); }
        }
        self.expect(&Token::RBrace, "`}`");
        let span = start.merge(self.current_span());
        Some(TraitDef { vis, name, generics, items, span })
    }

    // ── impl ──────────────────────────────────────────────────────────────────

    fn parse_impl_block(&mut self) -> Option<ImplBlock> {
        let start = self.peek_span();
        self.advance(); // `impl`
        let generics = self.parse_generic_params();
        let self_ty = self.parse_type()?;
        self.expect(&Token::LBrace, "`{`");
        let mut items = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let vis = self.eat_visibility();
            let is_async = if self.check(&Token::Async) {
                self.tokens.get(self.pos + 1).map(|t| matches!(t.node, Token::Fn)).unwrap_or(false)
            } else { false };
            if is_async { self.advance(); }
            let impl_attrs = self.parse_attrs();
            if let Some(f) = self.parse_fn_def(vis, is_async, impl_attrs) {
                items.push(ImplItem::Fn(f));
            } else { self.advance(); }
        }
        self.expect(&Token::RBrace, "`}`");
        let span = start.merge(self.current_span());
        Some(ImplBlock { generics, trait_: None, self_ty, items, span })
    }

    // ── type alias ────────────────────────────────────────────────────────────

    fn parse_type_alias(&mut self, vis: Visibility) -> Option<TypeAlias> {
        let start = self.peek_span();
        self.advance();
        let name = self.expect_ident()?;
        let generics = self.parse_generic_params();
        self.expect(&Token::Eq, "`=`");
        let ty = self.parse_type()?;
        self.expect(&Token::Semi, "`;`");
        let span = start.merge(self.current_span());
        Some(TypeAlias { vis, name, generics, ty, span })
    }

    // ── import ────────────────────────────────────────────────────────────────

    fn parse_import_decl(&mut self) -> Option<ImportDecl> {
        let start = self.peek_span();
        self.advance(); // `import`

        let items = if self.eat(&Token::Star) {
            ImportItems::Glob
        } else {
            self.expect(&Token::LBrace, "`{`");
            let mut names = Vec::new();
            while !matches!(self.peek(), Token::RBrace | Token::Eof) {
                let nstart = self.peek_span();
                let Some(name) = self.expect_ident() else { break };
                let alias = if self.eat(&Token::As) { self.expect_ident() } else { None };
                let nspan = nstart.merge(self.current_span());
                names.push(ImportedName { name, alias, span: nspan });
                if !self.eat(&Token::Comma) { break; }
            }
            self.expect(&Token::RBrace, "`}`");
            ImportItems::Named(names)
        };

        // `from "module-path"`
        let source = if self.check(&Token::Ident(SmolStr::new("from"))) {
            self.advance(); // `from`
            match self.peek().clone() {
                Token::StrLit(s) => { self.advance(); s }
                _ => {
                    let span = self.peek_span();
                    self.errors.push(ParseError::expected("module path string", self.peek(), span));
                    SmolStr::new("")
                }
            }
        } else { SmolStr::new("") };

        self.expect(&Token::Semi, "`;`");
        let span = start.merge(self.current_span());
        Some(ImportDecl { items, source, span })
    }

    // ── mod ───────────────────────────────────────────────────────────────────

    fn parse_mod_decl(&mut self, vis: Visibility) -> Option<ModDecl> {
        let start = self.peek_span();
        self.advance();
        let name = self.expect_ident()?;
        let body = if self.check(&Token::LBrace) {
            self.advance();
            let mut items = Vec::new();
            while !matches!(self.peek(), Token::RBrace | Token::Eof) {
                if let Some(item) = self.parse_item() { items.push(item); }
                else { self.advance(); }
            }
            self.expect(&Token::RBrace, "`}`");
            let span = start.merge(self.current_span());
            Some(Module { items, span })
        } else {
            self.expect(&Token::Semi, "`;`");
            None
        };
        let span = start.merge(self.current_span());
        Some(ModDecl { vis, name, body, span })
    }

    // ── const ─────────────────────────────────────────────────────────────────

    fn parse_const_item(&mut self, vis: Visibility) -> Option<ConstItem> {
        let start = self.peek_span();
        self.advance(); // `const`
        let name = self.expect_ident()?;
        let ty = if self.eat(&Token::Colon) { self.parse_type() } else { None };
        self.expect(&Token::Eq, "`=`");
        let value = self.parse_expr()?;
        self.expect(&Token::Semi, "`;`");
        let span = start.merge(self.current_span());
        Some(ConstItem { vis, name, ty, value, span })
    }

    // ── Types ─────────────────────────────────────────────────────────────────

    fn parse_type(&mut self) -> Option<Type> {
        let start = self.peek_span();
        let base = self.parse_base_type()?;

        // Postfix `[]` — `i32[]`, `string[]`, etc.
        if self.eat(&Token::LBracket) {
            self.expect(&Token::RBracket, "`]`");
            let span = start.merge(self.current_span());
            return Some(Type::Array { elem: Box::new(base), len: None, span });
        }

        Some(base)
    }

    fn parse_base_type(&mut self) -> Option<Type> {
        let start = self.peek_span();
        match self.peek().clone() {
            Token::KwI32     => { self.advance(); Some(Type::Named { name: SmolStr::new("i32"),     args: vec![], span: start }) }
            Token::KwI64     => { self.advance(); Some(Type::Named { name: SmolStr::new("i64"),     args: vec![], span: start }) }
            Token::KwF32     => { self.advance(); Some(Type::Named { name: SmolStr::new("f32"),     args: vec![], span: start }) }
            Token::KwF64     => { self.advance(); Some(Type::Named { name: SmolStr::new("f64"),     args: vec![], span: start }) }
            Token::KwBoolean => { self.advance(); Some(Type::Named { name: SmolStr::new("boolean"), args: vec![], span: start }) }
            Token::KwString  => { self.advance(); Some(Type::Named { name: SmolStr::new("string"),  args: vec![], span: start }) }
            Token::KwChar    => { self.advance(); Some(Type::Named { name: SmolStr::new("char"),    args: vec![], span: start }) }
            Token::KwVoid    => { self.advance(); Some(Type::Void(start)) }
            Token::KwNumber  => { self.advance(); Some(Type::Named { name: SmolStr::new("f64"),     args: vec![], span: start }) }
            Token::Ident(name) => {
                self.advance();
                let args = if self.eat(&Token::Lt) {
                    let mut args = Vec::new();
                    while !matches!(self.peek(), Token::Gt | Token::Eof) {
                        if let Some(t) = self.parse_type() { args.push(t); }
                        if !self.eat(&Token::Comma) { break; }
                    }
                    self.expect(&Token::Gt, "`>`");
                    args
                } else { Vec::new() };
                let span = start.merge(self.current_span());
                Some(Type::Named { name, args, span })
            }
            Token::LParen => {
                // `(T, U)` tuple  OR  `(T) => R` function type
                self.advance();
                let mut elems = Vec::new();
                while !matches!(self.peek(), Token::RParen | Token::Eof) {
                    if let Some(t) = self.parse_type() { elems.push(t); }
                    if !self.eat(&Token::Comma) { break; }
                }
                self.expect(&Token::RParen, "`)`");
                if self.eat(&Token::FatArrow) {
                    let ret = self.parse_type().unwrap_or(Type::Void(start));
                    let span = start.merge(ret.span());
                    Some(Type::Fn { params: elems, ret: Box::new(ret), span })
                } else {
                    let span = start.merge(self.current_span());
                    Some(Type::Tuple { elems, span })
                }
            }
            Token::LBracket => {
                // `[T, U]` fixed-element tuple array or `[T; N]` fixed-size array
                self.advance();
                let mut elems = Vec::new();
                while !matches!(self.peek(), Token::RBracket | Token::Eof) {
                    if let Some(t) = self.parse_type() { elems.push(t); }
                    if !self.eat(&Token::Comma) { break; }
                }
                self.expect(&Token::RBracket, "`]`");
                let span = start.merge(self.current_span());
                Some(Type::Tuple { elems, span })
            }
            _ => {
                self.errors.push(ParseError::expected("type", self.peek(), start));
                None
            }
        }
    }

    // ── Blocks & Statements ───────────────────────────────────────────────────

    fn parse_block(&mut self) -> Option<Block> {
        let start = self.peek_span();
        self.expect(&Token::LBrace, "`{`");
        let mut stmts = Vec::new();
        let mut tail = None;
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let tail_before = tail.is_some();
            if let Some(stmt) = self.parse_stmt(&mut tail) {
                stmts.push(stmt);
            } else if !tail.is_some() || tail_before {
                // Genuine parse failure (tail was not freshly set) — advance to recover.
                self.advance();
            }
            // If tail was just set, don't advance — the `}` will end the loop.
        }
        self.expect(&Token::RBrace, "`}`");
        let span = start.merge(self.current_span());
        Some(Block { stmts, tail, span })
    }

    fn parse_stmt(&mut self, tail: &mut Option<Box<Expr>>) -> Option<Stmt> {
        match self.peek() {
            Token::Let => {
                let start = self.peek_span();
                self.advance();
                let pattern = self.parse_pattern()?;
                let ty = if self.eat(&Token::Colon) { self.parse_type() } else { None };
                let init = if self.eat(&Token::Eq) { self.parse_expr() } else { None };
                self.expect(&Token::Semi, "`;`");
                let span = start.merge(self.current_span());
                Some(Stmt::Let(LetStmt { pattern, ty, init, span }))
            }
            Token::Const => {
                // `const x: T = expr;` inside a block → ConstItem
                let start = self.peek_span();
                self.advance();
                let name = self.expect_ident()?;
                let ty = if self.eat(&Token::Colon) { self.parse_type() } else { None };
                self.expect(&Token::Eq, "`=`");
                let value = self.parse_expr()?;
                self.expect(&Token::Semi, "`;`");
                let span = start.merge(self.current_span());
                Some(Stmt::Item(Item::Const(ConstItem {
                    vis: Visibility::Private, name, ty, value, span,
                })))
            }
            Token::Fn | Token::Struct | Token::Enum | Token::Import => {
                self.parse_item().map(Stmt::Item)
            }
            _ => {
                let expr = self.parse_expr()?;
                // Block-ending expressions (while/for/if/loop) are always
                // treated as void statements; they never need a trailing `;`.
                let is_block_expr = matches!(
                    expr.kind,
                    ExprKind::While { .. }
                    | ExprKind::For { .. }
                    | ExprKind::Loop(_)
                    | ExprKind::If { .. }
                );
                let has_semi = self.eat(&Token::Semi); // optional `;`
                if is_block_expr || has_semi {
                    let span = expr.span;
                    Some(Stmt::Expr(expr, span))
                } else {
                    // No `;` and not a block expression — this is the tail.
                    *tail = Some(Box::new(expr));
                    None
                }
            }
        }
    }

    // ── Patterns ──────────────────────────────────────────────────────────────

    fn parse_pattern(&mut self) -> Option<Pattern> {
        let start = self.peek_span();
        let kind = match self.peek().clone() {
            Token::Ident(name) if name == "_" => { self.advance(); PatternKind::Wildcard }
            Token::Ident(name) => {
                self.advance();
                PatternKind::Ident { name, mutable: false }
            }
            Token::IntLit(v)  => { let v = v; self.advance(); PatternKind::Literal(LitPat::Int(v)) }
            Token::BoolLit(v) => { let v = v; self.advance(); PatternKind::Literal(LitPat::Bool(v)) }
            Token::StrLit(v)  => { let v = v; self.advance(); PatternKind::Literal(LitPat::Str(v)) }
            Token::LBracket   => {
                // `[a, b]` — tuple pattern (TypeScript destructuring style)
                self.advance();
                let mut elems = Vec::new();
                while !matches!(self.peek(), Token::RBracket | Token::Eof) {
                    if let Some(p) = self.parse_pattern() { elems.push(p); }
                    if !self.eat(&Token::Comma) { break; }
                }
                self.expect(&Token::RBracket, "`]`");
                PatternKind::Tuple(elems)
            }
            Token::LParen => {
                self.advance();
                let mut elems = Vec::new();
                while !matches!(self.peek(), Token::RParen | Token::Eof) {
                    if let Some(p) = self.parse_pattern() { elems.push(p); }
                    if !self.eat(&Token::Comma) { break; }
                }
                self.expect(&Token::RParen, "`)`");
                PatternKind::Tuple(elems)
            }
            _ => {
                let span = self.peek_span();
                self.errors.push(ParseError::expected("pattern", self.peek(), span));
                return None;
            }
        };
        let span = start.merge(self.current_span());
        Some(Pattern { kind, span })
    }

    // ── Expressions ───────────────────────────────────────────────────────────

    fn parse_expr(&mut self) -> Option<Expr> { self.parse_assign() }

    fn parse_assign(&mut self) -> Option<Expr> {
        let lhs = self.parse_range()?;
        let start = lhs.span;
        if self.eat(&Token::Eq) {
            let rhs = self.parse_assign()?;
            let span = start.merge(rhs.span);
            return Some(Expr { kind: ExprKind::Assign { target: Box::new(lhs), rhs: Box::new(rhs) }, span });
        }
        let op = match self.peek() {
            Token::PlusEq    => Some(BinaryOp::Add),
            Token::MinusEq   => Some(BinaryOp::Sub),
            Token::StarEq    => Some(BinaryOp::Mul),
            Token::SlashEq   => Some(BinaryOp::Div),
            Token::PercentEq => Some(BinaryOp::Rem),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let rhs = self.parse_assign()?;
            let span = start.merge(rhs.span);
            return Some(Expr {
                kind: ExprKind::CompoundAssign { op, target: Box::new(lhs), rhs: Box::new(rhs) },
                span,
            });
        }
        Some(lhs)
    }

    fn parse_range(&mut self) -> Option<Expr> {
        let lhs = self.parse_or()?;
        let start = lhs.span;
        if self.check(&Token::DotDot) || self.check(&Token::DotDotEq) {
            let inclusive = matches!(self.peek(), Token::DotDotEq);
            self.advance();
            let end = self.parse_or();
            let span = start.merge(end.as_ref().map(|e| e.span).unwrap_or(start));
            return Some(Expr {
                kind: ExprKind::Range { start: Some(Box::new(lhs)), end: end.map(Box::new), inclusive },
                span,
            });
        }
        Some(lhs)
    }

    fn parse_or(&mut self) -> Option<Expr> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek(), Token::PipePipe) {
            self.advance();
            let rhs = self.parse_and()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr { kind: ExprKind::Binary { op: BinaryOp::Or, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span };
        }
        Some(lhs)
    }

    fn parse_and(&mut self) -> Option<Expr> {
        let mut lhs = self.parse_equality()?;
        while matches!(self.peek(), Token::AmpAmp) {
            self.advance();
            let rhs = self.parse_equality()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr { kind: ExprKind::Binary { op: BinaryOp::And, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span };
        }
        Some(lhs)
    }

    fn parse_equality(&mut self) -> Option<Expr> {
        let mut lhs = self.parse_comparison()?;
        loop {
            // Support both strict `===` / `!==` and loose `==` / `!=`
            let op = match self.peek() {
                Token::EqEq | Token::EqEqLoose     => BinaryOp::Eq,
                Token::BangEq | Token::BangEqLoose  => BinaryOp::Ne,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_comparison()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr { kind: ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span };
        }
        Some(lhs)
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut lhs = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Token::Lt   => BinaryOp::Lt,
                Token::Gt   => BinaryOp::Gt,
                Token::LtEq => BinaryOp::Le,
                Token::GtEq => BinaryOp::Ge,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_additive()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr { kind: ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span };
        }
        Some(lhs)
    }

    fn parse_additive(&mut self) -> Option<Expr> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus  => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_multiplicative()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr { kind: ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span };
        }
        Some(lhs)
    }

    fn parse_multiplicative(&mut self) -> Option<Expr> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star    => BinaryOp::Mul,
                Token::Slash   => BinaryOp::Div,
                Token::Percent => BinaryOp::Rem,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_unary()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr { kind: ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span };
        }
        Some(lhs)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        let start = self.peek_span();
        let op = match self.peek() {
            Token::Bang  => Some(UnaryOp::Not),
            Token::Minus => Some(UnaryOp::Neg),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let expr = self.parse_unary()?;
            let span = start.merge(expr.span);
            return Some(Expr { kind: ExprKind::Unary { op, expr: Box::new(expr) }, span });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            expr = match self.peek() {
                Token::LParen => {
                    self.advance();
                    let args = self.parse_call_args();
                    let span = expr.span.merge(self.current_span());
                    Expr { kind: ExprKind::Call { callee: Box::new(expr), args }, span }
                }
                Token::Dot => {
                    self.advance();
                    if let Token::Ident(field) = self.peek().clone() {
                        self.advance();
                        let span = expr.span.merge(self.current_span());
                        if self.check(&Token::LParen) {
                            self.advance();
                            let args = self.parse_call_args();
                            let s = expr.span.merge(self.current_span());
                            Expr { kind: ExprKind::MethodCall { receiver: Box::new(expr), method: field, args }, span: s }
                        } else {
                            Expr { kind: ExprKind::Field { base: Box::new(expr), field }, span }
                        }
                    } else { break; }
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(&Token::RBracket, "`]`");
                    let span = expr.span.merge(self.current_span());
                    Expr { kind: ExprKind::Index { base: Box::new(expr), index: Box::new(index) }, span }
                }
                Token::Await => {
                    self.advance();
                    let span = expr.span.merge(self.current_span());
                    Expr { kind: ExprKind::Await(Box::new(expr)), span }
                }
                Token::As => {
                    self.advance();
                    let ty = self.parse_type().unwrap_or(Type::Void(self.current_span()));
                    let span = expr.span.merge(ty.span());
                    Expr { kind: ExprKind::Cast { expr: Box::new(expr), ty }, span }
                }
                Token::Question => {
                    self.advance();
                    let span = expr.span.merge(self.current_span());
                    // `?` desugars to early-return on Err/None
                    Expr { kind: ExprKind::Unary { op: UnaryOp::Deref, expr: Box::new(expr) }, span }
                }
                _ => break,
            };
        }
        Some(expr)
    }

    fn parse_call_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            if let Some(e) = self.parse_expr() { args.push(e); }
            if !self.eat(&Token::Comma) { break; }
        }
        self.expect(&Token::RParen, "`)`");
        args
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        let start = self.peek_span();
        match self.peek().clone() {
            // ── Literals ──────────────────────────────────────────────────────
            Token::IntLit(v)  => { self.advance(); Some(Expr { kind: ExprKind::IntLit(v), span: start }) }
            Token::FloatLit(v)=> { self.advance(); Some(Expr { kind: ExprKind::FloatLit(v), span: start }) }
            Token::BoolLit(v) => { self.advance(); Some(Expr { kind: ExprKind::BoolLit(v), span: start }) }
            Token::StrLit(v)  => { self.advance(); Some(Expr { kind: ExprKind::StrLit(v), span: start }) }
            Token::CharLit(v) => { self.advance(); Some(Expr { kind: ExprKind::CharLit(v), span: start }) }

            // ── Template literal: `Hello ${name}!` ────────────────────────────
            Token::TemplateLitStart(_) | Token::TemplateLitEnd(_) => {
                self.parse_template_literal(start)
            }

            // ── `this` keyword ────────────────────────────────────────────────
            Token::This => {
                self.advance();
                let path = Path {
                    segments: vec![PathSegment { name: SmolStr::new("this"), args: vec![], span: start }],
                    span: start,
                };
                Some(Expr { kind: ExprKind::Path(path), span: start })
            }

            // ── Parenthesised expression or closure `(x) => expr` ────────────
            Token::LParen => {
                // Decide: is this a closure `(params) => body` or a parenthesised expr?
                // Heuristic: look ahead for the `=>` token after a matching `)`.
                if self.is_closure_start() {
                    return self.parse_closure(start);
                }
                self.advance();
                if self.check(&Token::RParen) {
                    self.advance();
                    let span = start.merge(self.current_span());
                    // Check for `() => expr`
                    if self.eat(&Token::FatArrow) {
                        return self.parse_closure_body(vec![], None, start);
                    }
                    return Some(Expr { kind: ExprKind::TupleLit(vec![]), span });
                }
                let first = self.parse_expr()?;
                if self.eat(&Token::Comma) {
                    let mut elems = vec![first];
                    while !matches!(self.peek(), Token::RParen | Token::Eof) {
                        if let Some(e) = self.parse_expr() { elems.push(e); }
                        if !self.eat(&Token::Comma) { break; }
                    }
                    self.expect(&Token::RParen, "`)`");
                    let span = start.merge(self.current_span());
                    Some(Expr { kind: ExprKind::TupleLit(elems), span })
                } else {
                    self.expect(&Token::RParen, "`)`");
                    Some(first) // parenthesised expression
                }
            }

            Token::LBracket => {
                self.advance();
                let mut elems = Vec::new();
                while !matches!(self.peek(), Token::RBracket | Token::Eof) {
                    if let Some(e) = self.parse_expr() { elems.push(e); }
                    if !self.eat(&Token::Comma) { break; }
                }
                self.expect(&Token::RBracket, "`]`");
                let span = start.merge(self.current_span());
                Some(Expr { kind: ExprKind::ArrayLit(elems), span })
            }

            Token::LBrace => {
                let block = self.parse_block()?;
                let span = block.span;
                Some(Expr { kind: ExprKind::Block(block), span })
            }

            Token::If     => self.parse_if(),
            Token::Match  => self.parse_match(),
            Token::While  => self.parse_while(),
            Token::Loop   => {
                self.advance();
                let body = self.parse_block()?;
                let span = start.merge(body.span);
                Some(Expr { kind: ExprKind::Loop(body), span })
            }
            Token::For    => self.parse_for(),
            Token::Return => {
                self.advance();
                let value = if !matches!(self.peek(), Token::Semi | Token::RBrace | Token::Eof) {
                    self.parse_expr().map(Box::new)
                } else { None };
                let span = start.merge(self.current_span());
                Some(Expr { kind: ExprKind::Return { value }, span })
            }
            Token::Break => {
                self.advance();
                let value = if !matches!(self.peek(), Token::Semi | Token::RBrace | Token::Eof) {
                    self.parse_expr().map(Box::new)
                } else { None };
                let span = start.merge(self.current_span());
                Some(Expr { kind: ExprKind::Break { value }, span })
            }
            Token::Continue => {
                self.advance();
                Some(Expr { kind: ExprKind::Continue, span: start })
            }

            Token::Async => {
                // `async (_) => { ... }` — async closure
                if self.tokens.get(self.pos + 1)
                    .map(|t| matches!(t.node, Token::LParen | Token::FatArrow))
                    .unwrap_or(false)
                {
                    self.advance();
                    if self.is_closure_start() {
                        return self.parse_closure(start);
                    }
                }
                let span = self.peek_span();
                self.errors.push(ParseError::expected("expression", self.peek(), span));
                None
            }

            Token::Ident(name) => {
                self.advance();
                // Path: `Foo.Bar` or `Foo::Bar` — dot notation for enum variants (TypeScript-like)
                let mut segments = vec![PathSegment { name, args: vec![], span: start }];
                while self.eat(&Token::Dot) || self.eat(&Token::ColonColon) {
                    if let Token::Ident(seg) = self.peek().clone() {
                        let s = self.peek_span();
                        self.advance();
                        segments.push(PathSegment { name: seg, args: vec![], span: s });
                    } else { break; }
                }
                let span = start.merge(self.current_span());
                let path = Path { segments, span };
                Some(Expr { kind: ExprKind::Path(path), span })
            }

            _ => {
                let span = self.peek_span();
                self.errors.push(ParseError::expected("expression", self.peek(), span));
                None
            }
        }
    }

    // ── Template literal parsing ──────────────────────────────────────────────

    fn parse_template_literal(&mut self, start: Span) -> Option<Expr> {
        let mut parts: Vec<TemplatePart> = Vec::new();

        loop {
            match self.peek().clone() {
                Token::TemplateLitStart(text) => {
                    self.advance();
                    parts.push(TemplatePart::Str(text));
                    // Next comes a `${` — parse the interpolated expression up to `}`
                    let expr = self.parse_expr()?;
                    parts.push(TemplatePart::Expr(Box::new(expr)));
                    self.expect(&Token::RBrace, "`}`");
                }
                Token::TemplateLitMid(text) => {
                    self.advance();
                    parts.push(TemplatePart::Str(text));
                    let expr = self.parse_expr()?;
                    parts.push(TemplatePart::Expr(Box::new(expr)));
                    self.expect(&Token::RBrace, "`}`");
                }
                Token::TemplateLitEnd(text) => {
                    self.advance();
                    parts.push(TemplatePart::Str(text));
                    break;
                }
                _ => break,
            }
        }

        let span = start.merge(self.current_span());
        Some(Expr { kind: ExprKind::TemplateLit { parts }, span })
    }

    // ── Closure parsing `(x: i32, y: i32): R => expr` ────────────────────────

    /// Heuristic: peek ahead to determine if `(` starts a closure.
    /// A closure has `(params) =>` or `(params): RetType =>`.
    fn is_closure_start(&self) -> bool {
        let mut depth = 0usize;
        let mut i = self.pos;
        // Skip `(`
        if !matches!(self.tokens.get(i).map(|t| &t.node), Some(Token::LParen)) { return false; }
        i += 1;
        // Scan until matching `)` at depth 0
        loop {
            match self.tokens.get(i).map(|t| &t.node) {
                None | Some(Token::Eof) => return false,
                Some(Token::LParen) => { depth += 1; i += 1; }
                Some(Token::RParen) if depth > 0 => { depth -= 1; i += 1; }
                Some(Token::RParen) => {
                    i += 1; // consume `)`
                    // After `)`, might see `: RetType` then `=>`
                    if matches!(self.tokens.get(i).map(|t| &t.node), Some(Token::Colon)) { i += 1; }
                    // Skip type tokens until `=>`
                    let mut j = i;
                    while j < self.tokens.len() {
                        match self.tokens.get(j).map(|t| &t.node) {
                            Some(Token::FatArrow) => return true,
                            Some(Token::LBrace | Token::Eof) => return false,
                            _ => j += 1,
                        }
                    }
                    return false;
                }
                _ => { i += 1; }
            }
        }
    }

    fn parse_closure(&mut self, start: Span) -> Option<Expr> {
        self.advance(); // `(`
        let mut params: Vec<ClosureParam> = Vec::new();
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            let pstart = self.peek_span();
            let pat = self.parse_pattern()?;
            let ty = if self.eat(&Token::Colon) { self.parse_type() } else { None };
            let pspan = pstart.merge(self.current_span());
            params.push(ClosureParam { pat, ty, span: pspan });
            if !self.eat(&Token::Comma) { break; }
        }
        self.expect(&Token::RParen, "`)`");
        // Optional return type annotation: `: RetType`
        let ret = if self.eat(&Token::Colon) { self.parse_type() } else { None };
        self.expect(&Token::FatArrow, "`=>`");
        self.parse_closure_body(params, ret, start)
    }

    fn parse_closure_body(
        &mut self,
        params: Vec<ClosureParam>,
        ret: Option<Type>,
        start: Span,
    ) -> Option<Expr> {
        let body = if self.check(&Token::LBrace) {
            let block = self.parse_block()?;
            let span = block.span;
            Box::new(Expr { kind: ExprKind::Block(block), span })
        } else {
            Box::new(self.parse_expr()?)
        };
        let span = start.merge(body.span);
        Some(Expr { kind: ExprKind::Closure { params, ret, body }, span })
    }

    // ── Control flow expressions ──────────────────────────────────────────────

    fn parse_if(&mut self) -> Option<Expr> {
        let start = self.peek_span();
        self.advance(); // `if`
        let cond = self.parse_expr()?;
        let then = self.parse_block()?;
        let else_ = if self.eat(&Token::Else) {
            if self.check(&Token::If) {
                self.parse_if().map(|e| Box::new(e))
            } else {
                let block = self.parse_block()?;
                let span = block.span;
                Some(Box::new(Expr { kind: ExprKind::Block(block), span }))
            }
        } else { None };
        let span = start.merge(self.current_span());
        Some(Expr { kind: ExprKind::If { cond: Box::new(cond), then, else_ }, span })
    }

    fn parse_match(&mut self) -> Option<Expr> {
        let start = self.peek_span();
        self.advance(); // `match`
        let scrutinee = self.parse_expr()?;
        self.expect(&Token::LBrace, "`{`");
        let mut arms = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let astart = self.peek_span();
            let pattern = self.parse_pattern()?;
            // Guard: `if condition`
            let guard = if self.eat(&Token::If) {
                self.parse_expr().map(Box::new)
            } else { None };
            self.expect(&Token::FatArrow, "`=>`");
            let body = self.parse_expr()?;
            self.eat(&Token::Comma);
            let span = astart.merge(body.span);
            arms.push(MatchArm { pattern, guard, body: Box::new(body), span });
        }
        self.expect(&Token::RBrace, "`}`");
        let span = start.merge(self.current_span());
        Some(Expr { kind: ExprKind::Match { scrutinee: Box::new(scrutinee), arms }, span })
    }

    fn parse_while(&mut self) -> Option<Expr> {
        let start = self.peek_span();
        self.advance();
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        let span = start.merge(body.span);
        Some(Expr { kind: ExprKind::While { cond: Box::new(cond), body }, span })
    }

    fn parse_for(&mut self) -> Option<Expr> {
        let start = self.peek_span();
        self.advance(); // `for`
        let pat = self.parse_pattern()?;
        self.expect(&Token::In, "`in`");
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        let span = start.merge(body.span);
        Some(Expr { kind: ExprKind::For { pat, iter: Box::new(iter), body }, span })
    }
}
