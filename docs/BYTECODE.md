# Selvr Bytecode Specification

**[Read as a web page →](bytecode.html)**

**Format version:** 0.1  
**Status:** Draft — subject to change before 1.0.

---

## Overview

Selvr compiles `.self` source files to a compact binary format called **Selvr Bytecode** (`.vlxc`).  A `.vlxc` file is a self-contained, position-independent module that can be loaded and executed by the Selvr VM (`selvr-vm`) compiled to WebAssembly.

The design priorities are:

| Priority | Rationale |
|----------|-----------|
| **Small size** | Faster network transfer than equivalent JS bundles |
| **Fast decode** | Linear scan; no tree traversal at startup |
| **Simple VM** | Stack machine; no register allocation needed |
| **Safe** | All array accesses bounds-checked; no raw pointers |

---

## File Layout

```
┌─────────────────────────────────────────────────┐
│  Magic     6 bytes  b"SELVR\x01"                │
│  Version   2 bytes  u8 major, u8 minor          │
│  Source    2+N bytes  u16 length + UTF-8 name   │
│  Hash      32 bytes   SHA-256 of source         │
├─────────────────────────────────────────────────┤
│  Constant pool                                  │
│    count   2 bytes  u16                         │
│    entries …                                    │
│      tag   1 byte   0=Str  1=Name               │
│      len   2 bytes  u16                         │
│      data  N bytes  UTF-8                       │
├─────────────────────────────────────────────────┤
│  Function table                                 │
│    count   2 bytes  u16                         │
│    records …                                    │
│      name_idx    2 bytes  u16 (pool index)      │
│      param_count 1 byte   u8                    │
│      local_count 2 bytes  u16                   │
│      flags       1 byte   bit0=export bit1=async│
│      code_len    4 bytes  u32                   │
│      code        N bytes  instruction stream    │
└─────────────────────────────────────────────────┘
```

All multi-byte integers are **little-endian**.

---

## Constant Pool

The constant pool stores all string and name literals referenced by the code.  Strings are deduplicated: each unique value appears at most once.

Two entry types exist:

| Tag | Kind | Used by |
|-----|------|---------|
| `0` | `Str`  | `PushStr`, `LoadGlobal`, `StoreGlobal` |
| `1` | `Name` | `NewStruct`, `GetField`, `SetField` |

Pool indices are `u16` (max 65 535 entries per module).

---

## Instruction Set

Each instruction is 1–9 bytes: opcode (`u8`) followed by zero or more immediate operands.

### Constants

| Opcode | Hex  | Immediates | Description |
|--------|------|-----------|-------------|
| `PushI32`  | `0x01` | `i32` (4 B) | Push 32-bit signed integer |
| `PushI64`  | `0x02` | `i64` (8 B) | Push 64-bit signed integer |
| `PushF64`  | `0x03` | `f64` (8 B) | Push double-precision float |
| `PushBool` | `0x04` | `u8`  (1 B) | Push boolean (`0`=false `1`=true) |
| `PushStr`  | `0x05` | `u16` (2 B) | Push string from pool[idx] |
| `PushNone` | `0x06` | `u16` (2 B, reserved) | Push `None` |
| `PushUnit` | `0x07` | `u16` (2 B, reserved) | Push unit/void |

### Locals & Globals

| Opcode | Hex  | Immediates | Description |
|--------|------|-----------|-------------|
| `LoadLocal`   | `0x10` | `u16` local index | Push local slot |
| `StoreLocal`  | `0x11` | `u16` local index | Pop into local slot |
| `LoadGlobal`  | `0x12` | `u16` pool index  | Push global (function or var) |
| `StoreGlobal` | `0x13` | `u16` pool index  | Pop into global variable |

### Stack Manipulation

| Opcode | Hex  | Description |
|--------|------|-------------|
| `Pop`  | `0x20` | Discard TOS |
| `Dup`  | `0x21` | Duplicate TOS |
| `Swap` | `0x22` | Swap top two values |

### Arithmetic

All operators consume two operands and produce one result.  Operand types must match at runtime (the type checker ensures this statically).

| Opcode | Hex  | Operation |
|--------|------|-----------|
| `Add`  | `0x30` | `a + b`  (i32, i64, f64, or string concat) |
| `Sub`  | `0x31` | `a - b` |
| `Mul`  | `0x32` | `a * b` |
| `Div`  | `0x33` | `a / b`  (panics on integer divide-by-zero) |
| `Rem`  | `0x34` | `a % b` |
| `Neg`  | `0x35` | `-a` (unary) |

### Bitwise

| Opcode  | Hex  | Operation |
|---------|------|-----------|
| `BitAnd` | `0x38` | `a & b` |
| `BitOr`  | `0x39` | `a \| b` |
| `BitXor` | `0x3A` | `a ^ b` |
| `Shl`    | `0x3B` | `a << b` |
| `Shr`    | `0x3C` | `a >> b` (arithmetic) |

### Comparison

All produce a `bool` result.

| Opcode | Hex  | Operation |
|--------|------|-----------|
| `Eq`   | `0x40` | `a == b` |
| `Ne`   | `0x41` | `a != b` |
| `Lt`   | `0x42` | `a <  b` |
| `Le`   | `0x43` | `a <= b` |
| `Gt`   | `0x44` | `a >  b` |
| `Ge`   | `0x45` | `a >= b` |

