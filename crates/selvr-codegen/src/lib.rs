// selvr-codegen — Phase 1: transpile Selvr AST → JavaScript.
//
// The output is readable, formatted JS that runs in any modern browser.
// Source maps are emitted in parallel so browser devtools point at `.self` files.
//
// Phase 2 backend (→ bytecode) will live in a separate `bytecode` module.

pub mod js;
pub mod sourcemap;
pub mod error;

pub use js::JsEmitter;
pub use error::CodegenError;
