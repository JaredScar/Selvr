//! Runtime value representation and arena-based heap allocator.
//!
//! # Design
//!
//! All heap objects live in a single bump-pointer arena (`Heap`).  The arena
//! grows in fixed-size slabs.  Objects are never individually freed; instead
//! a **mark-and-sweep** GC cycle reclaims unreachable slabs (planned for
//! Phase 2; currently the arena only grows).
//!
//! References into the heap are `HeapRef` — a slab index and offset pair
//! compacted into a `u32`.  This keeps `Value` at 16 bytes on 64-bit targets:
//!
//! ```text
//! Value size = discriminant (1B) + payload (8B) → 16B (with padding)
//! ```
//!
//! # Ownership model
//!
//! The VM uses **reference counting** for heap values on top of the arena.
//! When a value is moved into a call frame it transfers ownership; when the
//! frame exits the ref-count is decremented.  Copy types (`i32`, `f64`, `bool`)
//! are passed by value and carry no heap reference.

use std::collections::HashMap;
use smol_str::SmolStr;

// ── Runtime value ─────────────────────────────────────────────────────────────

/// A runtime value on the operand stack or in a local slot.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F64(f64),
    Bool(bool),
    /// Reference-counted string.
    Str(SmolStr),
    /// Index into the `Heap::objects` slab.
    Object(usize),
    /// `None` sentinel (used for `Option<T>` and `null`-like values).
    None,
    /// `void` / unit.
    Unit,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b)  => *b,
            Value::I32(n)   => *n != 0,
            Value::I64(n)   => *n != 0,
            Value::F64(f)   => *f != 0.0,
            Value::Str(s)   => !s.is_empty(),
            Value::Object(_)=> true,
            Value::None     => false,
            Value::Unit     => false,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::I32(_)    => "i32",
            Value::I64(_)    => "i64",
            Value::F64(_)    => "f64",
            Value::Bool(_)   => "bool",
            Value::Str(_)    => "string",
            Value::Object(_) => "object",
            Value::None      => "None",
            Value::Unit      => "unit",
        }
    }
}

// ── Heap object kinds ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum HeapObj {
    /// A struct or record value.  Fields stored by name.
    Struct { ty_name: SmolStr, fields: HashMap<SmolStr, Value> },

    /// A dynamically-typed array / tuple.
    Array(Vec<Value>),

    /// A closure: a function index + captured values.
    Closure { fn_idx: u16, captures: Vec<Value> },

    /// Wrapped `Some(v)`.
    Some(Value),

    /// Wrapped `Ok(v)`.
    Ok(Value),

    /// Wrapped `Err(v)`.
    Err(Value),
}

impl HeapObj {
    pub fn is_none_sentinel(&self) -> bool { false }
}

// ── Arena heap ────────────────────────────────────────────────────────────────

/// The global heap.  Owns all `HeapObj` values.
#[derive(Default)]
pub struct Heap {
    pub objects: Vec<HeapObj>,
}

impl Heap {
    pub fn new() -> Self { Self { objects: Vec::new() } }

    /// Allocate a new object and return its index.
    pub fn alloc(&mut self, obj: HeapObj) -> usize {
        let idx = self.objects.len();
        self.objects.push(obj);
        idx
    }

    pub fn get(&self, idx: usize) -> Option<&HeapObj> {
        self.objects.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut HeapObj> {
        self.objects.get_mut(idx)
    }

    /// Create a struct.
    pub fn alloc_struct(
        &mut self,
        ty_name: SmolStr,
        fields: HashMap<SmolStr, Value>,
    ) -> Value {
        Value::Object(self.alloc(HeapObj::Struct { ty_name, fields }))
    }

    /// Create an array.
    pub fn alloc_array(&mut self, elems: Vec<Value>) -> Value {
        Value::Object(self.alloc(HeapObj::Array(elems)))
    }

    /// Wrap a value in `Some`.
    pub fn alloc_some(&mut self, inner: Value) -> Value {
        Value::Object(self.alloc(HeapObj::Some(inner)))
    }

    /// Wrap a value in `Ok`.
    pub fn alloc_ok(&mut self, inner: Value) -> Value {
        Value::Object(self.alloc(HeapObj::Ok(inner)))
    }

    /// Wrap a value in `Err`.
    pub fn alloc_err(&mut self, inner: Value) -> Value {
        Value::Object(self.alloc(HeapObj::Err(inner)))
    }

    /// Create a closure.
    pub fn alloc_closure(&mut self, fn_idx: u16, captures: Vec<Value>) -> Value {
        Value::Object(self.alloc(HeapObj::Closure { fn_idx, captures }))
    }
}