### Boolean

| Opcode | Hex  | Operation |
|--------|------|-----------|
| `And`  | `0x48` | `a && b` (non-short-circuit at bytecode level) |
| `Or`   | `0x49` | `a \|\| b` |
| `Not`  | `0x4A` | `!a` |

### Control Flow

Jump offsets are **signed 32-bit** values relative to the byte immediately *after* the 4-byte immediate.

| Opcode  | Hex  | Immediates | Description |
|---------|------|-----------|-------------|
| `Jump`  | `0x50` | `i32` offset | Unconditional jump |
| `JumpT` | `0x51` | `i32` offset | Jump if TOS is truthy; pops TOS |
| `JumpF` | `0x52` | `i32` offset | Jump if TOS is falsy; pops TOS |

### Calls & Returns

| Opcode       | Hex  | Immediates | Description |
|--------------|------|-----------|-------------|
| `Call`       | `0x60` | `u8` arity | Call function on TOS with `arity` args below it |
| `CallNative` | `0x61` | `u16` name-idx, `u8` arity | Call built-in function |
| `Return`     | `0x62` | — | Return TOS to caller |
| `ReturnVoid` | `0x63` | — | Return unit to caller |

**Call convention:**  Arguments are pushed left-to-right, then the function value, then `Call arity`.  On entry the first `param_count` locals are initialised from the stack (args popped right-to-left).

### Objects & Fields

| Opcode      | Hex  | Immediates | Description |
|-------------|------|-----------|-------------|
| `NewStruct` | `0x70` | `u16` type-name-idx, `u16` field-count | Pop N field values, allocate struct |
| `GetField`  | `0x71` | `u16` field-name-idx | Pop object, push field value |
| `SetField`  | `0x72` | `u16` field-name-idx | Pop object and value, set field |

### Arrays

| Opcode     | Hex  | Immediates | Description |
|------------|------|-----------|-------------|
| `NewArray` | `0x78` | `u16` count | Pop N elements, push array |
| `ArrayGet` | `0x79` | — | Pop array and index, push element |
| `ArraySet` | `0x7A` | — | Pop array, index, value; set element |
| `ArrayLen` | `0x7B` | — | Pop array, push length (`i32`) |

### Option & Result

| Opcode    | Hex  | Description |
|-----------|------|-------------|
| `WrapSome` | `0x80` | Pop value, push `Some(value)` |
| `WrapOk`   | `0x81` | Pop value, push `Ok(value)` |
| `WrapErr`  | `0x82` | Pop value, push `Err(value)` |
| `IsNone`   | `0x83` | Pop value, push `true` if `None` |
| `IsErr`    | `0x84` | Pop value, push `true` if `Err(_)` |
| `Unwrap`   | `0x85` | Pop `Some`/`Ok`, push inner value; panic on `None`/`Err` |

### Closures

| Opcode        | Hex  | Immediates | Description |
|---------------|------|-----------|-------------|
| `MakeClosure` | `0x90` | `u16` fn-idx, `u8` capture-count | Pop N captured locals, push closure |

### Miscellaneous

| Opcode | Hex  | Description |
|--------|------|-------------|
| `Nop`  | `0xFF` | No operation |

---

## Value Representation

At runtime the VM uses a tagged-union `Value` type:

| Tag | Rust variant | Size |
|-----|-------------|------|
| `I32`    | `Value::I32(i32)`    | 8 B |
| `I64`    | `Value::I64(i64)`    | 16 B |
| `F64`    | `Value::F64(f64)`    | 16 B |
| `Bool`   | `Value::Bool(bool)`  | 8 B |
| `Str`    | `Value::Str(SmolStr)` | 32 B (stack-allocated ≤23 chars) |
| `Object` | `Value::Object(usize)` — heap index | 16 B |
| `None`   | `Value::None`        | 8 B |
| `Unit`   | `Value::Unit`        | 8 B |

Heap objects (`struct`, `array`, `closure`, `Some`, `Ok`, `Err`) live in a bump-pointer arena (`Heap`) and are referenced by index.

---

## Incremental Compilation

The CLI caches compiled modules by the **SHA-256 content hash** of the source file in `~/.SELVR/cache/<hex>.vlxc`.  On a subsequent build, if the source hash matches a cached artifact the compilation is skipped entirely.

---

## Optimization Passes

The following passes run over the linear bytecode after emission:

1. **Nop removal** — strips all `0xFF` bytes.
2. **Dead code elimination** — removes instructions after unconditional `Jump`/`Return`.
3. **Constant folding** — `PushI32 a  PushI32 b  <binop>` → `PushI32 (a op b)` for arithmetic and comparison operators.  Same for `F64`.

More advanced passes (inlining, escape analysis) are planned for Phase 3.

---

## Future Work

- **JIT compilation** — translate hot bytecode paths to native code via Cranelift.
- **SIMD opcodes** — vectorised arithmetic on `[f64; N]` arrays.
- **Source maps** — map bytecode offsets back to `.self` source locations.
- **Async coroutine state serialisation** — suspend/resume across page reloads.
