//! AST → IR lowering pass.
//!
//! Walks the typed AST and emits IR instructions into a CFG of `BasicBlock`s.
//! Each expression returns the `IrLocal` register where its result lands, or
//! `None` for void-typed expressions.

use smol_str::SmolStr;
use std::collections::HashMap;
use selvr_parser::ast::*;
use crate::ir::*;

// ── Lowering context ─────────────────────────────────────────────────────────

struct LowerCtx {
    fns:        Vec<IrFn>,
    globals:    Vec<IrGlobal>,
    // Per-function state:
    blocks:     Vec<BasicBlock>,
    cur_block:  usize,
    next_local: u32,
    next_block: u32,
    scopes:     Vec<HashMap<SmolStr, IrLocal>>,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            fns: Vec::new(), globals: Vec::new(),
            blocks: Vec::new(), cur_block: 0,
            next_local: 0, next_block: 0,
            scopes: Vec::new(),
        }
    }

    fn fresh_local(&mut self) -> IrLocal {
        let l = IrLocal(self.next_local);
        self.next_local += 1;
        l
    }

    fn fresh_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block);
        self.next_block += 1;
        self.blocks.push(BasicBlock::new(id));
        id
    }

    fn cur(&mut self) -> &mut BasicBlock { &mut self.blocks[self.cur_block] }

    fn emit(&mut self, instr: Instr) { self.cur().instrs.push(instr); }

    fn set_term(&mut self, term: Terminator) { self.cur().term = term; }

    fn switch_to(&mut self, id: BlockId) { self.cur_block = id.0 as usize; }

    fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }
    fn pop_scope(&mut self)  { self.scopes.pop(); }

    fn define(&mut self, name: SmolStr, local: IrLocal) {
        self.scopes.last_mut().unwrap().insert(name, local);
    }

    fn lookup(&self, name: &SmolStr) -> Option<IrLocal> {
        for scope in self.scopes.iter().rev() {
            if let Some(&l) = scope.get(name) { return Some(l); }
        }
        None
    }

    fn begin_fn(&mut self) {
        self.blocks = Vec::new();
        self.cur_block = 0;
        self.next_local = 0;
        self.next_block = 0;
        self.scopes = Vec::new();
        self.push_scope();
        self.fresh_block(); // block 0 = entry
    }
}

// ── Public entry ─────────────────────────────────────────────────────────────

pub fn lower_module(module: &Module) -> IrModule {
    let mut ctx = LowerCtx::new();

    for item in &module.items {
        match item {
            Item::FnDef(f)    => lower_fn(&mut ctx, f),
            Item::ImplBlock(i) => lower_impl(&mut ctx, i),
            Item::Const(c)    => lower_const(&mut ctx, c),
            _                 => {}
        }
    }

    IrModule { name: SmolStr::new("main"), fns: ctx.fns, globals: ctx.globals }
}

// ── Function ──────────────────────────────────────────────────────────────────

fn lower_fn(ctx: &mut LowerCtx, f: &FnDef) {
    ctx.begin_fn();
    let mut params = Vec::new();
    for param in &f.params {
        let local = ctx.fresh_local();
        params.push(local);
        ctx.define(param.name.clone(), local);
    }
    lower_block(ctx, &f.body);
    if matches!(ctx.cur().term, Terminator::Unreachable) {
        ctx.set_term(Terminator::Return(None));
    }
    let num_locals = ctx.next_local;
    let attrs: Vec<smol_str::SmolStr> = f.attrs.iter().map(|a| a.name.clone()).collect();
    ctx.fns.push(IrFn {
        name: f.name.clone(),
        params,
        ret_ty: IrType::Any,
        blocks: std::mem::take(&mut ctx.blocks),
        num_locals,
        is_async: f.is_async,
        is_export: matches!(f.vis, Visibility::Public),
        attrs,
        target: Target::Auto,
    });
}

