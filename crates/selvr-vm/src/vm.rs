//! Stack-machine interpreter loop.
//!
//! # Execution model
//!
//! The VM is a stack machine: each instruction pops its operands from the
//! **value stack** and pushes its result.  Local variables live in a
//! fixed-size **locals array** inside the call frame.
//!
//! Call frames are pushed onto a `Vec<Frame>`.  Each frame carries its own
//! stack slice (simulated via a stack-base offset) and locals array.
//!
//! # Error handling
//!
//! `VmError` represents all unrecoverable runtime errors (type mismatches,
//! panics, division by zero, etc.).  The Selvr `Result<T, E>` and `Option<T>`
//! types are represented as `HeapObj::Ok/Err/Some` and do **not** produce
//! `VmError` — they are regular values.

use std::collections::HashMap;
use smol_str::SmolStr;
use thiserror::Error;

use selvr_bytecode::module::{BytecodeModule, ConstValue};
use selvr_bytecode::opcode::Op;
use crate::mem::{Heap, Value, HeapObj};

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum VmError {
    #[error("type error: expected {expected}, found {found}")]
    TypeError { expected: &'static str, found: String },

    #[error("division by zero")]
    DivisionByZero,

    #[error("index {idx} out of bounds (len {len})")]
    IndexOutOfBounds { idx: usize, len: usize },

    #[error("stack underflow")]
    StackUnderflow,

    #[error("unwrap on None/Err")]
    UnwrapFailed,

    #[error("unresolved global `{0}`")]
    UnresolvedGlobal(String),

    #[error("unknown function `{0}`")]
    UnknownFunction(String),

    #[error("panic: {0}")]
    Panic(String),

    #[error("unknown opcode 0x{0:02x}")]
    UnknownOpcode(u8),
}

// ── Call frame ────────────────────────────────────────────────────────────────

struct Frame {
    /// Byte offset of the next instruction in `fn_idx`'s code.
    pc:       usize,
    /// Index into `BytecodeModule::fns`.
    fn_idx:   usize,
    /// Local variable slots.
    locals:   Vec<Value>,
    /// Bottom of this frame's stack slice (into the shared operand stack).
    stack_base: usize,
}

// ── VM ────────────────────────────────────────────────────────────────────────

/// The Selvr virtual machine.
pub struct Vm {
    pub module: BytecodeModule,
    pub heap:   Heap,
    /// Flat output buffer — `console.log` writes here.
    pub output: Vec<String>,
    /// Global variable store.
    globals: HashMap<SmolStr, Value>,
}

impl Vm {
    pub fn new(module: BytecodeModule) -> Self {
        Self {
            module,
            heap:    Heap::new(),
            output:  Vec::new(),
            globals: HashMap::new(),
        }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Call the named function with `args`.  Returns the return value.
    pub fn call_by_name(&mut self, name: &str, args: Vec<Value>) -> Result<Value, VmError> {
        let fn_idx = self.module.fns.iter().position(|f| {
            self.module.const_pool.get(f.name_idx)
                .map(|e| match e { ConstValue::Str(s) | ConstValue::Name(s) => s.as_str() == name })
                .unwrap_or(false)
        }).ok_or_else(|| VmError::UnknownFunction(name.to_string()))?;

        self.exec(fn_idx, args)
    }

    // ── Execution ─────────────────────────────────────────────────────────────

    fn exec(&mut self, fn_idx: usize, args: Vec<Value>) -> Result<Value, VmError> {
        let bc_fn  = &self.module.fns[fn_idx];
        let locals = {
            let mut l = vec![Value::Unit; bc_fn.local_count as usize];
            for (i, arg) in args.into_iter().enumerate() {
                if i < l.len() { l[i] = arg; }
            }
            l
        };

        let mut stack: Vec<Value> = Vec::new();
        let mut frames: Vec<Frame> = vec![Frame {
            pc: 0, fn_idx, locals, stack_base: 0,
        }];

        loop {
            let frame = frames.last_mut().unwrap();
            let code  = &self.module.fns[frame.fn_idx].code;

            if frame.pc >= code.len() {
                let retval = stack.pop().unwrap_or(Value::Unit);
                frames.pop();
                if frames.is_empty() { return Ok(retval); }
                // Keep return value on the caller's stack.
                stack.push(retval);
                continue;
            }

            let raw = code[frame.pc];
            let op  = Op::from_u8(raw).ok_or(VmError::UnknownOpcode(raw))?;
            frame.pc += 1;

            macro_rules! read_u8  { () => {{ let b = code[frame.pc]; frame.pc += 1; b }}; }
            macro_rules! read_u16 { () => {{
                let b = u16::from_le_bytes(code[frame.pc..frame.pc+2].try_into().unwrap());
                frame.pc += 2; b
            }}; }
            macro_rules! read_i32 { () => {{
                let b = i32::from_le_bytes(code[frame.pc..frame.pc+4].try_into().unwrap());
                frame.pc += 4; b
            }}; }
            macro_rules! read_i64 { () => {{
                let b = i64::from_le_bytes(code[frame.pc..frame.pc+8].try_into().unwrap());
                frame.pc += 8; b
            }}; }
            macro_rules! read_f64 { () => {{
                let b = f64::from_le_bytes(code[frame.pc..frame.pc+8].try_into().unwrap());
                frame.pc += 8; b
            }}; }
            macro_rules! pop { () => {
                stack.pop().ok_or(VmError::StackUnderflow)?
            }; }
            macro_rules! push { ($v:expr) => { stack.push($v); }; }

            match op {
                // ── Push constants ────────────────────────────────────────────
                Op::PushI32  => { let v = read_i32!(); push!(Value::I32(v)); }
                Op::PushI64  => { let v = read_i64!(); push!(Value::I64(v)); }
                Op::PushF64  => { let v = read_f64!(); push!(Value::F64(v)); }
                Op::PushBool => { let v = read_u8!();  push!(Value::Bool(v != 0)); }
                Op::PushStr  => {
                    let idx = read_u16!();
                    let s = match self.module.const_pool.get(idx) {
                        Some(ConstValue::Str(s) | ConstValue::Name(s)) => s.clone(),
                        _ => SmolStr::new(""),
                    };
                    push!(Value::Str(s));
                }
                Op::PushNone => { read_u16!(); push!(Value::None); }
                Op::PushUnit => { read_u16!(); push!(Value::Unit); }

                // ── Locals ────────────────────────────────────────────────────
                Op::LoadLocal => {
                    let idx = read_u16!() as usize;
                    let v = frames.last().unwrap().locals.get(idx)
                        .cloned().unwrap_or(Value::Unit);
                    push!(v);
                }
                Op::StoreLocal => {
                    let idx = read_u16!() as usize;
                    let v = pop!();
                    let frame = frames.last_mut().unwrap();
                    if idx < frame.locals.len() {
                        frame.locals[idx] = v;
                    }
                }

                // ── Globals ───────────────────────────────────────────────────
                Op::LoadGlobal => {
                    let idx = read_u16!();
                    let name = match self.module.const_pool.get(idx) {
                        Some(ConstValue::Str(s) | ConstValue::Name(s)) => s.clone(),
                        _ => SmolStr::new(""),
                    };
                    // Check user globals, then builtins, then functions.
                    if let Some(v) = self.globals.get(&name) {
                        push!(v.clone());
                    } else {
                        // Try to resolve as a function reference.
                        let fn_pos = self.module.fns.iter().position(|f| {
                            self.module.const_pool.get(f.name_idx)
                                .map(|e| match e { ConstValue::Str(s) | ConstValue::Name(s) => s == &name })
                                .unwrap_or(false)
                        });
                        if let Some(pos) = fn_pos {
                            push!(Value::I32(pos as i32)); // function ref = index
                        } else {
                            push!(Value::None);
                        }
                    }
                }
                Op::StoreGlobal => {
                    let idx = read_u16!();
                    let name = match self.module.const_pool.get(idx) {
                        Some(ConstValue::Str(s) | ConstValue::Name(s)) => s.clone(),
                        _ => SmolStr::new(""),
                    };
                    let v = pop!();
                    self.globals.insert(name, v);
                }

                // ── Stack ─────────────────────────────────────────────────────
                Op::Pop  => { pop!(); }
                Op::Dup  => { let v = stack.last().cloned().unwrap_or(Value::Unit); push!(v); }
                Op::Swap => {
                    let a = pop!();
                    let b = pop!();
                    push!(a);
                    push!(b);
                }

                // ── Arithmetic ────────────────────────────────────────────────
                Op::Add => { let (a, b) = (pop!(), pop!()); push!(arith_add(b, a)?); }
                Op::Sub => { let (a, b) = (pop!(), pop!()); push!(arith_sub(b, a)?); }
                Op::Mul => { let (a, b) = (pop!(), pop!()); push!(arith_mul(b, a)?); }
                Op::Div => { let (a, b) = (pop!(), pop!()); push!(arith_div(b, a)?); }
                Op::Rem => { let (a, b) = (pop!(), pop!()); push!(arith_rem(b, a)?); }
                Op::Neg => { let a = pop!(); push!(arith_neg(a)?); }

                // ── Bitwise ───────────────────────────────────────────────────
                Op::BitAnd => { let (a, b) = (pop!(), pop!()); push!(bitwise_and(b, a)?); }
                Op::BitOr  => { let (a, b) = (pop!(), pop!()); push!(bitwise_or(b, a)?); }
                Op::BitXor => { let (a, b) = (pop!(), pop!()); push!(bitwise_xor(b, a)?); }
                Op::Shl    => { let (a, b) = (pop!(), pop!()); push!(bitwise_shl(b, a)?); }
                Op::Shr    => { let (a, b) = (pop!(), pop!()); push!(bitwise_shr(b, a)?); }

                // ── Comparison ────────────────────────────────────────────────
                Op::Eq  => { let (a, b) = (pop!(), pop!()); push!(Value::Bool(val_eq(&b, &a))); }
                Op::Ne  => { let (a, b) = (pop!(), pop!()); push!(Value::Bool(!val_eq(&b, &a))); }
                Op::Lt  => { let (a, b) = (pop!(), pop!()); push!(cmp_op(b, a, |x| x < 0)?); }
                Op::Le  => { let (a, b) = (pop!(), pop!()); push!(cmp_op(b, a, |x| x <= 0)?); }
                Op::Gt  => { let (a, b) = (pop!(), pop!()); push!(cmp_op(b, a, |x| x > 0)?); }
                Op::Ge  => { let (a, b) = (pop!(), pop!()); push!(cmp_op(b, a, |x| x >= 0)?); }

                // ── Boolean ───────────────────────────────────────────────────
                Op::And => { let (a, b) = (pop!(), pop!()); push!(Value::Bool(b.is_truthy() && a.is_truthy())); }
                Op::Or  => { let (a, b) = (pop!(), pop!()); push!(Value::Bool(b.is_truthy() || a.is_truthy())); }
                Op::Not => { let a = pop!(); push!(Value::Bool(!a.is_truthy())); }

                // ── Control flow ──────────────────────────────────────────────
                Op::Jump => {
                    let off = read_i32!();
                    let frame = frames.last_mut().unwrap();
                    frame.pc = (frame.pc as i64 + off as i64) as usize;
                }
                Op::JumpT => {
                    let off = read_i32!();
                    let cond = pop!();
                    if cond.is_truthy() {
                        let frame = frames.last_mut().unwrap();
                        frame.pc = (frame.pc as i64 + off as i64) as usize;
                    }
                }
                Op::JumpF => {
                    let off = read_i32!();
                    let cond = pop!();
                    if !cond.is_truthy() {
                        let frame = frames.last_mut().unwrap();
                        frame.pc = (frame.pc as i64 + off as i64) as usize;
                    }
                }

                // ── Calls & returns ───────────────────────────────────────────
                Op::Call => {
                    let arity   = read_u8!() as usize;
                    let func    = pop!();
                    let mut args = Vec::with_capacity(arity);
                    for _ in 0..arity { args.push(pop!()); }
                    args.reverse();

                    match func {
                        Value::I32(fn_idx) => {
                            let fn_idx = fn_idx as usize;
                            let bc_fn  = &self.module.fns[fn_idx];
                            let mut locals = vec![Value::Unit; bc_fn.local_count as usize];
                            for (i, arg) in args.into_iter().enumerate() {
                                if i < locals.len() { locals[i] = arg; }
                            }
                            let stack_base = stack.len();
                            frames.push(Frame { pc: 0, fn_idx, locals, stack_base });
                        }
                        // Builtin call by name.
                        Value::Str(name) => {
                            let result = self.call_builtin(&name, args)?;
                            push!(result);
                        }
                        _ => return Err(VmError::TypeError {
                            expected: "function",
                            found: func.type_name().to_string(),
                        }),
                    }
                }

                Op::CallNative => {
                    let name_idx = read_u16!();
                    let arity    = read_u8!() as usize;
                    let mut args = Vec::with_capacity(arity);
                    for _ in 0..arity { args.push(pop!()); }
                    args.reverse();
                    let name = match self.module.const_pool.get(name_idx) {
                        Some(ConstValue::Str(s) | ConstValue::Name(s)) => s.clone(),
                        _ => SmolStr::new(""),
                    };
                    let result = self.call_builtin(&name, args)?;
                    push!(result);
                }

                Op::Return => {
                    let retval = pop!();
                    frames.pop();
                    if frames.is_empty() { return Ok(retval); }
                    stack.truncate(frames.last().unwrap().stack_base);
                    push!(retval);
                }

                Op::ReturnVoid => {
                    frames.pop();
                    if frames.is_empty() { return Ok(Value::Unit); }
                    stack.truncate(frames.last().unwrap().stack_base);
                    push!(Value::Unit);
                }

                // ── Objects ───────────────────────────────────────────────────
                Op::NewStruct => {
                    let name_idx    = read_u16!();
                    let field_count = read_u16!() as usize;
                    let ty_name     = match self.module.const_pool.get(name_idx) {
                        Some(ConstValue::Str(s) | ConstValue::Name(s)) => s.clone(),
                        _ => SmolStr::new("unknown"),
                    };
                    let mut fields = HashMap::new();
                    // Fields are pushed in declaration order; pop in reverse.
                    let mut vals: Vec<Value> = {
                        let mut v = Vec::with_capacity(field_count);
                        for _ in 0..field_count { v.push(pop!()); }
                        v
                    };
                    vals.reverse();
                    // We'd need field name ordering from the type system here;
                    // for now use positional names f0, f1, …
                    for (i, v) in vals.into_iter().enumerate() {
                        fields.insert(SmolStr::new(format!("f{i}")), v);
                    }
                    let obj = self.heap.alloc_struct(ty_name, fields);
                    push!(obj);
                }

                Op::GetField => {
                    let name_idx = read_u16!();
                    let field    = match self.module.const_pool.get(name_idx) {
                        Some(ConstValue::Str(s) | ConstValue::Name(s)) => s.clone(),
                        _ => SmolStr::new(""),
                    };
                    let obj = pop!();
                    let val = self.get_field(&obj, &field)?;
                    push!(val);
                }

                Op::SetField => {
                    let name_idx = read_u16!();
                    let field    = match self.module.const_pool.get(name_idx) {
                        Some(ConstValue::Str(s) | ConstValue::Name(s)) => s.clone(),
                        _ => SmolStr::new(""),
                    };
                    let val = pop!();
                    let obj = pop!();
                    self.set_field(&obj, &field, val)?;
                }

                // ── Arrays ────────────────────────────────────────────────────
                Op::NewArray => {
                    let count = read_u16!() as usize;
                    let mut elems: Vec<Value> = {
                        let mut v = Vec::with_capacity(count);
                        for _ in 0..count { v.push(pop!()); }
                        v
                    };
                    elems.reverse();
                    let arr = self.heap.alloc_array(elems);
                    push!(arr);
                }
                Op::ArrayGet => {
                    let idx = pop!();
                    let arr = pop!();
                    let val = self.array_get(&arr, &idx)?;
                    push!(val);
                }
                Op::ArraySet => {
                    let val = pop!();
                    let idx = pop!();
                    let arr = pop!();
                    self.array_set(&arr, &idx, val)?;
                }
                Op::ArrayLen => {
                    let arr = pop!();
                    let len = self.array_len(&arr)?;
                    push!(Value::I32(len as i32));
                }

                // ── Option / Result ───────────────────────────────────────────
                Op::WrapSome => { let v = pop!(); push!(self.heap.alloc_some(v)); }
                Op::WrapOk   => { let v = pop!(); push!(self.heap.alloc_ok(v)); }
                Op::WrapErr  => { let v = pop!(); push!(self.heap.alloc_err(v)); }
                Op::IsNone   => {
                    let v = pop!();
                    let is_none = matches!(v, Value::None)
                        || matches!(&v, Value::Object(i) if matches!(self.heap.get(*i), Some(HeapObj::Some(_))).not());
                    push!(Value::Bool(is_none));
                }
                Op::IsErr => {
                    let v = pop!();
                    let is_err = matches!(&v, Value::Object(i) if matches!(self.heap.get(*i), Some(HeapObj::Err(_))));
                    push!(Value::Bool(is_err));
                }
                Op::Unwrap => {
                    let v = pop!();
                    let inner = self.unwrap_value(v)?;
                    push!(inner);
                }

                // ── Closures ──────────────────────────────────────────────────
                Op::MakeClosure => {
                    let fn_idx       = read_u16!();
                    let capture_count = read_u8!() as usize;
                    let captures: Vec<Value> = {
                        let mut v = Vec::with_capacity(capture_count);
                        for _ in 0..capture_count { v.push(pop!()); }
                        v
                    };
                    let cl = self.heap.alloc_closure(fn_idx, captures);
                    push!(cl);
                }

                Op::Nop => {}

                #[allow(unreachable_patterns)]
                _ => return Err(VmError::UnknownOpcode(raw)),
            }
        }
    }

    // ── Built-in functions ────────────────────────────────────────────────────

    fn call_builtin(&mut self, name: &str, args: Vec<Value>) -> Result<Value, VmError> {
        match name {
            "console.log" | "print" => {
                let msg: Vec<String> = args.iter().map(|v| value_to_string(v, &self.heap)).collect();
                self.output.push(msg.join(" "));
                crate::dom::console_log(&msg.join(" "));
                Ok(Value::Unit)
            }
            "Math.sqrt"  => unary_f64(args, |x| x.sqrt()),
            "Math.abs"   => unary_f64(args, |x| x.abs()),
            "Math.floor" => unary_f64(args, |x| x.floor()),
            "Math.ceil"  => unary_f64(args, |x| x.ceil()),
            "Math.round" => unary_f64(args, |x| x.round()),
            "Math.pow"   => binary_f64(args, |a, b| a.powf(b)),
            "Math.min"   => binary_f64(args, f64::min),
            "Math.max"   => binary_f64(args, f64::max),
            "Math.clamp" => {
                if args.len() < 3 { return Ok(Value::Unit); }
                let v  = to_f64(&args[0])?;
                let lo = to_f64(&args[1])?;
                let hi = to_f64(&args[2])?;
                Ok(Value::F64(v.clamp(lo, hi)))
            }
            "String"  => {
                Ok(Value::Str(SmolStr::new(
                    args.first().map(|v| value_to_string(v, &self.heap)).unwrap_or_default()
                )))
            }
            "parseInt"   => {
                let s = args.first().map(|v| value_to_string(v, &self.heap)).unwrap_or_default();
                Ok(Value::I32(s.parse().unwrap_or(0)))
            }
            "parseFloat" => {
                let s = args.first().map(|v| value_to_string(v, &self.heap)).unwrap_or_default();
                Ok(Value::F64(s.parse().unwrap_or(0.0)))
            }
            _ => Ok(Value::Unit),
        }
    }

    // ── Field accessors ───────────────────────────────────────────────────────

    fn get_field(&self, val: &Value, field: &SmolStr) -> Result<Value, VmError> {
        match val {
            Value::Object(idx) => {
                match self.heap.get(*idx) {
                    Some(HeapObj::Struct { fields, .. }) => {
                        Ok(fields.get(field).cloned().unwrap_or(Value::None))
                    }
                    Some(HeapObj::Array(elems)) if field.as_str() == "length" => {
                        Ok(Value::I32(elems.len() as i32))
                    }
                    _ => Ok(Value::None),
                }
            }
            _ => Ok(Value::None),
        }
    }

    fn set_field(&mut self, val: &Value, field: &SmolStr, new_val: Value) -> Result<(), VmError> {
        if let Value::Object(idx) = val {
            if let Some(HeapObj::Struct { fields, .. }) = self.heap.get_mut(*idx) {
                fields.insert(field.clone(), new_val);
            }
        }
        Ok(())
    }

    fn array_get(&self, arr: &Value, idx: &Value) -> Result<Value, VmError> {
        let i = to_usize(idx)?;
        if let Value::Object(oi) = arr {
            if let Some(HeapObj::Array(elems)) = self.heap.get(*oi) {
                return elems.get(i).cloned()
                    .ok_or(VmError::IndexOutOfBounds { idx: i, len: elems.len() });
            }
        }
        Err(VmError::TypeError { expected: "array", found: arr.type_name().to_string() })
    }

    fn array_set(&mut self, arr: &Value, idx: &Value, val: Value) -> Result<(), VmError> {
        let i = to_usize(idx)?;
        if let Value::Object(oi) = arr {
            if let Some(HeapObj::Array(elems)) = self.heap.get_mut(*oi) {
                if i < elems.len() {
                    elems[i] = val;
                    return Ok(());
                }
                return Err(VmError::IndexOutOfBounds { idx: i, len: elems.len() });
            }
        }
        Err(VmError::TypeError { expected: "array", found: arr.type_name().to_string() })
    }

    fn array_len(&self, arr: &Value) -> Result<usize, VmError> {
        if let Value::Object(i) = arr {
            if let Some(HeapObj::Array(e)) = self.heap.get(*i) { return Ok(e.len()); }
        }
        Err(VmError::TypeError { expected: "array", found: arr.type_name().to_string() })
    }

    fn unwrap_value(&self, val: Value) -> Result<Value, VmError> {
        match &val {
            Value::None => Err(VmError::UnwrapFailed),
            Value::Object(i) => match self.heap.get(*i) {
                Some(HeapObj::Some(v) | HeapObj::Ok(v)) => Ok(v.clone()),
                Some(HeapObj::Err(_)) => Err(VmError::UnwrapFailed),
                _ => Ok(val),
            },
            _ => Ok(val),
        }
    }
}

// ── Helper trait for IsNone ───────────────────────────────────────────────────

trait NotExt {
    fn not(self) -> Self;
}
impl NotExt for bool {
    fn not(self) -> Self { !self }
}

// ── Arithmetic helpers ────────────────────────────────────────────────────────

fn arith_add(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x.wrapping_add(y))),
        (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x.wrapping_add(y))),
        (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x + y)),
        (Value::Str(x), Value::Str(y)) => Ok(Value::Str(SmolStr::new(format!("{x}{y}")))),
        (a, b) => Err(type_err("numeric or string", &format!("{} + {}", a.type_name(), b.type_name()))),
    }
}
fn arith_sub(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x.wrapping_sub(y))),
        (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x.wrapping_sub(y))),
        (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x - y)),
        (a, b) => Err(type_err("numeric", &format!("{} - {}", a.type_name(), b.type_name()))),
    }
}
fn arith_mul(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x.wrapping_mul(y))),
        (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x.wrapping_mul(y))),
        (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x * y)),
        (a, b) => Err(type_err("numeric", &format!("{} * {}", a.type_name(), b.type_name()))),
    }
}
fn arith_div(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::I32(x), Value::I32(y)) => {
            if y == 0 { return Err(VmError::DivisionByZero); }
            Ok(Value::I32(x.wrapping_div(y)))
        }
        (Value::I64(x), Value::I64(y)) => {
            if y == 0 { return Err(VmError::DivisionByZero); }
            Ok(Value::I64(x.wrapping_div(y)))
        }
        (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x / y)),
        (a, b) => Err(type_err("numeric", &format!("{} / {}", a.type_name(), b.type_name()))),
    }
}
fn arith_rem(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::I32(x), Value::I32(y)) => {
            if y == 0 { return Err(VmError::DivisionByZero); }
            Ok(Value::I32(x % y))
        }
        (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x % y)),
        (a, b) => Err(type_err("numeric", &format!("{} % {}", a.type_name(), b.type_name()))),
    }
}
fn arith_neg(a: Value) -> Result<Value, VmError> {
    match a {
        Value::I32(x) => Ok(Value::I32(-x)),
        Value::I64(x) => Ok(Value::I64(-x)),
        Value::F64(x) => Ok(Value::F64(-x)),
        v => Err(type_err("numeric", v.type_name())),
    }
}

