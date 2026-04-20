//! AST → canonical Selvr source printer.
//!
//! The actual AST has `Stmt::{Let, Expr, Item}` only.
//! Control flow (if/while/for/return/break/continue) are `ExprKind` variants.

use selvr_parser::ast::*;

pub struct Printer {
    buf:    String,
    depth:  usize,
    indent: usize,
}

impl Printer {
    pub fn new(indent: usize) -> Self {
        Self { buf: String::with_capacity(1024), depth: 0, indent }
    }

    pub fn finish(mut self) -> String {
        if !self.buf.ends_with('\n') { self.buf.push('\n'); }
        self.buf
    }

    fn ind(&mut self) {
        let n = self.depth * self.indent;
        for _ in 0..n { self.buf.push(' '); }
    }

    fn push(&mut self) { self.depth += 1; }
    fn pop(&mut self)  { self.depth = self.depth.saturating_sub(1); }

    // ── Module ────────────────────────────────────────────────────────────────

    pub fn print_module(&mut self, m: &Module) {
        for (i, item) in m.items.iter().enumerate() {
            if i > 0 { self.buf.push_str("\n\n"); }
            self.print_item(item);
        }
    }

    // ── Items ─────────────────────────────────────────────────────────────────

    fn print_item(&mut self, item: &Item) {
        match item {
            Item::FnDef(f)      => self.print_fn(f),
            Item::StructDef(s)  => self.print_struct(s),
            Item::EnumDef(e)    => self.print_enum(e),
            Item::TraitDef(t)   => self.print_trait(t),
            Item::ImplBlock(i)  => self.print_impl(i),
            Item::TypeAlias(a)  => self.print_type_alias(a),
            Item::ImportDecl(d) => self.print_import(d),
            Item::ModDecl(md)   => self.print_mod(md),
            Item::Const(c)      => self.print_const_item(c),
            Item::MacroDef(_)   => {}
        }
    }

    // ── Attributes ────────────────────────────────────────────────────────────

