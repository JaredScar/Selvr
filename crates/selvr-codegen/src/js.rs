//! JavaScript code emitter — transforms Selvr AST nodes to JS source text.
//!
//! Design:
//!  - `JsEmitter` accumulates output in a `String` buffer.
//!  - Each `emit_*` method appends to the buffer and optionally records a source-map entry.
//!  - The emitter produces modern ES2022 output (const/let, classes, async/await, etc.).
//!  - Selvr-specific runtime helpers (Option, pattern matching, etc.) are injected via
//!    the `SELVR_RUNTIME_PREAMBLE` constant and loaded from `SELVR-runtime.js`.

use selvr_parser::ast::{*, TemplatePart};
use crate::{error::CodegenError, sourcemap::SourceMap};

/// Minimal JS runtime preamble — full version will be loaded from `SELVR-runtime.js`.
pub const SELVR_RUNTIME_PREAMBLE: &str = r#"
// Selvr runtime — DO NOT EDIT (generated)
const __selvr = {
  Some: (v) => ({ tag: "Some", val: v }),
  None: { tag: "None" },
  isSome: (o) => o.tag === "Some",
  isNone: (o) => o.tag === "None",
  unwrap: (o) => { if (o.tag !== "Some") throw new Error("unwrap called on None"); return o.val; },
  Ok: (v) => ({ tag: "Ok", val: v }),
  Err: (e) => ({ tag: "Err", err: e }),
  panic: (msg) => { throw new Error("SELVR panic: " + msg); },
  print: (...args) => console.log(...args),
};
"#;

pub struct JsEmitter {
    buf: String,
    indent: usize,
    source_map: SourceMap,
}

impl JsEmitter {
    pub fn new(out_file: &str, src_file: &str) -> Self {
        Self {
            buf: String::new(),
            indent: 0,
            source_map: SourceMap::new(out_file, &[src_file]),
        }
    }

    pub fn emit_module(mut self, module: &Module) -> Result<(String, String), CodegenError> {
        self.buf.push_str(SELVR_RUNTIME_PREAMBLE);
        self.buf.push('\n');
        for item in &module.items {
            self.emit_item(item)?;
            self.buf.push('\n');
        }
        let sm_json = self.source_map.to_json();
        // Append source map reference comment
        self.buf.push_str("\n//# sourceMappingURL=output.js.map\n");
        Ok((self.buf, sm_json))
    }

    fn emit_item(&mut self, item: &Item) -> Result<(), CodegenError> {
        match item {
            Item::FnDef(f)      => self.emit_fn(f),
            Item::StructDef(s)  => self.emit_struct(s),
            Item::EnumDef(e)    => self.emit_enum(e),
            Item::Const(c)      => self.emit_const(c),
            Item::ImportDecl(_) => Ok(()), // handled by the module bundler
            Item::ModDecl(m)    => self.emit_mod(m),
            Item::TypeAlias(_)  => Ok(()), // erased at runtime
            _ => Err(CodegenError::Unsupported(format!("{:?}", std::mem::discriminant(item)))),
        }
    }

    // ── Functions ─────────────────────────────────────────────────────────────

    fn emit_fn(&mut self, f: &FnDef) -> Result<(), CodegenError> {
        self.write_indent();
        if f.vis == Visibility::Public {
            self.buf.push_str("export ");
        }
        if f.is_async { self.buf.push_str("async "); }
        self.buf.push_str("function ");
        self.buf.push_str(&f.name);
        self.buf.push('(');
        for (i, param) in f.params.iter().enumerate() {
            if i > 0 { self.buf.push_str(", "); }
            self.buf.push_str(&param.name);
        }
        self.buf.push_str(") ");
        self.emit_block(&f.body)?;
        self.buf.push('\n');
        Ok(())
    }

    // ── Structs ───────────────────────────────────────────────────────────────

