//! IR → Bytecode emitter.
//!
//! Walks each `IrFn` in an `IrModule` and translates its basic-block CFG into
//! a linear instruction stream, resolving jumps with a two-pass approach:
//!
//!   Pass 1 — emit instructions with placeholder jump offsets (0xDEAD_BEEF).
//!   Pass 2 — back-patch jump offsets once all block positions are known.

use selvr_ir::{IrModule, IrFn, BasicBlock, BlockId, Instr, Terminator, Value, Constant, BinOp, UnOp};
use crate::module::{BytecodeModule, BcFn, ConstPool};
use crate::opcode::Op;

// ── Public entry point ────────────────────────────────────────────────────────

pub fn emit_module(ir: &IrModule, source: &str) -> BytecodeModule {
    let mut bc = BytecodeModule::new(source);

    for ir_fn in &ir.fns {
        let bc_fn = emit_fn(&mut bc.const_pool, ir_fn);
        bc.fns.push(bc_fn);
    }

    bc
}

// ── Function emitter ──────────────────────────────────────────────────────────

fn emit_fn(pool: &mut ConstPool, f: &IrFn) -> BcFn {
    let name_idx = pool.intern_str(&f.name);
    let mut emitter = FnEmitter::new(pool);

    // Emit each basic block in order.
    let block_count = f.blocks.len();
    for i in 0..block_count {
        emitter.begin_block(f.blocks[i].id);
        emit_block(&mut emitter, &f.blocks[i]);
    }

    emitter.backpatch();

    BcFn {
        name_idx,
        param_count: f.params.len() as u8,
        local_count:  f.num_locals as u16,
        code:         emitter.code,
        is_export:    f.is_export,
        is_async:     f.is_async,
    }
}

// ── Per-function emitter state ────────────────────────────────────────────────

struct FnEmitter<'a> {
    pool:   &'a mut ConstPool,
    code:   Vec<u8>,
    /// Byte position of the start of each block (keyed by BlockId.0).
    block_offsets: Vec<Option<usize>>,
    /// List of (byte position of placeholder, target BlockId) for back-patching.
    patches: Vec<(usize, BlockId)>,
}

impl<'a> FnEmitter<'a> {
    fn new(pool: &'a mut ConstPool) -> Self {
        Self {
            pool,
            code: Vec::new(),
            block_offsets: vec![None; 64],
            patches: Vec::new(),
        }
    }

    fn pos(&self) -> usize { self.code.len() }

    fn begin_block(&mut self, id: BlockId) {
        let idx = id.0 as usize;
        if idx >= self.block_offsets.len() {
            self.block_offsets.resize(idx + 1, None);
        }
        self.block_offsets[idx] = Some(self.pos());
    }

    fn emit_u8(&mut self, v: u8)  { self.code.push(v); }
    fn emit_op(&mut self, op: Op) { self.code.push(op as u8); }

    fn emit_i16(&mut self, v: i16) {
        self.code.extend_from_slice(&v.to_le_bytes());
    }
    fn emit_u16(&mut self, v: u16) {
        self.code.extend_from_slice(&v.to_le_bytes());
    }
    fn emit_i32(&mut self, v: i32) {
        self.code.extend_from_slice(&v.to_le_bytes());
    }
    fn emit_i64(&mut self, v: i64) {
        self.code.extend_from_slice(&v.to_le_bytes());
    }
    fn emit_f64(&mut self, v: f64) {
        self.code.extend_from_slice(&v.to_le_bytes());
    }

    /// Emit a placeholder jump and record it for back-patching.
    fn emit_jump_placeholder(&mut self, target: BlockId) {
        let patch_pos = self.pos();
        self.emit_i32(0x0BAD_0000_u32 as i32);  // sentinel
        self.patches.push((patch_pos, target));
    }

