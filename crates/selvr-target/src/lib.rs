//! Selvr compile-time WASM/JS targeting pass.
//!
//! The targeting pass is the core innovation of the Selvr compiler:
//!
//!   1. **Infer** — score every function in the IR module with `infer_targets`.
//!      Each function gets a `Target::Wasm` or `Target::Js` annotation based on
//!      heuristics (numeric loop density, DOM API calls, `#[wasm]`/`#[js]` attrs).
//!   2. **Propagate** — run `propagate_targets` to enforce call-graph consistency.
//!   3. **Emit** — downstream crates use `IrFn.target` to route functions to the
//!      right backend.
//!
//! ## Quick usage
//! ```rust,ignore
//! let mut ir = selvr_ir::lower_module(&ast_module);
//! let mut map = selvr_target::infer_targets(&mut ir);
//! let changed = selvr_target::propagate_targets(&mut ir, &mut map);
//! println!("{}", map.explain());
//! ```

pub mod target;
pub mod infer;
pub mod propagate;

pub use target::{TargetMap, FnTarget};
// Re-export Target from selvr-ir for callers who only depend on selvr-target.
pub use selvr_ir::Target;
pub use infer::infer_targets;
pub use propagate::propagate_targets;