    fn emit_struct(&mut self, s: &StructDef) -> Result<(), CodegenError> {
        self.write_indent();
        if s.vis == Visibility::Public { self.buf.push_str("export "); }
        self.buf.push_str(&format!("class {} {{\n", s.name));
        self.indent += 1;
        // Constructor
        self.write_indent();
        self.buf.push_str("constructor(");
        for (i, f) in s.fields.iter().enumerate() {
            if i > 0 { self.buf.push_str(", "); }
            self.buf.push_str(&f.name);
        }
        self.buf.push_str(") {\n");
        self.indent += 1;
        for f in &s.fields {
            self.write_indent();
            self.buf.push_str(&format!("this.{} = {};\n", f.name, f.name));
        }
        self.indent -= 1;
        self.write_indent();
        self.buf.push_str("}\n");
        self.indent -= 1;
        self.write_indent();
        self.buf.push_str("}\n");
        Ok(())
    }

    // ── Enums ─────────────────────────────────────────────────────────────────

    fn emit_enum(&mut self, e: &EnumDef) -> Result<(), CodegenError> {
        // Enums compile to a frozen object of tag-bearing factory functions.
        self.write_indent();
        if e.vis == Visibility::Public { self.buf.push_str("export "); }
        self.buf.push_str(&format!("const {} = Object.freeze({{\n", e.name));
        self.indent += 1;
        for v in &e.variants {
            self.write_indent();
            match &v.kind {
                VariantKind::Unit => {
                    self.buf.push_str(&format!("{}: {{ tag: {:?} }},\n", v.name, v.name.as_str()));
                }
                VariantKind::Tuple(fields) => {
                    let args: Vec<String> = (0..fields.len()).map(|i| format!("_{i}")).collect();
                    self.buf.push_str(&format!(
                        "{}: ({}) => ({{ tag: {:?}, {} }}),\n",
                        v.name,
                        args.join(", "),
                        v.name.as_str(),
                        args.iter().map(|a| format!("{a}")).collect::<Vec<_>>().join(", ")
                    ));
                }
                VariantKind::Struct(fields) => {
                    let names: Vec<String> = fields.iter().map(|f| f.name.to_string()).collect();
                    self.buf.push_str(&format!(
                        "{}: ({{{ }}}) => ({{ tag: {:?}, {} }}),\n",
                        v.name,
                        names.join(", "),
                        v.name.as_str(),
                        names.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
                    ));
                }
            }
        }
        self.indent -= 1;
        self.write_indent();
        self.buf.push_str("});\n");
        Ok(())
    }

    // ── Const ─────────────────────────────────────────────────────────────────

    fn emit_const(&mut self, c: &ConstItem) -> Result<(), CodegenError> {
        self.write_indent();
        if c.vis == Visibility::Public { self.buf.push_str("export "); }
        self.buf.push_str("const ");
        self.buf.push_str(&c.name);
        self.buf.push_str(" = ");
        self.emit_expr(&c.value)?;
        self.buf.push_str(";\n");
        Ok(())
    }

    // ── Module ────────────────────────────────────────────────────────────────

    fn emit_mod(&mut self, m: &ModDecl) -> Result<(), CodegenError> {
        if let Some(body) = &m.body {
            for item in &body.items {
                self.emit_item(item)?;
            }
        }
        Ok(())
    }

    // ── Blocks ────────────────────────────────────────────────────────────────

    fn emit_block(&mut self, block: &Block) -> Result<(), CodegenError> {
        self.buf.push_str("{\n");
        self.indent += 1;
        for stmt in &block.stmts {
            self.emit_stmt(stmt)?;
        }
        if let Some(tail) = &block.tail {
            self.write_indent();
            self.buf.push_str("return ");
            self.emit_expr(tail)?;
            self.buf.push_str(";\n");
        }
        self.indent -= 1;
        self.write_indent();
        self.buf.push('}');
        Ok(())
    }

    // ── Statements ────────────────────────────────────────────────────────────

