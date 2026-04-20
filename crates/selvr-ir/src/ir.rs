//! IR type definitions.

use smol_str::SmolStr;

// ── Identifiers ──────────────────────────────────────────────────────────────

/// A numbered local register within a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IrLocal(pub u32);

/// A basic block label within a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

// ── IR type tags (used for code generation decisions) ────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    I32, I64, F64, Bool, Str, Void,
    Ref(Box<IrType>),   // heap-allocated object
    Any,                // union / erased type
}

// ── Constant values ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    I32(i32),
    I64(i64),
    F64(f64),
    Bool(bool),
    Str(SmolStr),
    None,
    Unit,
}

impl Constant {
    pub fn ty(&self) -> IrType {
        match self {
            Constant::I32(_)  => IrType::I32,
            Constant::I64(_)  => IrType::I64,
            Constant::F64(_)  => IrType::F64,
            Constant::Bool(_) => IrType::Bool,
            Constant::Str(_)  => IrType::Str,
            Constant::None | Constant::Unit => IrType::Any,
        }
    }
}

// ── Value operands ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Value {
    Local(IrLocal),
    Const(Constant),
    /// Reference to a top-level function or global.
    Global(SmolStr),
}

// ── Binary & Unary operators ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp { Neg, Not }

// ── Instructions ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Instr {
    /// `dst = src`
    Assign { dst: IrLocal, src: Value },

    /// `dst = lhs op rhs`
    BinOp  { dst: IrLocal, op: BinOp, lhs: Value, rhs: Value },

    /// `dst = op src`
    UnOp   { dst: IrLocal, op: UnOp,  src: Value },

    /// `dst = func(args)` — `dst` is None for void calls.
    Call   {
        dst:  Option<IrLocal>,
        func: Value,
        args: Vec<Value>,
    },

    /// `dst = new StructName { field: val, ... }`
    NewStruct {
        dst:    IrLocal,
        name:   SmolStr,
        fields: Vec<(SmolStr, Value)>,
    },

    /// `dst = base.field`
    GetField { dst: IrLocal, base: IrLocal, field: SmolStr },

    /// `base.field = val`
    SetField { base: IrLocal, field: SmolStr, val: Value },

    /// `dst = [elems...]`
    NewArray { dst: IrLocal, elems: Vec<Value> },

    /// `dst = array[idx]`
    ArrayGet { dst: IrLocal, array: IrLocal, idx: Value },

    /// `array[idx] = val`
    ArraySet { array: IrLocal, idx: Value, val: Value },

    /// `dst = Some(val)`
    WrapSome { dst: IrLocal, val: Value },

    /// `dst = Ok(val)` / `dst = Err(val)`
    WrapOk  { dst: IrLocal, val: Value },
    WrapErr { dst: IrLocal, val: Value },

    /// `dst = is_none(val)` — bool
    IsNone { dst: IrLocal, val: IrLocal },

    /// `dst = is_err(val)` — bool
    IsErr  { dst: IrLocal, val: IrLocal },

    /// `dst = unwrap(val)` — panics at runtime if None/Err
    Unwrap { dst: IrLocal, val: IrLocal },

    /// Create a closure capturing `captures`.
    Closure {
        dst:      IrLocal,
        fn_name:  SmolStr,
        captures: Vec<IrLocal>,
    },

    /// No-op placeholder / debug annotation.
    Nop,
}

// ── Terminators ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Terminator {
    /// `return val`
    Return(Option<Value>),

    /// Unconditional jump.
    Jump(BlockId),

    /// Conditional branch.
    Branch {
        cond:    Value,
        then_bb: BlockId,
        else_bb: BlockId,
    },

    /// Unreachable (after unconditional `return` / `panic`).
    Unreachable,
}

// ── Basic block ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id:     BlockId,
    pub instrs: Vec<Instr>,
    pub term:   Terminator,
}

impl BasicBlock {
    pub fn new(id: BlockId) -> Self {
        Self { id, instrs: Vec::new(), term: Terminator::Unreachable }
    }
}

// ── Compile-time target annotation ───────────────────────────────────────────

/// Which runtime this function is compiled to.
///
/// Set by the `selvr-target` analysis pass.  Functions start as `Auto`; the
/// pass fills in `Wasm` or `Js` before code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Target {
    /// Compiled to Selvr bytecode and executed by the WASM VM.
    Wasm,
    /// Transpiled to JavaScript and executed by V8 directly.
    Js,
    /// Not yet decided — filled in by the targeting pass.
    #[default]
    Auto,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Target::Wasm => write!(f,"wasm"), Target::Js => write!(f,"js"), Target::Auto => write!(f,"auto") }
    }
}

// ── IR function ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct IrFn {
    pub name:       SmolStr,
    pub params:     Vec<IrLocal>,
    pub ret_ty:     IrType,
    pub blocks:     Vec<BasicBlock>,
    pub num_locals: u32,
    pub is_async:   bool,
    pub is_export:  bool,
    /// Attribute names from `#[wasm]`, `#[js]`, `#[inline_bridge]` etc.
    pub attrs:      Vec<SmolStr>,
    /// Runtime target assigned by the targeting pass (starts as `Auto`).
    pub target:     Target,
}

impl IrFn {
    /// Returns the entry block (always index 0).
    pub fn entry(&self) -> &BasicBlock { &self.blocks[0] }
}

// ── Global ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct IrGlobal {
    pub name: SmolStr,
    pub ty:   IrType,
    pub init: Constant,
}

// ── IR module ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct IrModule {
    pub name:    SmolStr,
    pub fns:     Vec<IrFn>,
    pub globals: Vec<IrGlobal>,
}
