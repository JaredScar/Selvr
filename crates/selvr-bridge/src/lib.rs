//! Selvr WASM/JS bridge code generator.
//!
//! After the targeting pass assigns each function to a runtime, this crate
//! generates the **glue code** that lets the two halves call each other
//! transparently.
//!
//! # Generated artefacts
//!
//! Given the targeting map, the bridge emitter produces three text outputs:
//!
//! 1. **`app.js`** — includes:
//!    - All JS-targeted functions (transpiled from the IR by `selvr-codegen`).
//!    - A JS wrapper for every WASM-targeted export: serialises arguments,
//!      calls `selvr_vm.call()`, and deserialises the return value.
//!    - A JS import stub for every JS function called *from* WASM.
//!
//! 2. **`app.wasm`** — all WASM-targeted functions compiled to bytecode.
//!
//! 3. **`app.loader.js`** — bootstraps both, exposes the unified API:
//!    ```js
//!    await Selvr.load("app.wasm");
//!    const result = await Selvr.call("blur", pixels, 5);
//!    ```
//!
//! # Zero-copy fast path
//!
//! When a WASM function's parameter type is `f64[]` or `i32[]` and the
//! corresponding JS caller already has a `Float64Array` / `Int32Array`, the
//! bridge skips JSON serialisation and passes the typed array's buffer
//! directly to WASM via `WebAssembly.Memory`.  This drops bridge overhead
//! for large numeric arrays from O(n) to O(1).

pub mod codegen;
pub mod zerocopy;

pub use codegen::{BridgeEmitter, BridgeOutput};