    /// Back-patch all recorded jump placeholders.
    fn backpatch(&mut self) {
        for (patch_pos, target) in self.patches.drain(..).collect::<Vec<_>>() {
            let target_off = self.block_offsets
                .get(target.0 as usize)
                .and_then(|x| *x)
                .unwrap_or(0);
            // Offset is relative to the instruction *after* the 4-byte immediate.
            let after_imm = patch_pos + 4;
            let rel = target_off as i64 - after_imm as i64;
            let rel_i32 = rel as i32;
            self.code[patch_pos..patch_pos + 4].copy_from_slice(&rel_i32.to_le_bytes());
        }
    }
}

// ── Block emitter ─────────────────────────────────────────────────────────────

fn emit_block(e: &mut FnEmitter, bb: &BasicBlock) {
    for instr in &bb.instrs {
        emit_instr(e, instr);
    }
    emit_term(e, &bb.term);
}

fn emit_term(e: &mut FnEmitter, term: &Terminator) {
    match term {
        Terminator::Return(Some(val)) => {
            emit_value(e, val);
            e.emit_op(Op::Return);
        }
        Terminator::Return(None) => {
            e.emit_op(Op::ReturnVoid);
        }
        Terminator::Jump(target) => {
            e.emit_op(Op::Jump);
            e.emit_jump_placeholder(*target);
        }
        Terminator::Branch { cond, then_bb, else_bb } => {
            emit_value(e, cond);
            e.emit_op(Op::JumpT);
            e.emit_jump_placeholder(*then_bb);
            e.emit_op(Op::Jump);
            e.emit_jump_placeholder(*else_bb);
        }
        Terminator::Unreachable => {
            // Emit a return void so the code stream is always valid.
            e.emit_op(Op::ReturnVoid);
        }
    }
}

