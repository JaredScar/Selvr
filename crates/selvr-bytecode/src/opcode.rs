//! Selvr stack-machine instruction set.
//!
//! Each instruction is 1–9 bytes:
//!   Byte 0       — opcode (u8)
//!   Bytes 1..    — zero or more immediate operands (little-endian)
//!
//! Immediates are typed and sized as noted in each variant's doc comment.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    // ── Constants ────────────────────────────────────────────────────────────
    /// Push 32-bit signed integer.  Imm: i32 (4 B)
    PushI32  = 0x01,
    /// Push 64-bit signed integer.  Imm: i64 (8 B)
    PushI64  = 0x02,
    /// Push 64-bit float.  Imm: f64 (8 B)
    PushF64  = 0x03,
    /// Push boolean.  Imm: u8  (0 = false, 1 = true)
    PushBool = 0x04,
    /// Push interned string.  Imm: u16 — index into the constant pool's string table.
    PushStr  = 0x05,
    /// Push the `None` sentinel.
    PushNone = 0x06,
    /// Push unit (`void`).
    PushUnit = 0x07,

    // ── Locals ───────────────────────────────────────────────────────────────
    /// Load a local variable onto the stack.  Imm: u16
    LoadLocal  = 0x10,
    /// Pop the top of stack into a local variable.  Imm: u16
    StoreLocal = 0x11,

    // ── Globals ───────────────────────────────────────────────────────────────
    /// Load a global / function reference.  Imm: u16 (string pool)
    LoadGlobal  = 0x12,
    /// Store into a global.  Imm: u16 (string pool)
    StoreGlobal = 0x13,

    // ── Stack manipulation ────────────────────────────────────────────────────
    /// Discard the top of stack.
    Pop = 0x20,
    /// Duplicate the top of stack.
    Dup = 0x21,
    /// Swap the top two values.
    Swap = 0x22,

    // ── Arithmetic ───────────────────────────────────────────────────────────
    Add = 0x30,
    Sub = 0x31,
    Mul = 0x32,
    Div = 0x33,
    Rem = 0x34,
    Neg = 0x35,

    // ── Bitwise ──────────────────────────────────────────────────────────────
    BitAnd = 0x38,
    BitOr  = 0x39,
    BitXor = 0x3A,
    Shl    = 0x3B,
    Shr    = 0x3C,

    // ── Comparison ───────────────────────────────────────────────────────────
    Eq  = 0x40,
    Ne  = 0x41,
    Lt  = 0x42,
    Le  = 0x43,
    Gt  = 0x44,
    Ge  = 0x45,

    // ── Boolean ──────────────────────────────────────────────────────────────
    And = 0x48,
    Or  = 0x49,
    Not = 0x4A,

    // ── Control flow ─────────────────────────────────────────────────────────
    /// Unconditional jump.  Imm: i32 relative offset from *next* instruction.
    Jump  = 0x50,
    /// Jump if TOS is true.   Imm: i32 relative offset.
    JumpT = 0x51,
    /// Jump if TOS is false.  Imm: i32 relative offset.
    JumpF = 0x52,

    // ── Calls & returns ───────────────────────────────────────────────────────
    /// Call the function on TOS.  Imm: u8 argument count.
    Call       = 0x60,
    /// Call a native (JS-side) function.  Imm: u16 native index + u8 arg count.
    CallNative = 0x61,
    /// Return TOS to the caller.
    Return     = 0x62,
    /// Return unit to the caller.
    ReturnVoid = 0x63,

    // ── Objects & fields ─────────────────────────────────────────────────────
    /// Allocate a struct.  Imm: u16 string-pool index of the struct name,
    ///                          u16 field count.
    ///   Stack before: field_val_N ... field_val_0
    NewStruct = 0x70,
    /// Get a field.  Imm: u16 string-pool index of the field name.
    GetField  = 0x71,
    /// Set a field.  Imm: u16 string-pool index of the field name.
    ///   Stack: object, value → (side effect)
    SetField  = 0x72,

    // ── Arrays ───────────────────────────────────────────────────────────────
    /// Allocate array.  Imm: u16 element count.
    ///   Stack before: elem_N ... elem_0
    NewArray = 0x78,
    /// Index into array.  Stack: array, idx → value
    ArrayGet = 0x79,
    /// Set array element.  Stack: array, idx, value → (side effect)
    ArraySet = 0x7A,
    /// Push array length.  Stack: array → length (i32)
    ArrayLen = 0x7B,

    // ── Option / Result ───────────────────────────────────────────────────────
    /// Wrap TOS in `Some(...)`.
    WrapSome = 0x80,
    /// Wrap TOS in `Ok(...)`.
    WrapOk   = 0x81,
    /// Wrap TOS in `Err(...)`.
    WrapErr  = 0x82,
    /// Push `true` if TOS is `None`.
    IsNone   = 0x83,
    /// Push `true` if TOS is `Err`.
    IsErr    = 0x84,
    /// Unwrap `Some`/`Ok`; panic on `None`/`Err`.
    Unwrap   = 0x85,

    // ── Closures ─────────────────────────────────────────────────────────────
    /// Create a closure.  Imm: u16 function index, u8 capture count.
    ///   Stack before: capture_N ... capture_0
    MakeClosure = 0x90,

    // ── Misc ─────────────────────────────────────────────────────────────────
    /// No-op.
    Nop = 0xFF,
}

