//! Call-graph propagation of targeting decisions.
//!
//! After initial inference, the propagator enforces two rules to a fixed point:
//!
//! 1. **Downgrade**: a WASM function that calls a JS-only function is
//!    downgraded to JS — keeping it in WASM would require a bridge crossing
//!    on every call.
//!
//! 2. **Upgrade**: a JS function whose *entire* caller set is WASM, and whose
//!    own inference score was positive, is upgraded to WASM — this removes
//!    an otherwise-unnecessary bridge crossing.
//!
//! The propagator also mutates `IrFn.target` in the module to keep the IR
//! and the `TargetMap` in sync.

use std::collections::{HashMap, VecDeque};
use smol_str::SmolStr;
use selvr_ir::{IrModule, Target};
use crate::target::TargetMap;

/// Propagate through the call graph until stable.
///
/// Mutates both `module.fns[*].target` and the `TargetMap` entries.
/// Returns the names of functions whose targets changed.
pub fn propagate_targets(module: &mut IrModule, map: &mut TargetMap) -> Vec<SmolStr> {
    let mut changed: Vec<SmolStr> = Vec::new();

    // Build callee → callers reverse index.
    let callee_to_callers: HashMap<SmolStr, Vec<SmolStr>> = {
        let mut idx: HashMap<SmolStr, Vec<SmolStr>> = HashMap::new();
        for rec in map.fns.values() {
            for callee in &rec.callees {
                idx.entry(callee.clone()).or_default().push(rec.name.clone());
            }
        }
        idx
    };

    // Build name → index into module.fns for fast mutation.
    let fn_index: HashMap<SmolStr, usize> = module.fns.iter().enumerate()
        .map(|(i, f)| (f.name.clone(), i))
        .collect();

    let mut queue: VecDeque<SmolStr> = map.fns.keys().cloned().collect();
    let mut in_queue: HashMap<SmolStr, bool> = map.fns.keys()
        .map(|k| (k.clone(), true))
        .collect();

    while let Some(fn_name) = queue.pop_front() {
        in_queue.insert(fn_name.clone(), false);

        let rec = match map.fns.get(&fn_name) {
            Some(r) => r.clone(),
            None    => continue,
        };

        if rec.forced { continue; }

        // Rule 1: downgrade WASM → JS if any callee is JS-only.
        let has_js_callee = rec.callees.iter().any(|c| {
            map.fns.get(c.as_str()).map(|r| r.target == Target::Js).unwrap_or(false)
        });

        if has_js_callee && rec.target == Target::Wasm {
            update(fn_name.clone(), Target::Js,
                "downgraded: calls a JS-only function",
                map, module, &fn_index, &mut changed);

            // Re-queue callers.
            for caller in callee_to_callers.get(&fn_name).into_iter().flatten() {
                if !in_queue.get(caller).copied().unwrap_or(false) {
                    queue.push_back(caller.clone());
                    in_queue.insert(caller.clone(), true);
                }
            }
            continue;
        }

        // Rule 2: upgrade JS → WASM if all callers are WASM and score > 0.
        let callers = callee_to_callers.get(&fn_name);
        let all_callers_wasm = callers.map(|cs| {
            !cs.is_empty() && cs.iter().all(|c| {
                map.fns.get(c.as_str()).map(|r| r.target == Target::Wasm).unwrap_or(false)
            })
        }).unwrap_or(false);

        if all_callers_wasm && rec.target == Target::Js && rec.score > 0 {
            update(fn_name.clone(), Target::Wasm,
                "upgraded: all callers are WASM and score is positive",
                map, module, &fn_index, &mut changed);
        }
    }

    changed.sort();
    changed.dedup();
    changed
}

fn update(
    name:     SmolStr,
    target:   Target,
    reason:   &'static str,
    map:      &mut TargetMap,
    module:   &mut IrModule,
    fn_index: &HashMap<SmolStr, usize>,
    changed:  &mut Vec<SmolStr>,
) {
    if let Some(rec) = map.fns.get_mut(&name) {
        rec.target = target;
        rec.reason = smol_str::SmolStr::new(reason);
    }
    if let Some(&idx) = fn_index.get(&name) {
        module.fns[idx].target = target;
    }
    changed.push(name);
}