fn lower_impl(ctx: &mut LowerCtx, imp: &ImplBlock) {
    // Resolve the type name from the self_ty.
    let ty_name = match &imp.self_ty {
        Type::Named { name, .. } => name.clone(),
        _ => SmolStr::new("_"),
    };
    for item in &imp.items {
        if let ImplItem::Fn(f) = item {
            // Mangle name: "TypeName::method"
            let mut f2 = f.clone();
            f2.name = SmolStr::new(format!("{}::{}", ty_name, f.name));
            lower_fn(ctx, &f2);
        }
    }
}

fn lower_const(ctx: &mut LowerCtx, c: &ConstItem) {
    if let Some(k) = expr_to_const(&c.value) {
        ctx.globals.push(IrGlobal { name: c.name.clone(), ty: k.ty(), init: k });
    }
}

// ── Block & statements ────────────────────────────────────────────────────────

fn lower_block(ctx: &mut LowerCtx, block: &Block) {
    ctx.push_scope();
    for stmt in &block.stmts {
        lower_stmt(ctx, stmt);
    }
    if let Some(tail) = &block.tail {
        lower_expr(ctx, tail);
    }
    ctx.pop_scope();
}

fn lower_stmt(ctx: &mut LowerCtx, stmt: &Stmt) {
    match stmt {
        Stmt::Let(ls) => {
            let dst = ctx.fresh_local();
            // Bind simple ident pattern.
            if let PatternKind::Ident { name, .. } = &ls.pattern.kind {
                ctx.define(name.clone(), dst);
            }
            if let Some(init) = &ls.init {
                let src = lower_expr_val(ctx, init);
                ctx.emit(Instr::Assign { dst, src });
            }
        }
        Stmt::Expr(e, _) => {
            lower_expr(ctx, e);
        }
        Stmt::Item(_) => {}
    }
}

// ── Expressions ───────────────────────────────────────────────────────────────

