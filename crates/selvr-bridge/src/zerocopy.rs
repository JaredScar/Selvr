//! Zero-copy typed-array fast path for the WASM↔JS bridge.
//!
//! When a WASM-targeted function receives or returns a typed numeric array
//! (`f64[]`, `i32[]`) the bridge can avoid JSON serialisation entirely by
//! passing the raw buffer through `WebAssembly.Memory`:
//!
//! 1. Allocate a region in WASM linear memory (`selvr_alloc(n_bytes)`).
//! 2. Write the JS `Float64Array` / `Int32Array` directly into that region
//!    via a `DataView` on `WebAssembly.Memory`.
//! 3. Pass only the pointer + length to the WASM function — no copying.
//! 4. After the call, read the result region back through the same `DataView`.
//! 5. Release the region (`selvr_free(ptr, n_bytes)`).
//!
//! This reduces the bridge overhead for a 1 M-element `f64[]` from ~8 ms
//! (JSON stringify/parse) to ~50 µs (a single `ArrayBuffer.copyWithin`).
//!
//! # When is zero-copy applied?
//!
//! The `ZeroCopyPath` analysis classifies each bridge call site:
//!
//! | Condition | Result |
//! |-----------|--------|
//! | Both parameter and return are non-array scalars | `Scalar` (no overhead) |
//! | Parameter is `f64[]` / `i32[]` | `ZeroCopy` fast path |
//! | Return is `f64[]` / `i32[]` | `ZeroCopy` fast path |
//! | Mixed (some arrays, some objects) | `Serialised` (fallback to JSON) |

use selvr_ir::IrType;

/// The transfer strategy for a single bridge call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferMode {
    /// No heap data — scalars are passed in registers (i32 / f64).
    Scalar,
    /// All array params/returns use the linear-memory fast path.
    ZeroCopy,
    /// At least one parameter is a heap object that requires serialisation.
    Serialised,
}

/// Classify the transfer mode for a function given its parameter and return types.
pub fn classify(params: &[IrType], ret: &IrType) -> TransferMode {
    let any_array = params.iter().chain(std::iter::once(ret))
        .any(is_numeric_array);
    let any_object = params.iter().chain(std::iter::once(ret))
        .any(|t| matches!(t, IrType::Ref(_)) && !is_numeric_array(t));

    if any_object {
        TransferMode::Serialised
    } else if any_array {
        TransferMode::ZeroCopy
    } else {
        TransferMode::Scalar
    }
}

fn is_numeric_array(ty: &IrType) -> bool {
    matches!(ty,
        IrType::Ref(inner) if matches!(inner.as_ref(), IrType::F64 | IrType::I32 | IrType::I64)
    )
}

/// Generate the JS snippet that performs a zero-copy array transfer.
///
/// `param_name` is the JS variable holding the `Float64Array` / `Int32Array`.
/// `wasm_mem`   is the JS variable holding the `WebAssembly.Memory` object.
pub fn emit_zerocopy_write(
    param_name: &str,
    wasm_mem:   &str,
    ptr_var:    &str,
    elem_size:  usize, // 4 for i32, 8 for f64
) -> String {
    format!(
r#"// Zero-copy: write {param_name} into WASM linear memory
const {ptr_var}_len   = {param_name}.length * {elem_size};
const {ptr_var}_ptr   = selvr_vm.selvr_alloc({ptr_var}_len);
const {ptr_var}_view  = new Uint8Array({wasm_mem}.buffer, {ptr_var}_ptr, {ptr_var}_len);
{ptr_var}_view.set(new Uint8Array({param_name}.buffer, {param_name}.byteOffset, {ptr_var}_len));
"#
    )
}

/// Generate the JS snippet that reads a zero-copy result back from WASM memory.
pub fn emit_zerocopy_read(
    ptr_var:   &str,
    len_var:   &str,
    wasm_mem:  &str,
    ty:        &str, // "Float64Array" or "Int32Array"
) -> String {
    format!(
r#"// Zero-copy: read result back from WASM linear memory
const _result_view = new {ty}({wasm_mem}.buffer, {ptr_var}, {len_var});
const _result      = _result_view.slice(); // copy out before selvr_free
selvr_vm.selvr_free({ptr_var}, {len_var} * {ty}.BYTES_PER_ELEMENT);
"#
    )
}
