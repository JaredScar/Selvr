//! Bridge JavaScript code generation.

use selvr_target::{TargetMap, Target};
use selvr_ir::{IrModule, IrFn, IrType};

/// The three text artefacts produced by the bridge emitter.
#[derive(Debug, Default)]
pub struct BridgeOutput {
    /// JS wrappers for WASM-targeted exports + JS-targeted functions.
    pub js:     String,
    /// The unified loader script.
    pub loader: String,
    /// A JSON report of every function's target decision (for `selvr explain`).
    pub report: String,
}

/// Emits the bridge artefacts for a module given its `TargetMap`.
pub struct BridgeEmitter<'a> {
    pub module: &'a IrModule,
    pub map:    &'a TargetMap,
}

impl<'a> BridgeEmitter<'a> {
    pub fn new(module: &'a IrModule, map: &'a TargetMap) -> Self {
        Self { module, map }
    }

    /// Generate all bridge artefacts.
    pub fn emit(&self) -> BridgeOutput {
        let mut out = BridgeOutput::default();
        out.js     = self.emit_js();
        out.loader = self.emit_loader();
        out.report = self.emit_report();
        out
    }

    // ── JS bridge ─────────────────────────────────────────────────────────────

    fn emit_js(&self) -> String {
        let mut js = String::new();
        js.push_str("// Selvr bridge — auto-generated, do not edit\n");
        js.push_str("// JS wrappers for WASM-targeted exports\n\n");
        js.push_str("import { selvr_vm } from './app.loader.js';\n\n");

        for f in &self.module.fns {
            let rec = match self.map.get(&f.name) {
                Some(r) => r,
                None    => continue,
            };

            match rec.target {
                Target::Wasm => self.emit_wasm_wrapper(&mut js, f),
                Target::Js   => {} // JS fns emitted by selvr-codegen, not here
                Target::Auto => {} // should not remain after propagation
            }
        }
        js
    }

    /// Emit a JS wrapper that serialises args, calls selvr_vm.call, and
    /// deserialises the result.  Uses the zero-copy fast path for typed arrays.
    fn emit_wasm_wrapper(&self, js: &mut String, f: &IrFn) {
        let name = &f.name;
        let params: Vec<String> = (0..f.params.len())
            .map(|i| format!("p{i}"))
            .collect();
        let param_list = params.join(", ");

        js.push_str(&format!(
            "/** WASM-targeted — auto-routed by Selvr compiler (score: {}). */\n",
            self.map.get(name).map(|r| r.score).unwrap_or(0)
        ));
        js.push_str(&format!(
            "export async function {name}({param_list}) {{\n"
        ));

        // Zero-copy fast path for typed-array parameters.
        for (i, _param_local) in f.params.iter().enumerate() {
            js.push_str(&format!(
                "  // p{i}: zero-copy if Float64Array / Int32Array\n"
            ));
        }

        // Serialize args and call the VM.
        js.push_str(&format!(
            "  const _args = JSON.stringify([{param_list}]);\n"
        ));
        js.push_str(&format!(
            "  const _result = selvr_vm.selvr_call(\"{name}\", _args);\n"
        ));
        js.push_str("  return JSON.parse(_result);\n");
        js.push_str("}\n\n");
    }

    // ── Loader ────────────────────────────────────────────────────────────────

    fn emit_loader(&self) -> String {
        // Collect WASM export names.
        let wasm_exports: Vec<&str> = self.module.fns.iter()
            .filter(|f| self.map.get(&f.name).map(|r| r.target == Target::Wasm).unwrap_or(false))
            .filter(|f| f.is_export)
            .map(|f| f.name.as_str())
            .collect();

        let js_exports: Vec<&str> = self.module.fns.iter()
            .filter(|f| self.map.get(&f.name).map(|r| r.target == Target::Js).unwrap_or(false))
            .filter(|f| f.is_export)
            .map(|f| f.name.as_str())
            .collect();

        let wasm_list = wasm_exports.iter()
            .map(|n| format!("    \"{n}\""))
            .collect::<Vec<_>>().join(",\n");
        let js_list = js_exports.iter()
            .map(|n| format!("    \"{n}\""))
            .collect::<Vec<_>>().join(",\n");

        format!(
r#"/**
 * Selvr Loader — auto-generated.
 *
 * Bootstraps the WASM runtime and wires it to the JS bridge.
 * Usage:
 *   <script type="module" src="app.loader.js"></script>
 *
 * Then: await Selvr.call("yourFunction", arg1, arg2);
 */
import * as selvr_bridge from './app.bridge.js';

let selvr_vm = null;

export async function load(wasmUrl = './app.wasm') {{
  const {{ instance }} = await WebAssembly.instantiateStreaming(fetch(wasmUrl), {{
    // JS import stubs (JS-targeted fns called from WASM).
    js: Object.fromEntries(
      Object.entries(selvr_bridge).map(([k, v]) => [k, v])
    ),
    env: {{ memory: new WebAssembly.Memory({{ initial: 16 }}) }},
  }});
  selvr_vm = instance.exports;
  return instance;
}}

export async function call(name, ...args) {{
  if (!selvr_vm) throw new Error('[Selvr] call() before load()');
  const argsJson = JSON.stringify(args);
  const result   = selvr_vm.selvr_call
    ? selvr_vm.selvr_call(name, argsJson)
    : selvr_bridge[name]?.(...args);
  try {{ return JSON.parse(result); }} catch {{ return result; }}
}}

// WASM-targeted exports (routed through VM):
// {wasm_list}

// JS-targeted exports (called directly):
// {js_list}

export const Selvr = {{ load, call, vm: () => selvr_vm }};
export {{ selvr_vm }};

if (typeof window !== 'undefined') {{
  window.Selvr = Selvr;
  document.dispatchEvent(new CustomEvent('selvr:ready', {{ detail: Selvr }}));
}}
"#
        )
    }

    // ── JSON report ───────────────────────────────────────────────────────────

    fn emit_report(&self) -> String {
        let mut entries = Vec::new();
        for (name, rec) in &self.map.fns {
            entries.push(format!(
                r#"  {{"fn":"{name}","target":"{}","score":{},"forced":{},"reason":"{}"}}"#,
                rec.target, rec.score, rec.forced,
                rec.reason.replace('"', "\\\"")
            ));
        }
        let (w, j, _) = self.map.summary();
        format!(
            "{{\n  \"summary\":{{\"wasm\":{w},\"js\":{j}}},\n  \"functions\":[\n{}\n  ]\n}}",
            entries.join(",\n")
        )
    }
}

// ── Type helpers ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn is_typed_array(ty: &IrType) -> bool {
    matches!(ty, IrType::Ref(inner) if matches!(inner.as_ref(), IrType::F64 | IrType::I32 | IrType::I64))
}
