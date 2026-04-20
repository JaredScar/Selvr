//! `wasm-bindgen` entry points — the public API of the Selvr WASM runtime.
//!
//! The loader script (`runtime/SELVR-loader.js`) calls these functions.
//!
//! ## Exported API
//!
//! | JS call | Description |
//! |---------|-------------|
//! | `SELVR_load(bytes)` | Load a compiled `.vlxc` bytecode module. |
//! | `SELVR_call(name, args_json)` | Call an exported function by name. |
//! | `SELVR_resume(id)` | Resume a suspended async coroutine. |
//! | `SELVR_version()` | Return the runtime version string. |

use wasm_bindgen::prelude::*;
use std::cell::RefCell;

use selvr_bytecode::encode::decode;
use crate::vm::Vm;
use crate::mem::Value;

// ── Global VM instance ────────────────────────────────────────────────────────

thread_local! {
    static VM: RefCell<Option<Vm>> = RefCell::new(None);
}

// ── Entry points ──────────────────────────────────────────────────────────────

/// Load a Selvr bytecode module from a `Uint8Array`.
#[wasm_bindgen]
pub fn SELVR_load(bytes: &[u8]) -> Result<(), JsValue> {
    let module = decode(bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    VM.with(|cell| {
        *cell.borrow_mut() = Some(Vm::new(module));
    });
    Ok(())
}

/// Call an exported Selvr function by name.
///
/// `args_json` is a JSON array of arguments encoded as primitive JS values.
/// Returns a JSON-encoded result or an error string.
#[wasm_bindgen]
pub fn SELVR_call(name: &str, args_json: &str) -> Result<String, JsValue> {
    let args = parse_args(args_json)?;
    VM.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let vm = borrow.as_mut().ok_or_else(|| JsValue::from_str("no module loaded"))?;
        let result = vm.call_by_name(name, args)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(value_to_json(&result, &vm.heap))
    })
}

/// Resume a suspended async coroutine by its numeric ID.
#[wasm_bindgen]
pub fn SELVR_resume(_id: u32) {
    // Hook into the scheduler — see runtime.rs.
}

/// Return the runtime version string.
#[wasm_bindgen]
pub fn SELVR_version() -> String {
    format!("selvr-vm {}", env!("CARGO_PKG_VERSION"))
}

/// Return buffered console output (newline-separated) and clear the buffer.
#[wasm_bindgen]
pub fn SELVR_drain_output() -> String {
    VM.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if let Some(vm) = borrow.as_mut() {
            let out = vm.output.join("\n");
            vm.output.clear();
            out
        } else {
            String::new()
        }
    })
}

// ── Argument / result serialisation ──────────────────────────────────────────

fn parse_args(json: &str) -> Result<Vec<Value>, JsValue> {
    // Minimal JSON array parser for primitive types.
    let json = json.trim();
    if json == "[]" || json.is_empty() { return Ok(vec![]); }

    let inner = json.trim_start_matches('[').trim_end_matches(']').trim();
    if inner.is_empty() { return Ok(vec![]); }

    let mut args = Vec::new();
    for token in split_json_array(inner) {
        let t = token.trim();
        if t == "null" || t == "undefined" {
            args.push(Value::None);
        } else if t == "true" {
            args.push(Value::Bool(true));
        } else if t == "false" {
            args.push(Value::Bool(false));
        } else if t.starts_with('"') {
            let s = t.trim_matches('"').to_string();
            args.push(Value::Str(smol_str::SmolStr::new(s)));
        } else if let Ok(n) = t.parse::<i32>() {
            args.push(Value::I32(n));
        } else if let Ok(f) = t.parse::<f64>() {
            args.push(Value::F64(f));
        }
    }
    Ok(args)
}

fn split_json_array(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth  = 0i32;
    let mut start  = 0;
    for (i, c) in s.char_indices() {
        match c {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            ',' if depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(&s[start..]);
    result
}

fn value_to_json(val: &Value, heap: &crate::mem::Heap) -> String {
    use crate::mem::HeapObj;
    match val {
        Value::I32(n)    => n.to_string(),
        Value::I64(n)    => n.to_string(),
        Value::F64(f)    => f.to_string(),
        Value::Bool(b)   => b.to_string(),
        Value::Str(s)    => format!("\"{}\"", s.replace('"', "\\\"")),
        Value::None      => "null".to_string(),
        Value::Unit      => "null".to_string(),
        Value::Object(i) => match heap.get(*i) {
            Some(HeapObj::Array(elems)) => {
                let items: Vec<String> = elems.iter()
                    .map(|v| value_to_json(v, heap))
                    .collect();
                format!("[{}]", items.join(","))
            }
            Some(HeapObj::Some(inner) | HeapObj::Ok(inner)) => value_to_json(inner, heap),
            Some(HeapObj::Err(inner)) => {
                format!("{{\"__err\":{}}}", value_to_json(inner, heap))
            }
            Some(HeapObj::Struct { fields, .. }) => {
                let pairs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, value_to_json(v, heap)))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            }
            _ => "null".to_string(),
        },
    }
}