fn lower_expr(ctx: &mut LowerCtx, expr: &Expr) -> Option<IrLocal> {
    match &expr.kind {
        ExprKind::IntLit(n)   => some_const(ctx, Constant::I32(*n as i32)),
        ExprKind::FloatLit(f) => some_const(ctx, Constant::F64(*f)),
        ExprKind::BoolLit(b)  => some_const(ctx, Constant::Bool(*b)),
        ExprKind::StrLit(s)   => some_const(ctx, Constant::Str(s.clone())),
        ExprKind::CharLit(c)  => some_const(ctx, Constant::I32(*c as i32)),

        ExprKind::Path(path) => {
            if let Some(seg) = path.segments.first() {
                if let Some(local) = ctx.lookup(&seg.name) {
                    return Some(local);
                }
                let dst = ctx.fresh_local();
                ctx.emit(Instr::Assign { dst, src: Value::Global(seg.name.clone()) });
                return Some(dst);
            }
            None
        }

        ExprKind::Binary { op, lhs, rhs } => {
            let l = lower_expr_val(ctx, lhs);
            let r = lower_expr_val(ctx, rhs);
            let dst = ctx.fresh_local();
            ctx.emit(Instr::BinOp { dst, op: binop(*op), lhs: l, rhs: r });
            Some(dst)
        }

        ExprKind::Unary { op, expr: operand } => {
            let src = lower_expr_val(ctx, operand);
            let dst = ctx.fresh_local();
            let ir_op = match op {
                UnaryOp::Neg => UnOp::Neg,
                UnaryOp::Not => UnOp::Not,
                _ => return Some(ctx.fresh_local()),
            };
            ctx.emit(Instr::UnOp { dst, op: ir_op, src });
            Some(dst)
        }

        ExprKind::Assign { target, rhs } => {
            let val = lower_expr_val(ctx, rhs);
            if let ExprKind::Path(p) = &target.kind {
                if let Some(seg) = p.segments.first() {
                    if let Some(dst) = ctx.lookup(&seg.name) {
                        ctx.emit(Instr::Assign { dst, src: val });
                    }
                }
            } else if let ExprKind::Field { base, field } = &target.kind {
                if let Some(base_l) = lower_expr_local(ctx, base) {
                    ctx.emit(Instr::SetField { base: base_l, field: field.clone(), val });
                }
            }
            None
        }

        ExprKind::CompoundAssign { op, target, rhs } => {
            let lhs_val = lower_expr_val(ctx, target);
            let rhs_val = lower_expr_val(ctx, rhs);
            let tmp = ctx.fresh_local();
            ctx.emit(Instr::BinOp { dst: tmp, op: binop(*op), lhs: lhs_val, rhs: rhs_val });
            if let ExprKind::Path(p) = &target.kind {
                if let Some(seg) = p.segments.first() {
                    if let Some(dst) = ctx.lookup(&seg.name) {
                        ctx.emit(Instr::Assign { dst, src: Value::Local(tmp) });
                    }
                }
            }
            None
        }

        ExprKind::Call { callee, args } => {
            let func = lower_expr_val(ctx, callee);
            let arg_vals: Vec<Value> = args.iter().map(|a| lower_expr_val(ctx, a)).collect();
            let dst = ctx.fresh_local();
            ctx.emit(Instr::Call { dst: Some(dst), func, args: arg_vals });
            Some(dst)
        }

        ExprKind::MethodCall { receiver, method, args } => {
            let recv = lower_expr_local(ctx, receiver)?;
            let method_val = Value::Global(SmolStr::new(format!("__method_{method}")));
            let mut all_args = vec![Value::Local(recv)];
            all_args.extend(args.iter().map(|a| lower_expr_val(ctx, a)));
            let dst = ctx.fresh_local();
            ctx.emit(Instr::Call { dst: Some(dst), func: method_val, args: all_args });
            Some(dst)
        }

        ExprKind::Field { base, field } => {
            let base_l = lower_expr_local(ctx, base)?;
            let dst = ctx.fresh_local();
            ctx.emit(Instr::GetField { dst, base: base_l, field: field.clone() });
            Some(dst)
        }

        ExprKind::Index { base, index } => {
            let array = lower_expr_local(ctx, base)?;
            let idx   = lower_expr_val(ctx, index);
            let dst   = ctx.fresh_local();
            ctx.emit(Instr::ArrayGet { dst, array, idx });
            Some(dst)
        }

        ExprKind::ArrayLit(elems) => {
            let vals: Vec<Value> = elems.iter().map(|e| lower_expr_val(ctx, e)).collect();
            let dst = ctx.fresh_local();
            ctx.emit(Instr::NewArray { dst, elems: vals });
            Some(dst)
        }

        ExprKind::TupleLit(elems) => {
            let vals: Vec<Value> = elems.iter().map(|e| lower_expr_val(ctx, e)).collect();
            let dst = ctx.fresh_local();
            ctx.emit(Instr::NewArray { dst, elems: vals });
            Some(dst)
        }

        ExprKind::ArrayRepeat { elem, len } => {
            let val = lower_expr_val(ctx, elem);
            let len_val = lower_expr_val(ctx, len);
            let dst = ctx.fresh_local();
            // Emit as a single-element array for now — full repeat requires a loop.
            ctx.emit(Instr::NewArray { dst, elems: vec![val] });
            let _ = len_val;
            Some(dst)
        }

        ExprKind::StructLit { ty, fields } => {
            let ty_name = ty.segments.last()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| SmolStr::new("_"));
            let ir_fields: Vec<(SmolStr, Value)> = fields.iter().map(|f| {
                let v = f.value.as_ref()
                    .map(|e| lower_expr_val(ctx, e))
                    .unwrap_or(Value::Const(Constant::Unit));
                (f.name.clone(), v)
            }).collect();
            let dst = ctx.fresh_local();
            ctx.emit(Instr::NewStruct { dst, name: ty_name, fields: ir_fields });
            Some(dst)
        }

        ExprKind::If { cond, then, else_ } => {
            let cond_val = lower_expr_val(ctx, cond);
            let then_bb  = ctx.fresh_block();
            let else_bb  = ctx.fresh_block();
            let merge_bb = ctx.fresh_block();

            ctx.set_term(Terminator::Branch { cond: cond_val, then_bb, else_bb });

            ctx.switch_to(then_bb);
            lower_block(ctx, then);
            if matches!(ctx.cur().term, Terminator::Unreachable) {
                ctx.set_term(Terminator::Jump(merge_bb));
            }

            ctx.switch_to(else_bb);
            if let Some(eb) = else_ {
                lower_expr(ctx, eb);
            }
            if matches!(ctx.cur().term, Terminator::Unreachable) {
                ctx.set_term(Terminator::Jump(merge_bb));
            }

            ctx.switch_to(merge_bb);
            None
        }

        ExprKind::While { cond, body } => {
            let cond_bb = ctx.fresh_block();
            let body_bb = ctx.fresh_block();
            let exit_bb = ctx.fresh_block();

            ctx.set_term(Terminator::Jump(cond_bb));
            ctx.switch_to(cond_bb);
            let cv = lower_expr_val(ctx, cond);
            ctx.set_term(Terminator::Branch { cond: cv, then_bb: body_bb, else_bb: exit_bb });

            ctx.switch_to(body_bb);
            lower_block(ctx, body);
            ctx.set_term(Terminator::Jump(cond_bb));

            ctx.switch_to(exit_bb);
            None
        }

        ExprKind::For { pat, iter, body } => {
            let iter_local = lower_expr_local(ctx, iter)?;
            let idx    = ctx.fresh_local();
            let len    = ctx.fresh_local();
            let cond_v = ctx.fresh_local();
            let elem   = ctx.fresh_local();

            ctx.emit(Instr::Assign  { dst: idx, src: Value::Const(Constant::I32(0)) });
            ctx.emit(Instr::GetField { dst: len, base: iter_local, field: SmolStr::new("length") });

            let header_bb = ctx.fresh_block();
            let body_bb   = ctx.fresh_block();
            let exit_bb   = ctx.fresh_block();

            ctx.set_term(Terminator::Jump(header_bb));
            ctx.switch_to(header_bb);
            ctx.emit(Instr::BinOp { dst: cond_v, op: BinOp::Lt, lhs: Value::Local(idx), rhs: Value::Local(len) });
            ctx.set_term(Terminator::Branch { cond: Value::Local(cond_v), then_bb: body_bb, else_bb: exit_bb });

            ctx.switch_to(body_bb);
            ctx.emit(Instr::ArrayGet { dst: elem, array: iter_local, idx: Value::Local(idx) });
            ctx.push_scope();
            if let PatternKind::Ident { name, .. } = &pat.kind { ctx.define(name.clone(), elem); }
            lower_block(ctx, body);
            ctx.pop_scope();
            ctx.emit(Instr::BinOp { dst: idx, op: BinOp::Add, lhs: Value::Local(idx), rhs: Value::Const(Constant::I32(1)) });
            ctx.set_term(Terminator::Jump(header_bb));

            ctx.switch_to(exit_bb);
            None
        }

        ExprKind::Loop(body) => {
            let loop_bb = ctx.fresh_block();
            ctx.set_term(Terminator::Jump(loop_bb));
            ctx.switch_to(loop_bb);
            lower_block(ctx, body);
            ctx.set_term(Terminator::Jump(loop_bb));
            None
        }

        ExprKind::Match { scrutinee, arms } => {
            let scrut = lower_expr_local(ctx, scrutinee)?;
            let merge_bb = ctx.fresh_block();
            let result   = ctx.fresh_local();
            for arm in arms {
                let body_bb = ctx.fresh_block();
                let next_bb = ctx.fresh_block();
                let cond = emit_pat_test(ctx, scrut, &arm.pattern);
                ctx.set_term(Terminator::Branch { cond, then_bb: body_bb, else_bb: next_bb });
                ctx.switch_to(body_bb);
                ctx.push_scope();
                bind_ir_pat(ctx, scrut, &arm.pattern);
                if let Some(guard) = &arm.guard { lower_expr(ctx, guard); }
                let arm_val = lower_expr_val(ctx, &arm.body);
                ctx.emit(Instr::Assign { dst: result, src: arm_val });
                ctx.pop_scope();
                ctx.set_term(Terminator::Jump(merge_bb));
                ctx.switch_to(next_bb);
            }
            ctx.set_term(Terminator::Jump(merge_bb));
            ctx.switch_to(merge_bb);
            Some(result)
        }

        ExprKind::Block(b) => { lower_block(ctx, b); None }

        ExprKind::Return { value } => {
            let val = value.as_ref().map(|e| lower_expr_val(ctx, e));
            ctx.set_term(Terminator::Return(val));
            let dead = ctx.fresh_block();
            ctx.switch_to(dead);
            None
        }

        ExprKind::Break { .. } | ExprKind::Continue => None,

        ExprKind::Await(e) => lower_expr(ctx, e),

        ExprKind::Cast { expr, .. } => lower_expr(ctx, expr),

        ExprKind::Closure { params, body, .. } => {
            let fn_name = SmolStr::new(format!("__closure_{}", ctx.next_local));
            // We model the closure as a synthetic FnDef.
            // Synthesise parameter names from closure patterns.
            let fn_params: Vec<Param> = params.iter().enumerate().map(|(i, cp)| {
                let name = match &cp.pat.kind {
                    PatternKind::Ident { name, .. } => name.clone(),
                    _ => SmolStr::new(format!("__p{i}")),
                };
                let ty = cp.ty.clone().unwrap_or(Type::Infer(cp.span));
                Param { name, ty, span: cp.span }
            }).collect();
            let fn_def = FnDef {
                vis: Visibility::Private,
                name: fn_name.clone(),
                generics: Vec::new(),
                params: fn_params,
                return_ty: None,
                body: Block { stmts: Vec::new(), tail: Some(body.clone()), span: body.span },
                is_async: false,
                attrs: Vec::new(),
                span: expr.span,
            };
            lower_fn(ctx, &fn_def);
            let dst = ctx.fresh_local();
            ctx.emit(Instr::Closure { dst, fn_name, captures: Vec::new() });
            Some(dst)
        }

        ExprKind::TemplateLit { parts } => {
            // Concatenate parts into a string.
            let mut pieces: Vec<Value> = Vec::new();
            for part in parts {
                match part {
                    TemplatePart::Str(s) => pieces.push(Value::Const(Constant::Str(s.clone()))),
                    TemplatePart::Expr(e) => pieces.push(lower_expr_val(ctx, e)),
                }
            }
            let mut acc = ctx.fresh_local();
            ctx.emit(Instr::Assign { dst: acc, src: pieces.first().cloned().unwrap_or(Value::Const(Constant::Str(SmolStr::new("")))) });
            for val in pieces.into_iter().skip(1) {
                let next = ctx.fresh_local();
                ctx.emit(Instr::BinOp { dst: next, op: BinOp::Add, lhs: Value::Local(acc), rhs: val });
                acc = next;
            }
            Some(acc)
        }

        ExprKind::Range { .. } | ExprKind::MacroCall { .. } => None,
    }
}

