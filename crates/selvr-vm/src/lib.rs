//! Selvr stack-machine VM.
//!
//! Modules:
//!   `mem`     — arena allocator + heap value representation.
//!   `vm`      — interpreter loop.
//!   `dom`     — DOM/fetch/timer bindings (stubs on non-WASM targets).
//!   `runtime` — async event loop integration.
//!   `wasm`    — `wasm-bindgen` entry points (WASM targets only).

pub mod mem;
pub mod vm;
pub mod dom;
pub mod runtime;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use vm::{Vm, VmError};
pub use mem::{Heap, Value};