fn bitwise_and(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x & y)),
        _ => Err(type_err("integer", "non-integer")),
    }
}
fn bitwise_or(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) { (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x | y)), _ => Err(type_err("integer", "non-integer")), }
}
fn bitwise_xor(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) { (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x ^ y)), _ => Err(type_err("integer", "non-integer")), }
}
fn bitwise_shl(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) { (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x << (y & 31))), _ => Err(type_err("integer", "non-integer")), }
}
fn bitwise_shr(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) { (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x >> (y & 31))), _ => Err(type_err("integer", "non-integer")), }
}

fn val_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::I32(x), Value::I32(y)) => x == y,
        (Value::I64(x), Value::I64(y)) => x == y,
        (Value::F64(x), Value::F64(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y))   => x == y,
        (Value::None, Value::None)       => true,
        (Value::Unit, Value::Unit)       => true,
        _ => false,
    }
}

fn cmp_op(a: Value, b: Value, pred: impl Fn(i32) -> bool) -> Result<Value, VmError> {
    let ord = match (&a, &b) {
        (Value::I32(x), Value::I32(y)) => x.cmp(y) as i32,
        (Value::I64(x), Value::I64(y)) => x.cmp(y) as i32,
        (Value::F64(x), Value::F64(y)) => x.partial_cmp(y).map(|o| o as i32).unwrap_or(0),
        (Value::Str(x), Value::Str(y)) => x.cmp(y) as i32,
        _ => return Err(type_err("comparable", &format!("{} vs {}", a.type_name(), b.type_name()))),
    };
    Ok(Value::Bool(pred(ord)))
}