fn lower_expr_val(ctx: &mut LowerCtx, expr: &Expr) -> Value {
    match lower_expr(ctx, expr) {
        Some(l) => Value::Local(l),
        None    => Value::Const(Constant::Unit),
    }
}

fn lower_expr_local(ctx: &mut LowerCtx, expr: &Expr) -> Option<IrLocal> {
    match lower_expr_val(ctx, expr) {
        Value::Local(l) => Some(l),
        other => {
            let dst = ctx.fresh_local();
            ctx.emit(Instr::Assign { dst, src: other });
            Some(dst)
        }
    }
}

fn some_const(ctx: &mut LowerCtx, k: Constant) -> Option<IrLocal> {
    let dst = ctx.fresh_local();
    ctx.emit(Instr::Assign { dst, src: Value::Const(k) });
    Some(dst)
}

// ── Pattern helpers ───────────────────────────────────────────────────────────

fn emit_pat_test(ctx: &mut LowerCtx, scrut: IrLocal, pat: &Pattern) -> Value {
    let dst = ctx.fresh_local();
    match &pat.kind {
        PatternKind::Wildcard | PatternKind::Ident { .. } => {
            ctx.emit(Instr::Assign { dst, src: Value::Const(Constant::Bool(true)) });
        }
        PatternKind::Literal(lit) => {
            let k = match lit {
                LitPat::Int(n)   => Constant::I32(*n as i32),
                LitPat::Float(f) => Constant::F64(*f),
                LitPat::Bool(b)  => Constant::Bool(*b),
                LitPat::Str(s)   => Constant::Str(s.clone()),
                LitPat::Char(c)  => Constant::I32(*c as i32),
            };
            ctx.emit(Instr::BinOp { dst, op: BinOp::Eq, lhs: Value::Local(scrut), rhs: Value::Const(k) });
        }
        PatternKind::TupleStruct { path, .. } => {
            let variant = path.segments.last().map(|s| s.name.as_str()).unwrap_or("");
            if variant == "None" {
                ctx.emit(Instr::IsNone { dst, val: scrut });
            } else if variant == "Some" {
                let tmp = ctx.fresh_local();
                ctx.emit(Instr::IsNone { dst: tmp, val: scrut });
                ctx.emit(Instr::UnOp { dst, op: UnOp::Not, src: Value::Local(tmp) });
            } else {
                ctx.emit(Instr::Assign { dst, src: Value::Const(Constant::Bool(true)) });
            }
        }
        _ => { ctx.emit(Instr::Assign { dst, src: Value::Const(Constant::Bool(true)) }); }
    }
    Value::Local(dst)
}