impl Op {
    /// Number of immediate bytes this opcode carries.
    pub fn imm_bytes(self) -> usize {
        use Op::*;
        match self {
            PushI32 => 4,
            PushI64 | PushF64 => 8,
            PushBool => 1,
            PushStr | PushNone | PushUnit => 2,
            LoadLocal | StoreLocal | LoadGlobal | StoreGlobal => 2,
            Pop | Dup | Swap => 0,
            Add | Sub | Mul | Div | Rem | Neg => 0,
            BitAnd | BitOr | BitXor | Shl | Shr => 0,
            Eq | Ne | Lt | Le | Gt | Ge => 0,
            And | Or | Not => 0,
            Jump | JumpT | JumpF => 4,
            Call => 1,
            CallNative => 3,
            Return | ReturnVoid => 0,
            NewStruct => 4,
            GetField | SetField => 2,
            NewArray => 2,
            ArrayGet | ArraySet | ArrayLen => 0,
            WrapSome | WrapOk | WrapErr | IsNone | IsErr | Unwrap => 0,
            MakeClosure => 3,
            Nop => 0,
        }
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        // SAFETY: We only match known discriminants.
        match v {
            0x01 => Some(Op::PushI32),
            0x02 => Some(Op::PushI64),
            0x03 => Some(Op::PushF64),
            0x04 => Some(Op::PushBool),
            0x05 => Some(Op::PushStr),
            0x06 => Some(Op::PushNone),
            0x07 => Some(Op::PushUnit),
            0x10 => Some(Op::LoadLocal),
            0x11 => Some(Op::StoreLocal),
            0x12 => Some(Op::LoadGlobal),
            0x13 => Some(Op::StoreGlobal),
            0x20 => Some(Op::Pop),
            0x21 => Some(Op::Dup),
            0x22 => Some(Op::Swap),
            0x30 => Some(Op::Add),
            0x31 => Some(Op::Sub),
            0x32 => Some(Op::Mul),
            0x33 => Some(Op::Div),
            0x34 => Some(Op::Rem),
            0x35 => Some(Op::Neg),
            0x38 => Some(Op::BitAnd),
            0x39 => Some(Op::BitOr),
            0x3A => Some(Op::BitXor),
            0x3B => Some(Op::Shl),
            0x3C => Some(Op::Shr),
            0x40 => Some(Op::Eq),
            0x41 => Some(Op::Ne),
            0x42 => Some(Op::Lt),
            0x43 => Some(Op::Le),
            0x44 => Some(Op::Gt),
            0x45 => Some(Op::Ge),
            0x48 => Some(Op::And),
            0x49 => Some(Op::Or),
            0x4A => Some(Op::Not),
            0x50 => Some(Op::Jump),
            0x51 => Some(Op::JumpT),
            0x52 => Some(Op::JumpF),
            0x60 => Some(Op::Call),
            0x61 => Some(Op::CallNative),
            0x62 => Some(Op::Return),
            0x63 => Some(Op::ReturnVoid),
            0x70 => Some(Op::NewStruct),
            0x71 => Some(Op::GetField),
            0x72 => Some(Op::SetField),
            0x78 => Some(Op::NewArray),
            0x79 => Some(Op::ArrayGet),
            0x7A => Some(Op::ArraySet),
            0x7B => Some(Op::ArrayLen),
            0x80 => Some(Op::WrapSome),
            0x81 => Some(Op::WrapOk),
            0x82 => Some(Op::WrapErr),
            0x83 => Some(Op::IsNone),
            0x84 => Some(Op::IsErr),
            0x85 => Some(Op::Unwrap),
            0x90 => Some(Op::MakeClosure),
            0xFF => Some(Op::Nop),
            _    => None,
        }
    }
}
