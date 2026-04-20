//! Automatic WASM/JS target inference.
//!
//! # Scoring rubric
//!
//! ## WASM signals (positive)
//!
//! | Signal | Score |
//! |--------|-------|
//! | Returns `f64[]` or `i32[]` | +40 |
//! | Annotated `#[wasm]` | +1000 (forces) |
//! | Calls `Math.*` ≥ 3 times | +25 |
//! | Multiply in a loop body (FMA pattern) | +30 |
//! | ≥ 10 numeric ops in a loop body | +50 |
//! | Function is `async` (heuristic: async helpers tend to be compute) | +5 |
//!
//! ## JS signals (negative)
//!
//! | Signal | Score |
//! |--------|-------|
//! | Calls a DOM API | -1000 (forces) |
//! | Annotated `#[js]` | -1000 (forces) |
//! | Calls `console.log` | -10 |
//!
//! ## Tie-breaking
//!
//! Score ≥ 50 → `Target::Wasm`; otherwise → `Target::Js` (safer default).

use smol_str::SmolStr;
use selvr_ir::{IrModule, IrFn, IrType, Instr, Value, Constant, BinOp, Terminator, Target};
use crate::target::{FnTarget, TargetMap};

const WASM_THRESHOLD: i32 = 50;

/// Infer and annotate targets for every function in `module`.
///
/// Mutates each `IrFn.target` in place **and** returns a `TargetMap` with
/// the full scoring details for diagnostics / `selvr explain`.
pub fn infer_targets(module: &mut IrModule) -> TargetMap {
    let mut map = TargetMap::new();
    for f in &mut module.fns {
        let rec = score_fn(f);
        f.target = rec.target;
        map.insert(rec);
    }
    map
}

// ── Per-function scoring ─────────────────────────────────────────────────────

fn score_fn(f: &IrFn) -> FnTarget {
    // Check explicit developer override from `#[wasm]` / `#[js]` attributes.
    for attr in &f.attrs {
        match attr.as_str() {
            "wasm" => return forced(f, Target::Wasm, "forced by #[wasm] attribute", 1000),
            "js"   => return forced(f, Target::Js,   "forced by #[js] attribute",   -1000),
            _      => {}
        }
    }

    let mut score: i32 = 0;
    let mut reasons: Vec<&'static str> = Vec::new();

    // Async event handler pattern (named `on*` or `handle*`).
    if f.is_async && (f.name.starts_with("on") || f.name.starts_with("handle")) {
        score -= 200;
        reasons.push("async event handler naming pattern");
    }

    let mut math_calls:  usize = 0;
    let mut dom_call           = false;
    let mut mul_in_loop        = false;
    let mut numeric_in_loop:   usize = 0;

    for (bb_idx, bb) in f.blocks.iter().enumerate() {
        let in_loop = is_loop_body(bb_idx, f);
        if in_loop {
            for instr in &bb.instrs {
                match instr {
                    Instr::BinOp { op: BinOp::Mul, .. } => {
                        mul_in_loop = true;
                        numeric_in_loop += 1;
                    }
                    Instr::BinOp { op: BinOp::Add | BinOp::Sub | BinOp::Div | BinOp::Rem, .. } => {
                        numeric_in_loop += 1;
                    }
                    _ => {}
                }
            }
        }

        for instr in &bb.instrs {
            if let Instr::Call { func, .. } = instr {
                match func {
                    Value::Global(name) | Value::Const(Constant::Str(name)) => {
                        if is_math_call(name.as_str())  { math_calls += 1; }
                        if is_dom_call(name.as_str())   { dom_call = true; }
                        if name.as_str() == "console.log" { score -= 10; }
                    }
                    _ => {}
                }
            }
        }
    }

    // DOM call instantly forces JS.
    if dom_call {
        score -= 1000;
        reasons.push("calls a DOM API");
    }

    if math_calls >= 3 {
        score += 25;
        reasons.push("calls Math.* 3+ times");
    } else if math_calls > 0 {
        score += math_calls as i32 * 6;
    }

    if mul_in_loop {
        score += 30;
        reasons.push("multiply inside loop body (FMA pattern)");
    }

    if numeric_in_loop >= 10 {
        score += 50;
        reasons.push(">= 10 numeric ops inside loop body");
    } else if numeric_in_loop > 0 {
        score += numeric_in_loop as i32 * 4;
    }

    if is_numeric_array_ty(&f.ret_ty) {
        score += 40;
        reasons.push("returns a numeric array type");
    }

    let target = if score >= WASM_THRESHOLD { Target::Wasm } else { Target::Js };

    let reason = if reasons.is_empty() {
        if target == Target::Wasm {
            SmolStr::new("score above WASM threshold")
        } else {
            SmolStr::new("default: JS (score below WASM threshold)")
        }
    } else {
        SmolStr::new(reasons.join("; "))
    };

    FnTarget {
        name:    f.name.clone(),
        target,
        forced:  false,
        reason,
        score,
        callees: collect_callees(f),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn forced(f: &IrFn, target: Target, reason: &'static str, score: i32) -> FnTarget {
    FnTarget {
        name:    f.name.clone(),
        target,
        forced:  true,
        reason:  SmolStr::new(reason),
        score,
        callees: collect_callees(f),
    }
}

fn is_math_call(name: &str) -> bool {
    name.starts_with("Math.") || matches!(name,
        "sqrt" | "abs" | "floor" | "ceil" | "round" | "pow"
        | "sin" | "cos" | "tan" | "exp" | "log" | "log2" | "log10"
        | "min" | "max" | "clamp"
    )
}

fn is_dom_call(name: &str) -> bool {
    name.starts_with("document.")
    || name.starts_with("window.")
    || name == "addEventListener"
    || name == "querySelector"
    || name == "getElementById"
    || name.starts_with("__method_addEventListener")
    || name.starts_with("__method_querySelector")
    || name.starts_with("__method_getElementById")
}

/// Rough back-edge detection: block `bb_idx` is inside a loop if its
/// terminator targets an earlier block.
fn is_loop_body(bb_idx: usize, f: &IrFn) -> bool {
    match &f.blocks[bb_idx].term {
        Terminator::Jump(t)       if (t.0 as usize) < bb_idx => true,
        Terminator::Branch { then_bb, .. } if (then_bb.0 as usize) < bb_idx => true,
        Terminator::Branch { else_bb, .. } if (else_bb.0 as usize) < bb_idx => true,
        _ => false,
    }
}

fn is_numeric_array_ty(ty: &IrType) -> bool {
    matches!(ty, IrType::Ref(inner)
        if matches!(inner.as_ref(), IrType::F64 | IrType::I32 | IrType::I64))
}

fn collect_callees(f: &IrFn) -> Vec<SmolStr> {
    let mut out: Vec<SmolStr> = Vec::new();
    for bb in &f.blocks {
        for instr in &bb.instrs {
            if let Instr::Call { func: Value::Global(name), .. } = instr {
                out.push(name.clone());
            }
        }
    }
    out.sort();
    out.dedup();
    out
}