fn unary_f64(args: Vec<Value>, f: impl Fn(f64) -> f64) -> Result<Value, VmError> {
    let v = args.first().ok_or(VmError::Panic("missing argument".into()))?;
    Ok(Value::F64(f(to_f64(v)?)))
}

fn binary_f64(args: Vec<Value>, f: impl Fn(f64, f64) -> f64) -> Result<Value, VmError> {
    if args.len() < 2 { return Ok(Value::Unit); }
    Ok(Value::F64(f(to_f64(&args[0])?, to_f64(&args[1])?)))
}

fn to_f64(v: &Value) -> Result<f64, VmError> {
    match v {
        Value::F64(f) => Ok(*f),
        Value::I32(n) => Ok(*n as f64),
        Value::I64(n) => Ok(*n as f64),
        v => Err(type_err("f64", v.type_name())),
    }
}

fn to_usize(v: &Value) -> Result<usize, VmError> {
    match v {
        Value::I32(n) => Ok(*n as usize),
        Value::I64(n) => Ok(*n as usize),
        v => Err(type_err("integer", v.type_name())),
    }
}

fn value_to_string(v: &Value, heap: &Heap) -> String {
    match v {
        Value::I32(n)    => n.to_string(),
        Value::I64(n)    => n.to_string(),
        Value::F64(f)    => f.to_string(),
        Value::Bool(b)   => b.to_string(),
        Value::Str(s)    => s.to_string(),
        Value::None      => "None".to_string(),
        Value::Unit      => "()".to_string(),
        Value::Object(i) => match heap.get(*i) {
            Some(HeapObj::Struct { ty_name, fields }) => {
                let fstr: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{k}: {}", value_to_string(v, heap)))
                    .collect();
                format!("{ty_name} {{ {} }}", fstr.join(", "))
            }
            Some(HeapObj::Array(elems)) => {
                let items: Vec<String> = elems.iter()
                    .map(|v| value_to_string(v, heap))
                    .collect();
                format!("[{}]", items.join(", "))
            }
            Some(HeapObj::Some(inner)) => format!("Some({})", value_to_string(inner, heap)),
            Some(HeapObj::Ok(inner))   => format!("Ok({})",   value_to_string(inner, heap)),
            Some(HeapObj::Err(inner))  => format!("Err({})",  value_to_string(inner, heap)),
            Some(HeapObj::Closure { fn_idx, .. }) => format!("<closure@{fn_idx}>"),
            None => "<dangling>".to_string(),
        },
    }
}

fn type_err(expected: &'static str, found: &str) -> VmError {
    VmError::TypeError { expected, found: found.to_string() }
}
