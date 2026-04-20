//! SL005 — force_target_hint  (Info)
//! SL006 — wasm_dom_call      (Error)
//! SL007 — js_heavy_compute   (Warning)
//!
//! Checks for contradictions between explicit `#[wasm]` / `#[js]` annotations
//! and the compiler's automatic targeting analysis.

use selvr_target::{TargetMap, Target};
#[allow(unused_imports)]
use selvr_target::Target::Auto;
use crate::{LintDiagnostic, LintSpan, Severity};

const WASM_THRESHOLD: i32 = 60;
const DOM_NEGATIVE:   i32 = -100;

pub fn check(_src: &str, map: &TargetMap, out: &mut Vec<LintDiagnostic>) {
    for rec in map.fns.values() {
        let dummy_span = LintSpan { start: 0, end: 0, line: 1, col: 1 };

        if rec.forced {
            match rec.target {
                Target::Wasm if rec.score < DOM_NEGATIVE => {
                    // #[wasm] but calls DOM APIs — that's an error.
                    out.push(LintDiagnostic {
                        code:     "SL006",
                        name:     "wasm_dom_call",
                        severity: Severity::Error,
                        message:  format!(
                            "`{}` is annotated `#[wasm]` but its analysis score ({}) indicates \
                             DOM API calls — WebAssembly cannot call DOM APIs directly",
                            rec.name, rec.score
                        ),
                        span:  dummy_span,
                        fix:   Some("remove `#[wasm]` or move DOM calls to a JS helper function".into()),
                    });
                }
                Target::Js if rec.score >= WASM_THRESHOLD => {
                    // #[js] but looks compute-heavy — warn but don't error.
                    out.push(LintDiagnostic {
                        code:     "SL007",
                        name:     "js_heavy_compute",
                        severity: Severity::Warning,
                        message:  format!(
                            "`{}` is annotated `#[js]` but its analysis score ({}) suggests it \
                             would run significantly faster in WebAssembly",
                            rec.name, rec.score
                        ),
                        span:  dummy_span,
                        fix:   Some("remove `#[js]` to let the compiler auto-target this function".into()),
                    });
                }
                Target::Auto | _ => {
                    // Forced but consistent — emit info hint.
                    out.push(LintDiagnostic {
                        code:     "SL005",
                        name:     "force_target_hint",
                        severity: Severity::Info,
                        message:  format!(
                            "`{}` has an explicit `#[{}]` override (compiler would choose the same target)",
                            rec.name,
                            if rec.target == Target::Wasm { "wasm" } else { "js" }
                        ),
                        span: dummy_span,
                        fix:  Some("you may remove the annotation to let the compiler decide".into()),
                    });
                }
            }
        }
    }
}