    fn print_attrs(&mut self, attrs: &[Attr]) {
        for a in attrs {
            self.ind();
            self.buf.push_str("#[");
            self.buf.push_str(&a.name);
            if !a.args.is_empty() {
                self.buf.push('(');
                for (i, arg) in a.args.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(arg);
                }
                self.buf.push(')');
            }
            self.buf.push_str("]\n");
        }
    }

    // ── Functions ─────────────────────────────────────────────────────────────

    fn print_fn(&mut self, f: &FnDef) {
        self.print_attrs(&f.attrs);
        self.ind();
        if f.vis == Visibility::Public { self.buf.push_str("export "); }
        if f.is_async { self.buf.push_str("async "); }
        self.buf.push_str("fn ");
        self.buf.push_str(&f.name);
        self.print_generics(&f.generics);
        self.buf.push('(');
        for (i, p) in f.params.iter().enumerate() {
            if i > 0 { self.buf.push_str(", "); }
            self.buf.push_str(&p.name);
            self.buf.push_str(": ");
            self.print_type(&p.ty);
        }
        self.buf.push(')');
        if let Some(ret) = &f.return_ty {
            self.buf.push_str(": ");
            self.print_type(ret);
        }
        self.buf.push(' ');
        self.print_block(&f.body);
        self.buf.push('\n');
    }

    fn print_block(&mut self, b: &Block) {
        self.buf.push_str("{\n");
        self.push();
        for stmt in &b.stmts {
            self.print_stmt(stmt);
        }
        if let Some(tail) = &b.tail {
            self.ind();
            self.print_expr(tail);
            self.buf.push('\n');
        }
        self.pop();
        self.ind();
        self.buf.push('}');
    }

    // ── Types ─────────────────────────────────────────────────────────────────

    fn print_type(&mut self, ty: &Type) {
        match ty {
            Type::Named { name, args, .. } => {
                self.buf.push_str(name);
                if !args.is_empty() {
                    self.buf.push('<');
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 { self.buf.push_str(", "); }
                        self.print_type(a);
                    }
                    self.buf.push('>');
                }
            }
            Type::Tuple { elems, .. } => {
                self.buf.push('(');
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.print_type(e);
                }
                self.buf.push(')');
            }
            Type::Array { elem, len, .. } => {
                self.buf.push('[');
                self.print_type(elem);
                if let Some(l) = len {
                    self.buf.push_str("; ");
                    self.print_expr(l);
                }
                self.buf.push(']');
            }
            Type::Fn { params, ret, .. } => {
                self.buf.push_str("fn(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.print_type(p);
                }
                self.buf.push_str(") -> ");
                self.print_type(ret);
            }
            Type::Void(_)  => self.buf.push_str("void"),
            Type::Infer(_) => self.buf.push('_'),
        }
    }

    // ── Generics ──────────────────────────────────────────────────────────────

    fn print_generics(&mut self, gs: &[GenericParam]) {
        if gs.is_empty() { return; }
        self.buf.push('<');
        for (i, g) in gs.iter().enumerate() {
            if i > 0 { self.buf.push_str(", "); }
            self.buf.push_str(&g.name);
            if !g.bounds.is_empty() {
                self.buf.push_str(": ");
                for (j, b) in g.bounds.iter().enumerate() {
                    if j > 0 { self.buf.push_str(" + "); }
                    self.print_type(b);
                }
            }
        }
        self.buf.push('>');
    }

    // ── Statements ────────────────────────────────────────────────────────────

    fn print_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Let(b) => {
                self.ind();
                self.buf.push_str("let ");
                self.print_pattern(&b.pattern);
                if let Some(ty) = &b.ty {
                    self.buf.push_str(": ");
                    self.print_type(ty);
                }
                if let Some(init) = &b.init {
                    self.buf.push_str(" = ");
                    self.print_expr(init);
                }
                self.buf.push_str(";\n");
            }
            Stmt::Expr(e, _) => {
                self.ind();
                self.print_expr(e);
                // Omit semicolons for block-like expressions.
                match &e.kind {
                    ExprKind::If { .. } | ExprKind::While { .. }
                    | ExprKind::For { .. } | ExprKind::Loop(_) => {}
                    _ => { self.buf.push(';'); }
                }
                self.buf.push('\n');
            }
            Stmt::Item(item) => self.print_item(item),
        }
    }

    // ── Patterns ──────────────────────────────────────────────────────────────

    fn print_pattern(&mut self, p: &Pattern) {
        match &p.kind {
            PatternKind::Wildcard => self.buf.push('_'),
            PatternKind::Ident { name, mutable } => {
                if *mutable { self.buf.push_str("mut "); }
                self.buf.push_str(name);
            }
            PatternKind::Literal(lit) => match lit {
                LitPat::Int(n)   => self.buf.push_str(&n.to_string()),
                LitPat::Float(f) => self.buf.push_str(&f.to_string()),
                LitPat::Bool(b)  => self.buf.push_str(if *b { "true" } else { "false" }),
                LitPat::Str(s)   => { self.buf.push('"'); self.buf.push_str(s); self.buf.push('"'); }
                LitPat::Char(c)  => { self.buf.push('\''); self.buf.push(*c); self.buf.push('\''); }
            }
            PatternKind::Tuple(elems) => {
                self.buf.push('(');
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.print_pattern(e);
                }
                self.buf.push(')');
            }
            PatternKind::Array(elems) => {
                self.buf.push('[');
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.print_pattern(e);
                }
                self.buf.push(']');
            }
            PatternKind::Struct { path, fields, rest } => {
                self.print_path(path);
                self.buf.push_str(" { ");
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(&f.name);
                    if let Some(p2) = &f.pat {
                        self.buf.push_str(": ");
                        self.print_pattern(p2);
                    }
                }
                if *rest { self.buf.push_str(", .."); }
                self.buf.push_str(" }");
            }
            PatternKind::TupleStruct { path, elems } => {
                self.print_path(path);
                self.buf.push('(');
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.print_pattern(e);
                }
                self.buf.push(')');
            }
            PatternKind::Or(pats) => {
                for (i, p) in pats.iter().enumerate() {
                    if i > 0 { self.buf.push_str(" | "); }
                    self.print_pattern(p);
                }
            }
            PatternKind::Range { start, end, inclusive } => {
                if let Some(s) = start { self.print_pattern(s); }
                self.buf.push_str(if *inclusive { "..=" } else { ".." });
                if let Some(e) = end { self.print_pattern(e); }
            }
        }
    }

    // ── Paths ─────────────────────────────────────────────────────────────────

    fn print_path(&mut self, p: &Path) {
        for (i, seg) in p.segments.iter().enumerate() {
            if i > 0 { self.buf.push_str("::"); }
            self.buf.push_str(&seg.name);
            if !seg.args.is_empty() {
                self.buf.push('<');
                for (j, a) in seg.args.iter().enumerate() {
                    if j > 0 { self.buf.push_str(", "); }
                    self.print_type(a);
                }
                self.buf.push('>');
            }
        }
    }

    // ── Expressions ───────────────────────────────────────────────────────────

    fn print_expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::IntLit(n)   => self.buf.push_str(&n.to_string()),
            ExprKind::FloatLit(f) => self.buf.push_str(&f.to_string()),
            ExprKind::BoolLit(b)  => self.buf.push_str(if *b { "true" } else { "false" }),
            ExprKind::StrLit(s)   => { self.buf.push('"'); self.buf.push_str(s); self.buf.push('"'); }
            ExprKind::CharLit(c)  => { self.buf.push('\''); self.buf.push(*c); self.buf.push('\''); }
            ExprKind::Path(p)     => self.print_path(p),

            ExprKind::Unary { op, expr } => {
                self.buf.push_str(match op {
                    UnaryOp::Neg   => "-",
                    UnaryOp::Not   => "!",
                    UnaryOp::Deref => "*",
                });
                self.print_expr(expr);
            }
            ExprKind::Binary { op, lhs, rhs } => {
                self.print_expr(lhs);
                self.buf.push(' ');
                self.buf.push_str(binop_str(*op));
                self.buf.push(' ');
                self.print_expr(rhs);
            }
            ExprKind::Assign { target, rhs } => {
                self.print_expr(target);
                self.buf.push_str(" = ");
                self.print_expr(rhs);
            }
            ExprKind::CompoundAssign { op, target, rhs } => {
                self.print_expr(target);
                self.buf.push(' ');
                self.buf.push_str(binop_str(*op));
                self.buf.push_str("= ");
                self.print_expr(rhs);
            }

            ExprKind::If { cond, then, else_ } => {
                self.buf.push_str("if ");
                self.print_expr(cond);
                self.buf.push(' ');
                self.print_block(then);
                if let Some(el) = else_ {
                    self.buf.push_str(" else ");
                    match &el.kind {
                        ExprKind::If { .. } => self.print_expr(el),
                        ExprKind::Block(b)  => self.print_block(b),
                        _                   => { self.buf.push_str("{ "); self.print_expr(el); self.buf.push_str(" }"); }
                    }
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.buf.push_str("match ");
                self.print_expr(scrutinee);
                self.buf.push_str(" {\n");
                self.push();
                for arm in arms {
                    self.ind();
                    self.print_pattern(&arm.pattern);
                    if let Some(g) = &arm.guard {
                        self.buf.push_str(" if ");
                        self.print_expr(g);
                    }
                    self.buf.push_str(" => ");
                    self.print_expr(&arm.body);
                    self.buf.push_str(",\n");
                }
                self.pop();
                self.ind();
                self.buf.push('}');
            }
            ExprKind::Loop(body) => {
                self.buf.push_str("loop ");
                self.print_block(body);
            }
            ExprKind::While { cond, body } => {
                self.buf.push_str("while ");
                self.print_expr(cond);
                self.buf.push(' ');
                self.print_block(body);
            }
            ExprKind::For { pat, iter, body } => {
                self.buf.push_str("for ");
                self.print_pattern(pat);
                self.buf.push_str(" in ");
                self.print_expr(iter);
                self.buf.push(' ');
                self.print_block(body);
            }
            ExprKind::Break { value } => {
                self.buf.push_str("break");
                if let Some(v) = value { self.buf.push(' '); self.print_expr(v); }
            }
            ExprKind::Continue => self.buf.push_str("continue"),
            ExprKind::Return { value } => {
                self.buf.push_str("return");
                if let Some(v) = value { self.buf.push(' '); self.print_expr(v); }
            }

            ExprKind::Call { callee, args } => {
                self.print_expr(callee);
                self.buf.push('(');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.print_expr(a);
                }
                self.buf.push(')');
            }
            ExprKind::MethodCall { receiver, method, args } => {
                self.print_expr(receiver);
                self.buf.push('.');
                self.buf.push_str(method);
                self.buf.push('(');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.print_expr(a);
                }
                self.buf.push(')');
            }
            ExprKind::Field { base, field } => {
                self.print_expr(base);
                self.buf.push('.');
                self.buf.push_str(field);
            }
            ExprKind::Index { base, index } => {
                self.print_expr(base);
                self.buf.push('[');
                self.print_expr(index);
                self.buf.push(']');
            }

            ExprKind::StructLit { ty, fields } => {
                self.print_path(ty);
                self.buf.push_str(" { ");
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(&f.name);
                    if let Some(v) = &f.value {
                        self.buf.push_str(": ");
                        self.print_expr(v);
                    }
                }
                self.buf.push_str(" }");
            }
            ExprKind::TupleLit(elems) => {
                self.buf.push('(');
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.print_expr(e);
                }
                self.buf.push(')');
            }
            ExprKind::ArrayLit(elems) => {
                self.buf.push('[');
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.print_expr(e);
                }
                self.buf.push(']');
            }
            ExprKind::ArrayRepeat { elem, len } => {
                self.buf.push('[');
                self.print_expr(elem);
                self.buf.push_str("; ");
                self.print_expr(len);
                self.buf.push(']');
            }
            ExprKind::Closure { params, ret, body } => {
                self.buf.push('(');
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.print_pattern(&p.pat);
                    if let Some(ty) = &p.ty {
                        self.buf.push_str(": ");
                        self.print_type(ty);
                    }
                }
                self.buf.push(')');
                if let Some(ret) = ret {
                    self.buf.push_str(": ");
                    self.print_type(ret);
                }
                self.buf.push_str(" => ");
                self.print_expr(body);
            }
            ExprKind::TemplateLit { parts } => {
                self.buf.push('`');
                for p in parts {
                    match p {
                        TemplatePart::Str(s)  => self.buf.push_str(s),
                        TemplatePart::Expr(e) => {
                            self.buf.push_str("${");
                            self.print_expr(e);
                            self.buf.push('}');
                        }
                    }
                }
                self.buf.push('`');
            }
            ExprKind::Await(expr) => {
                self.print_expr(expr);
                self.buf.push_str(".await");
            }
            ExprKind::Cast { expr, ty } => {
                self.print_expr(expr);
                self.buf.push_str(" as ");
                self.print_type(ty);
            }
            ExprKind::Block(b) => self.print_block(b),
            ExprKind::Range { start, end, inclusive } => {
                if let Some(s) = start { self.print_expr(s); }
                self.buf.push_str(if *inclusive { "..=" } else { ".." });
                if let Some(e) = end { self.print_expr(e); }
            }
            ExprKind::MacroCall { name, .. } => {
                self.buf.push_str(name);
                self.buf.push_str("!(...)");
            }
        }
    }

    // ── Struct / Enum / Trait / Impl ──────────────────────────────────────────

    fn print_struct(&mut self, s: &StructDef) {
        self.ind();
        if s.vis == Visibility::Public { self.buf.push_str("export "); }
        self.buf.push_str("struct ");
        self.buf.push_str(&s.name);
        self.print_generics(&s.generics);
        self.buf.push_str(" {\n");
        self.push();
        for f in &s.fields {
            self.ind();
            if f.vis == Visibility::Public { self.buf.push_str("pub "); }
            self.buf.push_str(&f.name);
            self.buf.push_str(": ");
            self.print_type(&f.ty);
            self.buf.push_str(",\n");
        }
        self.pop();
        self.ind();
        self.buf.push_str("}\n");
    }

    fn print_enum(&mut self, e: &EnumDef) {
        self.ind();
        if e.vis == Visibility::Public { self.buf.push_str("export "); }
        self.buf.push_str("enum ");
        self.buf.push_str(&e.name);
        self.print_generics(&e.generics);
        self.buf.push_str(" {\n");
        self.push();
        for v in &e.variants {
            self.ind();
            self.buf.push_str(&v.name);
            match &v.kind {
                VariantKind::Unit => {}
                VariantKind::Tuple(ts) => {
                    self.buf.push('(');
                    for (i, t) in ts.iter().enumerate() {
                        if i > 0 { self.buf.push_str(", "); }
                        self.print_type(t);
                    }
                    self.buf.push(')');
                }
                VariantKind::Struct(fs) => {
                    self.buf.push_str(" { ");
                    for (i, f) in fs.iter().enumerate() {
                        if i > 0 { self.buf.push_str(", "); }
                        self.buf.push_str(&f.name);
                        self.buf.push_str(": ");
                        self.print_type(&f.ty);
                    }
                    self.buf.push_str(" }");
                }
            }
            self.buf.push_str(",\n");
        }
        self.pop();
        self.ind();
        self.buf.push_str("}\n");
    }

    fn print_trait(&mut self, t: &TraitDef) {
        self.ind();
        if t.vis == Visibility::Public { self.buf.push_str("export "); }
        self.buf.push_str("trait ");
        self.buf.push_str(&t.name);
        self.print_generics(&t.generics);
        self.buf.push_str(" {\n");
        self.push();
        for item in &t.items {
            match item {
                TraitItem::FnSig(sig) => {
                    self.ind();
                    self.buf.push_str("fn ");
                    self.buf.push_str(&sig.name);
                    self.print_generics(&sig.generics);
                    self.buf.push('(');
                    for (i, p) in sig.params.iter().enumerate() {
                        if i > 0 { self.buf.push_str(", "); }
                        self.buf.push_str(&p.name);
                        self.buf.push_str(": ");
                        self.print_type(&p.ty);
                    }
                    self.buf.push(')');
                    if let Some(r) = &sig.return_ty {
                        self.buf.push_str(": ");
                        self.print_type(r);
                    }
                    self.buf.push_str(";\n");
                }
                TraitItem::FnDef(f) => self.print_fn(f),
                TraitItem::TypeAssoc { name, .. } => {
                    self.ind();
                    self.buf.push_str("type ");
                    self.buf.push_str(name);
                    self.buf.push_str(";\n");
                }
            }
        }
        self.pop();
        self.ind();
        self.buf.push_str("}\n");
    }

    fn print_impl(&mut self, im: &ImplBlock) {
        self.ind();
        self.buf.push_str("impl");
        self.print_generics(&im.generics);
        if let Some(tr) = &im.trait_ {
            self.buf.push(' ');
            self.print_type(tr);
            self.buf.push_str(" for ");
        } else {
            self.buf.push(' ');
        }
        self.print_type(&im.self_ty);
        self.buf.push_str(" {\n");
        self.push();
        for item in &im.items {
            match item {
                ImplItem::Fn(f)  => self.print_fn(f),
                ImplItem::Const(c) => self.print_const_item(c),
                ImplItem::TypeAssoc { name, ty, .. } => {
                    self.ind();
                    self.buf.push_str("type ");
                    self.buf.push_str(name);
                    self.buf.push_str(" = ");
                    self.print_type(ty);
                    self.buf.push_str(";\n");
                }
            }
        }
        self.pop();
        self.ind();
        self.buf.push_str("}\n");
    }

    fn print_type_alias(&mut self, a: &TypeAlias) {
        self.ind();
        if a.vis == Visibility::Public { self.buf.push_str("export "); }
        self.buf.push_str("type ");
        self.buf.push_str(&a.name);
        self.print_generics(&a.generics);
        self.buf.push_str(" = ");
        self.print_type(&a.ty);
        self.buf.push_str(";\n");
    }

    fn print_import(&mut self, d: &ImportDecl) {
        self.ind();
        self.buf.push_str("import ");
        match &d.items {
            ImportItems::Named(names) => {
                self.buf.push_str("{ ");
                for (i, n) in names.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(&n.name);
                    if let Some(alias) = &n.alias {
                        self.buf.push_str(" as ");
                        self.buf.push_str(alias);
                    }
                }
                self.buf.push_str(" } ");
            }
            ImportItems::Glob => self.buf.push_str("* "),
        }
        self.buf.push_str("from \"");
        self.buf.push_str(&d.source);
        self.buf.push_str("\";\n");
    }

    fn print_mod(&mut self, m: &ModDecl) {
        self.ind();
        if m.vis == Visibility::Public { self.buf.push_str("export "); }
        self.buf.push_str("mod ");
        self.buf.push_str(&m.name);
        if let Some(body) = &m.body {
            self.buf.push_str(" {\n");
            self.push();
            self.print_module(body);
            self.pop();
            self.ind();
            self.buf.push_str("}\n");
        } else {
            self.buf.push_str(";\n");
        }
    }

    fn print_const_item(&mut self, c: &ConstItem) {
        self.ind();
        if c.vis == Visibility::Public { self.buf.push_str("export "); }
        self.buf.push_str("const ");
        self.buf.push_str(&c.name);
        if let Some(ty) = &c.ty {
            self.buf.push_str(": ");
            self.print_type(ty);
        }
        self.buf.push_str(" = ");
        self.print_expr(&c.value);
        self.buf.push_str(";\n");
    }
}

fn binop_str(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add    => "+",  BinaryOp::Sub    => "-",
        BinaryOp::Mul    => "*",  BinaryOp::Div    => "/",  BinaryOp::Rem    => "%",
        BinaryOp::BitAnd => "&",  BinaryOp::BitOr  => "|",  BinaryOp::BitXor => "^",
        BinaryOp::Shl    => "<<", BinaryOp::Shr    => ">>",
        BinaryOp::And    => "&&", BinaryOp::Or     => "||",
        BinaryOp::Eq     => "==", BinaryOp::Ne     => "!=",
        BinaryOp::Lt     => "<",  BinaryOp::Le     => "<=",
        BinaryOp::Gt     => ">",  BinaryOp::Ge     => ">=",
    }
}
