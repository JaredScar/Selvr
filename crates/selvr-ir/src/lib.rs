//! Selvr mid-level Intermediate Representation (IR).
//!
//! The IR sits between the AST and bytecode:
//!
//!   AST  →  (lower)  →  IR  →  (emit)  →  Bytecode
//!
//! The IR uses a Control-Flow Graph (CFG) with basic blocks of three-address
//! instructions.  Each function is in SSA-like form: locals are numbered
//! (`IrLocal`) and definitions dominate uses within the function.
//!
//! The IR is intentionally *not* typed beyond what is needed for code
//! generation.  The type checker has already validated the program; the IR
//! just needs to know if a value is a reference vs. an immediate.

pub mod ir;
pub mod lower;

pub use ir::*;
pub use lower::lower_module;
