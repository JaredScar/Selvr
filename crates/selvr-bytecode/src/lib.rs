//! Selvr bytecode — format, emitter, optimizer, and binary encoder.
//!
//! Pipeline:
//!   IR  →  (emit)  →  BytecodeModule  →  (opt)  →  (encode)  →  [u8]
//!
//! The bytecode is a compact stack-machine instruction stream stored in a
//! self-contained binary module.  The on-disk layout is:
//!
//!   [magic 6B] [version 2B] [string pool] [fn table] [code sections]
//!
//! See `docs/BYTECODE.md` for the full specification.

pub mod opcode;
pub mod module;
pub mod emit;
pub mod opt;
pub mod encode;
pub mod incr;

pub use module::{BytecodeModule, BcFn, ConstPool, ConstValue};
pub use emit::emit_module;
pub use opt::optimise;
pub use encode::{encode, decode};
pub use incr::IncrementalCache;