    fn emit_stmt(&mut self, stmt: &Stmt) -> Result<(), CodegenError> {
        match stmt {
            Stmt::Let(l) => {
                self.write_indent();
                // `let` in Selvr is always mutable; `const` is only for
                // top-level ConstItem.  Always emit `let` here.
                self.buf.push_str("let ");
                self.emit_pattern(&l.pattern)?;
                if let Some(init) = &l.init {
                    self.buf.push_str(" = ");
                    self.emit_expr(init)?;
                }
                self.buf.push_str(";\n");
            }
            Stmt::Expr(e, _) => {
                // Emit control-flow expressions as native JS statements so
                // they don't get wrapped in IIFEs.
                match &e.kind {
                    ExprKind::If { cond, then, else_ } => {
                        self.write_indent();
                        self.buf.push_str("if (");
                        self.emit_expr(cond)?;
                        self.buf.push_str(") ");
                        self.emit_block(then)?;
                        if let Some(el) = else_ {
                            self.buf.push_str(" else ");
                            match &el.kind {
                                ExprKind::If { .. } => self.emit_expr(el)?,
                                ExprKind::Block(b)  => self.emit_block(b)?,
                                _                   => self.emit_expr(el)?,
                            }
                        }
                        self.buf.push('\n');
                    }
                    ExprKind::While { cond, body } => {
                        self.write_indent();
                        self.buf.push_str("while (");
                        self.emit_expr(cond)?;
                        self.buf.push_str(") ");
                        self.emit_block(body)?;
                        self.buf.push('\n');
                    }
                    ExprKind::For { pat, iter, body } => {
                        self.write_indent();
                        self.buf.push_str("for (const ");
                        self.emit_pattern(pat)?;
                        self.buf.push_str(" of ");
                        self.emit_expr(iter)?;
                        self.buf.push_str(") ");
                        self.emit_block(body)?;
                        self.buf.push('\n');
                    }
                    ExprKind::Loop(body) => {
                        self.write_indent();
                        self.buf.push_str("while (true) ");
                        self.emit_block(body)?;
                        self.buf.push('\n');
                    }
                    _ => {
                        self.write_indent();
                        self.emit_expr(e)?;
                        self.buf.push_str(";\n");
                    }
                }
            }
            Stmt::Item(item) => self.emit_item(item)?,
        }
        Ok(())
    }

    // ── Patterns (in let / destructuring) ─────────────────────────────────────