fn bind_ir_pat(ctx: &mut LowerCtx, scrut: IrLocal, pat: &Pattern) {
    match &pat.kind {
        PatternKind::Ident { name, .. } => ctx.define(name.clone(), scrut),
        PatternKind::TupleStruct { elems, .. } => {
            if let Some(inner) = elems.first() {
                let unwrapped = ctx.fresh_local();
                ctx.emit(Instr::Unwrap { dst: unwrapped, val: scrut });
                bind_ir_pat(ctx, unwrapped, inner);
            }
        }
        _ => {}
    }
}

// ── Operator mapping ──────────────────────────────────────────────────────────

fn binop(op: BinaryOp) -> BinOp {
    match op {
        BinaryOp::Add    => BinOp::Add,
        BinaryOp::Sub    => BinOp::Sub,
        BinaryOp::Mul    => BinOp::Mul,
        BinaryOp::Div    => BinOp::Div,
        BinaryOp::Rem    => BinOp::Rem,
        BinaryOp::Eq     => BinOp::Eq,
        BinaryOp::Ne     => BinOp::Ne,
        BinaryOp::Lt     => BinOp::Lt,
        BinaryOp::Le     => BinOp::Le,
        BinaryOp::Gt     => BinOp::Gt,
        BinaryOp::Ge     => BinOp::Ge,
        BinaryOp::And    => BinOp::And,
        BinaryOp::Or     => BinOp::Or,
        BinaryOp::BitAnd => BinOp::BitAnd,
        BinaryOp::BitOr  => BinOp::BitOr,
        BinaryOp::BitXor => BinOp::BitXor,
        BinaryOp::Shl    => BinOp::Shl,
        BinaryOp::Shr    => BinOp::Shr,
    }
}

// ── Constant helpers ──────────────────────────────────────────────────────────

fn expr_to_const(expr: &Expr) -> Option<Constant> {
    match &expr.kind {
        ExprKind::IntLit(n)   => Some(Constant::I32(*n as i32)),
        ExprKind::FloatLit(f) => Some(Constant::F64(*f)),
        ExprKind::BoolLit(b)  => Some(Constant::Bool(*b)),
        ExprKind::StrLit(s)   => Some(Constant::Str(s.clone())),
        _ => None,
    }
}
