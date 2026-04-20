//! Bytecode module structure — the in-memory representation of a compiled `.self` file.

use smol_str::SmolStr;

// ── Constant pool ─────────────────────────────────────────────────────────────

/// A compile-time constant that can be referenced by index in instructions.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Str(SmolStr),
    /// Interned struct/field name (stored separately for O(1) field access).
    Name(SmolStr),
}

/// The constant pool (string table) for a module.
#[derive(Debug, Clone, Default)]
pub struct ConstPool {
    pub entries: Vec<ConstValue>,
}

impl ConstPool {
    pub fn new() -> Self { Self { entries: Vec::new() } }

    /// Intern a string, returning its index.  Deduplicates.
    pub fn intern_str(&mut self, s: &str) -> u16 {
        for (i, e) in self.entries.iter().enumerate() {
            if let ConstValue::Str(existing) = e {
                if existing.as_str() == s { return i as u16; }
            }
        }
        let idx = self.entries.len() as u16;
        self.entries.push(ConstValue::Str(SmolStr::new(s)));
        idx
    }

    /// Intern a name (used for struct/field names in `NewStruct`, `GetField`, `SetField`).
    pub fn intern_name(&mut self, s: &str) -> u16 {
        for (i, e) in self.entries.iter().enumerate() {
            if let ConstValue::Name(existing) = e {
                if existing.as_str() == s { return i as u16; }
            }
        }
        let idx = self.entries.len() as u16;
        self.entries.push(ConstValue::Name(SmolStr::new(s)));
        idx
    }

    pub fn get(&self, idx: u16) -> Option<&ConstValue> {
        self.entries.get(idx as usize)
    }
}

// ── Function record ───────────────────────────────────────────────────────────

/// A compiled function inside a bytecode module.
#[derive(Debug, Clone)]
pub struct BcFn {
    /// Interned name index in the constant pool.
    pub name_idx:   u16,
    /// Number of parameters (first `param_count` locals = parameters).
    pub param_count: u8,
    /// Total locals (including parameters and temporaries).
    pub local_count: u16,
    /// Linear bytecode stream.
    pub code:        Vec<u8>,
    /// Whether this function is exported (callable from JS).
    pub is_export:   bool,
    /// Whether this function is async.
    pub is_async:    bool,
}

// ── Module ────────────────────────────────────────────────────────────────────

/// Magic bytes that start every Selvr bytecode file.
pub const MAGIC: &[u8; 6] = b"SELVR\x01";
/// Bytecode format version (major, minor).
pub const VERSION: (u8, u8) = (0, 1);

/// A compiled Selvr module ready for the VM or binary serialisation.
#[derive(Debug, Clone)]
pub struct BytecodeModule {
    pub const_pool: ConstPool,
    pub fns:        Vec<BcFn>,
    /// Source file name (for stack traces).
    pub source:     SmolStr,
    /// Content hash of the source used for incremental caching.
    pub src_hash:   [u8; 32],
}

impl BytecodeModule {
    pub fn new(source: impl Into<SmolStr>) -> Self {
        Self {
            const_pool: ConstPool::new(),
            fns: Vec::new(),
            source: source.into(),
            src_hash: [0u8; 32],
        }
    }

    /// Find a function by name.
    pub fn find_fn(&self, name: &str) -> Option<&BcFn> {
        let pool = &self.const_pool;
        self.fns.iter().find(|f| {
            pool.get(f.name_idx)
                .and_then(|e| if let ConstValue::Str(s) | ConstValue::Name(s) = e { Some(s.as_str()) } else { None })
                == Some(name)
        })
    }
}
