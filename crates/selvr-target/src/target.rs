//! Target annotation types.
//!
//! `Target` itself lives in `selvr-ir` so that the IR can carry the
//! annotation without a circular dependency.  This module re-exports it
//! and adds the richer `FnTarget` / `TargetMap` types used by the analysis.

pub use selvr_ir::Target;

use smol_str::SmolStr;
use indexmap::IndexMap;

// ── Per-function targeting record ─────────────────────────────────────────────

/// All information about how a single function was targeted.
#[derive(Debug, Clone)]
pub struct FnTarget {
    /// The function's mangled name (as it appears in the IR).
    pub name:    SmolStr,
    /// Final assigned target.
    pub target:  Target,
    /// Whether the target was forced by a `#[wasm]` / `#[js]` attribute.
    pub forced:  bool,
    /// Human-readable reason the targeting pass chose this target.
    pub reason:  SmolStr,
    /// Score used by the auto-inference pass (higher = more WASM-like).
    pub score:   i32,
    /// Names of functions called from this one (for call-graph propagation).
    pub callees: Vec<SmolStr>,
}

// ── Module targeting map ──────────────────────────────────────────────────────

/// The complete targeting result for one Selvr module.
#[derive(Debug, Clone, Default)]
pub struct TargetMap {
    pub fns: IndexMap<SmolStr, FnTarget>,
}

impl TargetMap {
    pub fn new() -> Self { Self::default() }

    pub fn insert(&mut self, rec: FnTarget) {
        self.fns.insert(rec.name.clone(), rec);
    }

    pub fn get(&self, name: &str) -> Option<&FnTarget> {
        self.fns.get(name)
    }

    /// (wasm_count, js_count, undecided_count)
    pub fn summary(&self) -> (usize, usize, usize) {
        let wasm = self.fns.values().filter(|r| r.target == Target::Wasm).count();
        let js   = self.fns.values().filter(|r| r.target == Target::Js).count();
        let auto = self.fns.values().filter(|r| r.target == Target::Auto).count();
        (wasm, js, auto)
    }

    /// Render a human-readable split report (matches `selvr explain` output).
    pub fn explain(&self) -> String {
        let mut out = String::from("=== Selvr targeting report ===\n\n");
        for (name, rec) in &self.fns {
            let icon = match rec.target {
                Target::Wasm => "[wasm]",
                Target::Js   => "[js]  ",
                Target::Auto => "[auto]",
            };
            let forced = if rec.forced { " [forced by attribute]" } else { "" };
            out.push_str(&format!(
                "  {icon}  {name}{forced}\n         score: {}  reason: {}\n\n",
                rec.score, rec.reason
            ));
        }
        let (w, j, a) = self.summary();
        out.push_str(&format!("Summary: {w} wasm  {j} js  {a} undecided\n"));
        out
    }

    /// Render a machine-readable JSON split report.
    pub fn to_json(&self) -> String {
        let (w, j, _) = self.summary();
        let entries: Vec<String> = self.fns.values().map(|rec| {
            format!(
                "    {{\"fn\":\"{}\",\"target\":\"{}\",\"score\":{},\"forced\":{},\"reason\":\"{}\"}}",
                rec.name, rec.target, rec.score, rec.forced,
                rec.reason.replace('"', "\\\"")
            )
        }).collect();
        format!(
            "{{\n  \"summary\":{{\"wasm\":{w},\"js\":{j}}},\n  \"functions\":[\n{}\n  ]\n}}",
            entries.join(",\n")
        )
    }
}