fn emit_instr(e: &mut FnEmitter, instr: &Instr) {
    match instr {
        Instr::Assign { dst, src } => {
            emit_value(e, src);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::BinOp { dst, op, lhs, rhs } => {
            emit_value(e, lhs);
            emit_value(e, rhs);
            e.emit_op(binop_to_op(*op));
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::UnOp { dst, op, src } => {
            emit_value(e, src);
            e.emit_op(unop_to_op(*op));
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::Call { dst, func, args } => {
            // Push args left to right, then func, then CALL arity.
            for arg in args { emit_value(e, arg); }
            emit_value(e, func);
            e.emit_op(Op::Call);
            e.emit_u8(args.len() as u8);
            if let Some(d) = dst {
                e.emit_op(Op::StoreLocal);
                e.emit_u16(d.0 as u16);
            } else {
                e.emit_op(Op::Pop);
            }
        }

        Instr::NewStruct { dst, name, fields } => {
            for (_, val) in fields { emit_value(e, val); }
            let name_idx = e.pool.intern_name(name);
            e.emit_op(Op::NewStruct);
            e.emit_u16(name_idx);
            e.emit_u16(fields.len() as u16);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::GetField { dst, base, field } => {
            e.emit_op(Op::LoadLocal);
            e.emit_u16(base.0 as u16);
            let field_idx = e.pool.intern_name(field);
            e.emit_op(Op::GetField);
            e.emit_u16(field_idx);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::SetField { base, field, val } => {
            e.emit_op(Op::LoadLocal);
            e.emit_u16(base.0 as u16);
            emit_value(e, val);
            let field_idx = e.pool.intern_name(field);
            e.emit_op(Op::SetField);
            e.emit_u16(field_idx);
        }

        Instr::NewArray { dst, elems } => {
            for elem in elems { emit_value(e, elem); }
            e.emit_op(Op::NewArray);
            e.emit_u16(elems.len() as u16);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::ArrayGet { dst, array, idx } => {
            e.emit_op(Op::LoadLocal);
            e.emit_u16(array.0 as u16);
            emit_value(e, idx);
            e.emit_op(Op::ArrayGet);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::ArraySet { array, idx, val } => {
            e.emit_op(Op::LoadLocal);
            e.emit_u16(array.0 as u16);
            emit_value(e, idx);
            emit_value(e, val);
            e.emit_op(Op::ArraySet);
        }

        Instr::WrapSome { dst, val } => {
            emit_value(e, val);
            e.emit_op(Op::WrapSome);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::WrapOk { dst, val } => {
            emit_value(e, val);
            e.emit_op(Op::WrapOk);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::WrapErr { dst, val } => {
            emit_value(e, val);
            e.emit_op(Op::WrapErr);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::IsNone { dst, val } => {
            e.emit_op(Op::LoadLocal);
            e.emit_u16(val.0 as u16);
            e.emit_op(Op::IsNone);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::IsErr { dst, val } => {
            e.emit_op(Op::LoadLocal);
            e.emit_u16(val.0 as u16);
            e.emit_op(Op::IsErr);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::Unwrap { dst, val } => {
            e.emit_op(Op::LoadLocal);
            e.emit_u16(val.0 as u16);
            e.emit_op(Op::Unwrap);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::Closure { dst, fn_name, captures } => {
            for &cap in captures {
                e.emit_op(Op::LoadLocal);
                e.emit_u16(cap.0 as u16);
            }
            let fn_idx = e.pool.intern_str(fn_name);
            e.emit_op(Op::MakeClosure);
            e.emit_u16(fn_idx);
            e.emit_u8(captures.len() as u8);
            e.emit_op(Op::StoreLocal);
            e.emit_u16(dst.0 as u16);
        }

        Instr::Nop => { e.emit_op(Op::Nop); }
    }
}

fn emit_value(e: &mut FnEmitter, val: &Value) {
    match val {
        Value::Local(l) => {
            e.emit_op(Op::LoadLocal);
            e.emit_u16(l.0 as u16);
        }
        Value::Const(c) => emit_const(e, c),
        Value::Global(name) => {
            let idx = e.pool.intern_str(name);
            e.emit_op(Op::LoadGlobal);
            e.emit_u16(idx);
        }
    }
}

fn emit_const(e: &mut FnEmitter, c: &Constant) {
    match c {
        Constant::I32(v)  => { e.emit_op(Op::PushI32); e.emit_i32(*v); }
        Constant::I64(v)  => { e.emit_op(Op::PushI64); e.emit_i64(*v); }
        Constant::F64(v)  => { e.emit_op(Op::PushF64); e.emit_f64(*v); }
        Constant::Bool(b) => { e.emit_op(Op::PushBool); e.emit_u8(if *b { 1 } else { 0 }); }
        Constant::Str(s)  => {
            let idx = e.pool.intern_str(s);
            e.emit_op(Op::PushStr);
            e.emit_u16(idx);
        }
        Constant::None  => { e.emit_op(Op::PushNone); e.emit_u16(0); }
        Constant::Unit  => { e.emit_op(Op::PushUnit); e.emit_u16(0); }
    }
}

// ── Operator mapping ──────────────────────────────────────────────────────────

fn binop_to_op(op: BinOp) -> Op {
    match op {
        BinOp::Add    => Op::Add,
        BinOp::Sub    => Op::Sub,
        BinOp::Mul    => Op::Mul,
        BinOp::Div    => Op::Div,
        BinOp::Rem    => Op::Rem,
        BinOp::Eq     => Op::Eq,
        BinOp::Ne     => Op::Ne,
        BinOp::Lt     => Op::Lt,
        BinOp::Le     => Op::Le,
        BinOp::Gt     => Op::Gt,
        BinOp::Ge     => Op::Ge,
        BinOp::And    => Op::And,
        BinOp::Or     => Op::Or,
        BinOp::BitAnd => Op::BitAnd,
        BinOp::BitOr  => Op::BitOr,
        BinOp::BitXor => Op::BitXor,
        BinOp::Shl    => Op::Shl,
        BinOp::Shr    => Op::Shr,
    }
}

fn unop_to_op(op: UnOp) -> Op {
    match op {
        UnOp::Neg => Op::Neg,
        UnOp::Not => Op::Not,
    }
}
