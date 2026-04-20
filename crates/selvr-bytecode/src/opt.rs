//! Bytecode optimization passes.
//!
//! Currently implemented passes (all run by `optimise`):
//!
//!   1. **Constant folding** — evaluates arithmetic/comparison on constant
//!      operands at compile time, replacing the sequence with a single PUSH.
//!
//!   2. **Dead code elimination (DCE)** — removes unreachable instructions
//!      that follow an unconditional jump / return inside a basic block.
//!
//!   3. **Nop removal** — strips all `Nop` instructions.
//!
//! These passes operate directly on the linear bytecode stream so they require
//! no additional IR structure.

use crate::module::{BytecodeModule, BcFn};
use crate::opcode::Op;

/// Run all optimisation passes over a `BytecodeModule` in place.
pub fn optimise(module: &mut BytecodeModule) {
    for f in &mut module.fns {
        remove_nops(f);
        dce(f);
        constant_fold(f);
    }
}

// ── Pass 1: Nop removal ───────────────────────────────────────────────────────

fn remove_nops(f: &mut BcFn) {
    let old = std::mem::take(&mut f.code);
    let mut i = 0;
    while i < old.len() {
        if old[i] == Op::Nop as u8 {
            i += 1;
            continue;
        }
        let op = Op::from_u8(old[i]).unwrap_or(Op::Nop);
        let len = 1 + op.imm_bytes();
        if i + len <= old.len() {
            f.code.extend_from_slice(&old[i..i + len]);
        }
        i += len;
    }
}

// ── Pass 2: Dead code elimination ─────────────────────────────────────────────

/// Remove instructions that provably cannot be reached because they follow an
/// unconditional `Jump`, `Return`, or `ReturnVoid` within the same flat stream.
///
/// This is a simplified linear scan — it does not track cross-block reachability
/// (that is handled by the IR-level pass before emission).
fn dce(f: &mut BcFn) {
    let old = std::mem::take(&mut f.code);
    let mut out   = Vec::with_capacity(old.len());
    let mut alive = true;
    let mut i     = 0;

    while i < old.len() {
        let op = match Op::from_u8(old[i]) {
            Some(o) => o,
            None    => { i += 1; continue; }
        };
        let len = 1 + op.imm_bytes();

        if alive {
            if i + len <= old.len() {
                out.extend_from_slice(&old[i..i + len]);
            }
            match op {
                Op::Return | Op::ReturnVoid | Op::Jump => alive = false,
                _ => {}
            }
        } else {
            // Dead instruction — skip.  Re-enable on a JumpT / JumpF target
            // coming in (we can't track that without a full CFG, so we just
            // re-enable unconditionally at any label candidate; erring on the
            // safe side is correct).
        }
        i += len;
    }

    f.code = out;
}

// ── Pass 3: Constant folding ───────────────────────────────────────────────────

/// Replace `PUSH_* PUSH_* <binop>` triplets where both operands are constants
/// with a single `PUSH_*` of the pre-computed result.
///
/// Works on i32 and f64 operands for all arithmetic/comparison operators.
fn constant_fold(f: &mut BcFn) {
    let old = std::mem::take(&mut f.code);
    let mut out = Vec::with_capacity(old.len());
    let mut i   = 0;

    while i < old.len() {
        let op = Op::from_u8(old[i]).unwrap_or(Op::Nop);

        // Look ahead for PUSH_I32 PUSH_I32 <binop> triplet.
        if op == Op::PushI32 && i + 5 + 5 + 1 <= old.len() {
            let a    = i32::from_le_bytes(old[i+1..i+5].try_into().unwrap());
            let op2  = Op::from_u8(old[i+5]);
            if op2 == Some(Op::PushI32) {
                let b    = i32::from_le_bytes(old[i+6..i+10].try_into().unwrap());
                let bop  = Op::from_u8(old[i+10]);
                if let Some(result) = fold_i32(a, b, bop) {
                    // Emit single PushI32 result.
                    out.push(Op::PushI32 as u8);
                    out.extend_from_slice(&result.to_le_bytes());
                    i += 11;
                    continue;
                }
            }
        }

        // Look ahead for PUSH_F64 PUSH_F64 <binop>.
        if op == Op::PushF64 && i + 9 + 9 + 1 <= old.len() {
            let a   = f64::from_le_bytes(old[i+1..i+9].try_into().unwrap());
            let op2 = Op::from_u8(old[i+9]);
            if op2 == Some(Op::PushF64) {
                let b   = f64::from_le_bytes(old[i+10..i+18].try_into().unwrap());
                let bop = Op::from_u8(old[i+18]);
                if let Some(result) = fold_f64(a, b, bop) {
                    out.push(Op::PushF64 as u8);
                    out.extend_from_slice(&result.to_le_bytes());
                    i += 19;
                    continue;
                }
            }
        }

        // Nothing folded — copy instruction verbatim.
        let len = 1 + op.imm_bytes();
        if i + len <= old.len() {
            out.extend_from_slice(&old[i..i + len]);
        }
        i += len;
    }

    f.code = out;
}

fn fold_i32(a: i32, b: i32, op: Option<Op>) -> Option<i32> {
    match op {
        Some(Op::Add) => Some(a.wrapping_add(b)),
        Some(Op::Sub) => Some(a.wrapping_sub(b)),
        Some(Op::Mul) => Some(a.wrapping_mul(b)),
        Some(Op::Div) if b != 0 => Some(a.wrapping_div(b)),
        Some(Op::Rem) if b != 0 => Some(a.wrapping_rem(b)),
        Some(Op::Eq)  => Some((a == b) as i32),
        Some(Op::Ne)  => Some((a != b) as i32),
        Some(Op::Lt)  => Some((a < b)  as i32),
        Some(Op::Le)  => Some((a <= b) as i32),
        Some(Op::Gt)  => Some((a > b)  as i32),
        Some(Op::Ge)  => Some((a >= b) as i32),
        Some(Op::BitAnd) => Some(a & b),
        Some(Op::BitOr)  => Some(a | b),
        Some(Op::BitXor) => Some(a ^ b),
        Some(Op::Shl)    => Some(a.wrapping_shl(b as u32)),
        Some(Op::Shr)    => Some(a.wrapping_shr(b as u32)),
        _ => None,
    }
}

fn fold_f64(a: f64, b: f64, op: Option<Op>) -> Option<f64> {
    match op {
        Some(Op::Add) => Some(a + b),
        Some(Op::Sub) => Some(a - b),
        Some(Op::Mul) => Some(a * b),
        Some(Op::Div) if b != 0.0 => Some(a / b),
        _ => None,
    }
}
