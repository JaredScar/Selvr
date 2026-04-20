// std/core/iter.self
// Iterator adaptors for Selvr.
//
// Selvr arrays expose .map(), .filter(), .reduce(), .find(), .forEach() natively
// because the JS codegen targets Array.prototype methods. These functions provide
// Selvr-idiomatic wrappers that are also WASM-targeting-aware.
//
// All numeric-heavy operations (sum, product, min, max, dot) are scored for WASM
// by the targeting pass (dense numeric loops). DOM-touching operations stay JS.

// ── Numeric reductions (WASM candidates) ────────────────────────────────────

/// Sum all elements of an i32 array.
#[wasm]
export fn sum_i32(v: i32[]): i32 {
    let s: i32 = 0;
    let i: i32 = 0;
    while i < v.length {
        s = s + v[i];
        i = i + 1;
    }
    return s;
}

/// Sum all elements of a f64 array.
#[wasm]
export fn sum_f64(v: f64[]): f64 {
    let s: f64 = 0.0;
    let i: i32 = 0;
    while i < v.length {
        s = s + v[i];
        i = i + 1;
    }
    return s;
}

/// Product of all elements.
#[wasm]
export fn product_i32(v: i32[]): i32 {
    let p: i32 = 1;
    let i: i32 = 0;
    while i < v.length {
        p = p * v[i];
        i = i + 1;
    }
    return p;
}

/// Minimum element of an i32 array. Returns 0 on empty input.
#[wasm]
export fn min_i32(v: i32[]): i32 {
    if v.length === 0 { return 0; }
    let m: i32 = v[0];
    let i: i32 = 1;
    while i < v.length {
        if v[i] < m { m = v[i]; }
        i = i + 1;
    }
    return m;
}

/// Maximum element of an i32 array. Returns 0 on empty input.
#[wasm]
export fn max_i32(v: i32[]): i32 {
    if v.length === 0 { return 0; }
    let m: i32 = v[0];
    let i: i32 = 1;
    while i < v.length {
        if v[i] > m { m = v[i]; }
        i = i + 1;
    }
    return m;
}

/// Dot product of two f64 arrays (zero-copy WASM bridge path).
#[wasm]
export fn dot(a: f64[], b: f64[], n: i32): f64 {
    let s: f64 = 0.0;
    let i: i32 = 0;
    while i < n {
        s = s + a[i] * b[i];
        i = i + 1;
    }
    return s;
}

/// Count elements satisfying a predicate (JS — predicate involves closure).
#[js]
export fn count_where(v: i32[]): i32 {
    let c: i32 = 0;
    let i: i32 = 0;
    while i < v.length {
        if v[i] > 0 {
            c = c + 1;
        }
        i = i + 1;
    }
    return c;
}

/// Fill an i32 array with a constant value.
#[wasm]
export fn fill_i32(n: i32, val: i32): i32[] {
    let v: i32[] = [];
    let i: i32 = 0;
    while i < n {
        v.push(val);
        i = i + 1;
    }
    return v;
}

/// Generate i32 range [start, end).
#[wasm]
export fn range(start: i32, end: i32): i32[] {
    let v: i32[] = [];
    let i: i32 = start;
    while i < end {
        v.push(i);
        i = i + 1;
    }
    return v;
}

/// Reverse an array in-place.
#[wasm]
export fn reverse_i32(v: i32[], n: i32): void {
    let lo: i32 = 0;
    let hi: i32 = n - 1;
    while lo < hi {
        let tmp: i32 = v[lo];
        v[lo] = v[hi];
        v[hi] = tmp;
        lo = lo + 1;
        hi = hi - 1;
    }
}

/// Check if all elements satisfy `> 0`.
#[wasm]
export fn all_positive(v: i32[]): boolean {
    let i: i32 = 0;
    while i < v.length {
        if v[i] <= 0 { return false; }
        i = i + 1;
    }
    return true;
}