    fn emit_pattern(&mut self, pat: &Pattern) -> Result<(), CodegenError> {
        match &pat.kind {
            PatternKind::Ident { name, .. } => { self.buf.push_str(name); }
            PatternKind::Wildcard => { self.buf.push('_'); }
            PatternKind::Tuple(pats) => {
                self.buf.push('[');
                for (i, p) in pats.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.emit_pattern(p)?;
                }
                self.buf.push(']');
            }
            _ => return Err(CodegenError::Unsupported("complex pattern in let".into())),
        }
        Ok(())
    }

    // ── Expressions ───────────────────────────────────────────────────────────

    fn emit_expr(&mut self, expr: &Expr) -> Result<(), CodegenError> {
        match &expr.kind {
            ExprKind::IntLit(v)   => { self.buf.push_str(&v.to_string()); }
            ExprKind::FloatLit(v) => { self.buf.push_str(&format!("{v:?}")); }
            ExprKind::BoolLit(v)  => { self.buf.push_str(if *v { "true" } else { "false" }); }
            ExprKind::StrLit(v)   => { self.buf.push_str(&format!("{v:?}")); }
            ExprKind::CharLit(v)  => { self.buf.push_str(&format!("{v:?}")); }
            ExprKind::Path(p)     => { self.emit_path(p)?; }

            ExprKind::Unary { op, expr } => {
                self.buf.push_str(match op {
                    UnaryOp::Not => "!",
                    UnaryOp::Neg => "-",
                    UnaryOp::Deref => "", // deref is implicit in JS
                });
                self.buf.push('(');
                self.emit_expr(expr)?;
                self.buf.push(')');
            }

            ExprKind::Binary { op, lhs, rhs } => {
                self.buf.push('(');
                self.emit_expr(lhs)?;
                self.buf.push(' ');
                self.buf.push_str(js_binop(*op));
                self.buf.push(' ');
                self.emit_expr(rhs)?;
                self.buf.push(')');
            }

            ExprKind::Assign { target, rhs } => {
                self.emit_expr(target)?;
                self.buf.push_str(" = ");
                self.emit_expr(rhs)?;
            }

            ExprKind::CompoundAssign { op, target, rhs } => {
                self.emit_expr(target)?;
                self.buf.push(' ');
                self.buf.push_str(js_binop(*op));
                self.buf.push_str("= ");
                self.emit_expr(rhs)?;
            }

            ExprKind::Call { callee, args } => {
                self.emit_expr(callee)?;
                self.buf.push('(');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.emit_expr(a)?;
                }
                self.buf.push(')');
            }

            ExprKind::MethodCall { receiver, method, args } => {
                self.emit_expr(receiver)?;
                self.buf.push('.');
                self.buf.push_str(method);
                self.buf.push('(');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.emit_expr(a)?;
                }
                self.buf.push(')');
            }

            ExprKind::Field { base, field } => {
                self.emit_expr(base)?;
                self.buf.push('.');
                self.buf.push_str(field);
            }

            ExprKind::Index { base, index } => {
                self.emit_expr(base)?;
                self.buf.push('[');
                self.emit_expr(index)?;
                self.buf.push(']');
            }

            ExprKind::If { cond, then, else_ } => {
                self.buf.push_str("(() => { if (");
                self.emit_expr(cond)?;
                self.buf.push_str(") ");
                self.emit_block(then)?;
                if let Some(e) = else_ {
                    self.buf.push_str(" else ");
                    match &e.kind {
                        ExprKind::Block(b) => self.emit_block(b)?,
                        _ => self.emit_expr(e)?,
                    }
                }
                self.buf.push_str(" })()");
            }

            ExprKind::Match { scrutinee, arms } => {
                self.emit_match(scrutinee, arms)?;
            }

            ExprKind::Block(b) => {
                self.buf.push_str("(() => ");
                self.emit_block(b)?;
                self.buf.push_str(")()");
            }

            ExprKind::Return { value } => {
                self.buf.push_str("return");
                if let Some(v) = value {
                    self.buf.push(' ');
                    self.emit_expr(v)?;
                }
            }

            ExprKind::Break { value } => {
                // JS `break` can't return a value; this needs a loop wrapper for value-breaks.
                self.buf.push_str("break");
                if value.is_some() {
                    return Err(CodegenError::Unsupported("break with value in JS backend (use loop)".into()));
                }
            }

            ExprKind::Continue => { self.buf.push_str("continue"); }

            ExprKind::While { cond, body } => {
                self.buf.push_str("while (");
                self.emit_expr(cond)?;
                self.buf.push_str(") ");
                self.emit_block(body)?;
            }

            ExprKind::Loop(body) => {
                self.buf.push_str("while (true) ");
                self.emit_block(body)?;
            }

            ExprKind::For { pat, iter, body } => {
                self.buf.push_str("for (const ");
                self.emit_pattern(pat)?;
                self.buf.push_str(" of ");
                self.emit_expr(iter)?;
                self.buf.push_str(") ");
                self.emit_block(body)?;
            }

            ExprKind::Await(expr) => {
                self.buf.push_str("(await ");
                self.emit_expr(expr)?;
                self.buf.push(')');
            }

            ExprKind::TupleLit(elems) => {
                self.buf.push('[');
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.emit_expr(e)?;
                }
                self.buf.push(']');
            }

            ExprKind::ArrayLit(elems) => {
                self.buf.push('[');
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.emit_expr(e)?;
                }
                self.buf.push(']');
            }

            ExprKind::StructLit { ty, fields } => {
                self.buf.push_str("new ");
                self.emit_path(ty)?;
                self.buf.push('(');
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    if let Some(v) = &f.value {
                        self.emit_expr(v)?;
                    } else {
                        self.buf.push_str(&f.name);
                    }
                }
                self.buf.push(')');
            }

            ExprKind::Closure { params, body, .. } => {
                self.buf.push('(');
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.emit_pattern(&p.pat)?;
                }
                self.buf.push_str(") => ");
                self.emit_expr(body)?;
            }

            ExprKind::Cast { expr, .. } => {
                // Most casts are no-ops in JS; numeric casts may need Math.trunc, etc.
                self.emit_expr(expr)?;
            }

            ExprKind::Range { start, end, inclusive } => {
                // Compile to a generator or array slice at call site — stub for now.
                self.buf.push_str("__selvr.range(");
                if let Some(s) = start { self.emit_expr(s)?; } else { self.buf.push_str("undefined"); }
                self.buf.push_str(", ");
                if let Some(e) = end { self.emit_expr(e)?; } else { self.buf.push_str("undefined"); }
                self.buf.push_str(", ");
                self.buf.push_str(if *inclusive { "true" } else { "false" });
                self.buf.push(')');
            }

            ExprKind::TemplateLit { parts } => {
                self.buf.push('`');
                for part in parts {
                    match part {
                        TemplatePart::Str(s) => {
                            self.buf.push_str(s);
                        }
                        TemplatePart::Expr(e) => {
                            self.buf.push_str("${");
                            self.emit_expr(e)?;
                            self.buf.push('}');
                        }
                    }
                }
                self.buf.push('`');
            }

            ExprKind::MacroCall { name, .. } => {
                return Err(CodegenError::Unsupported(format!("macro `{name}!` not expanded")));
            }

            _ => return Err(CodegenError::Unsupported("expression variant".into())),
        }
        Ok(())
    }

    fn emit_path(&mut self, path: &Path) -> Result<(), CodegenError> {
        for (i, seg) in path.segments.iter().enumerate() {
            if i > 0 { self.buf.push('.'); }
            self.buf.push_str(&seg.name);
        }
        Ok(())
    }

    fn emit_match(&mut self, scrutinee: &Expr, arms: &[MatchArm]) -> Result<(), CodegenError> {
        // Compile match to an IIFE with if-else chain.
        self.buf.push_str("(() => {\n");
        self.indent += 1;
        self.write_indent();
        self.buf.push_str("const __m = ");
        self.emit_expr(scrutinee)?;
        self.buf.push_str(";\n");
        for (i, arm) in arms.iter().enumerate() {
            self.write_indent();
            if i > 0 { self.buf.push_str("} else "); }
            self.buf.push_str("if (");
            self.emit_match_condition("__m", &arm.pattern)?;
            self.buf.push_str(") {\n");
            self.indent += 1;
            self.write_indent();
            self.buf.push_str("return ");
            self.emit_expr(&arm.body)?;
            self.buf.push_str(";\n");
            self.indent -= 1;
        }
        self.write_indent();
        self.buf.push_str("} else { __selvr.panic(\"non-exhaustive match\"); }\n");
        self.indent -= 1;
        self.write_indent();
        self.buf.push_str("})()");
        Ok(())
    }

    fn emit_match_condition(&mut self, scrutinee: &str, pat: &Pattern) -> Result<(), CodegenError> {
        match &pat.kind {
            PatternKind::Wildcard => { self.buf.push_str("true"); }
            PatternKind::Literal(lit) => {
                self.buf.push_str(&format!("{scrutinee} === "));
                match lit {
                    LitPat::Int(v)  => { self.buf.push_str(&v.to_string()); }
                    LitPat::Bool(v) => { self.buf.push_str(if *v { "true" } else { "false" }); }
                    LitPat::Str(v)  => { self.buf.push_str(&format!("{v:?}")); }
                    LitPat::Float(v) => { self.buf.push_str(&format!("{v:?}")); }
                    LitPat::Char(v)  => { self.buf.push_str(&format!("{v:?}")); }
                }
            }
            PatternKind::Ident { .. } => { self.buf.push_str("true"); /* binding — always matches */ }
            PatternKind::TupleStruct { path, .. } => {
                // Enum variant: check `.tag`
                let tag = path.segments.last().map(|s| s.name.as_str()).unwrap_or("_");
                self.buf.push_str(&format!("{scrutinee}.tag === {tag:?}"));
            }
            _ => { self.buf.push_str("true"); }
        }
        Ok(())
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.buf.push_str("  ");
        }
    }
}

fn js_binop(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Rem => "%",
        BinaryOp::And => "&&",
        BinaryOp::Or  => "||",
        BinaryOp::Eq  => "===",
        BinaryOp::Ne  => "!==",
        BinaryOp::Lt  => "<",
        BinaryOp::Le  => "<=",
        BinaryOp::Gt  => ">",
        BinaryOp::Ge  => ">=",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr  => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::Shl    => "<<",
        BinaryOp::Shr    => ">>",
    }
}
