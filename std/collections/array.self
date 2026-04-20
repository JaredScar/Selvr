// std/collections/array.self
// Array<T> utilities — higher-order wrappers over Selvr's built-in array type.
//
// The targeting pass scores purely-numeric functions as WASM and DOM/callback
// functions as JS automatically — no annotations needed on most of these.

/// Concatenate two i32 arrays.
#[wasm]
export fn concat_i32(a: i32[], b: i32[]): i32[] {
    let result: i32[] = [];
    let i: i32 = 0;
    while i < a.length {
        result.push(a[i]);
        i = i + 1;
    }
    let j: i32 = 0;
    while j < b.length {
        result.push(b[j]);
        j = j + 1;
    }
    return result;
}

/// Slice an i32 array [from, to).
#[wasm]
export fn slice_i32(v: i32[], from: i32, to: i32): i32[] {
    let result: i32[] = [];
    let i: i32 = from;
    while i < to {
        result.push(v[i]);
        i = i + 1;
    }
    return result;
}

/// Check if an array contains a value.
#[wasm]
export fn includes_i32(v: i32[], val: i32): boolean {
    let i: i32 = 0;
    while i < v.length {
        if v[i] === val { return true; }
        i = i + 1;
    }
    return false;
}

/// Return the index of the first occurrence of `val`, or -1 if not found.
#[wasm]
export fn index_of_i32(v: i32[], val: i32): i32 {
    let i: i32 = 0;
    while i < v.length {
        if v[i] === val { return i; }
        i = i + 1;
    }
    return -1;
}

/// Insertion-sort an i32 array in ascending order (in-place).
/// For large arrays, the targeting pass routes this to WASM.
#[wasm]
export fn sort_i32(v: i32[], n: i32): void {
    let i: i32 = 1;
    while i < n {
        let key: i32 = v[i];
        let j: i32 = i - 1;
        while j >= 0 {
            if v[j] > key {
                v[j + 1] = v[j];
                j = j - 1;
            } else {
                j = -1;
            }
        }
        v[j + 1] = key;
        i = i + 1;
    }
}

/// Dedup a sorted i32 array — removes consecutive duplicates.
#[wasm]
export fn dedup_i32(v: i32[], n: i32): i32[] {
    let result: i32[] = [];
    if n === 0 { return result; }
    result.push(v[0]);
    let i: i32 = 1;
    while i < n {
        if v[i] !== v[i - 1] {
            result.push(v[i]);
        }
        i = i + 1;
    }
    return result;
}

/// Zip two i32 arrays into an array of sums (element-wise).
/// (Full generic zip awaits Phase 3 type system completion.)
#[wasm]
export fn zip_sum_i32(a: i32[], b: i32[], n: i32): i32[] {
    let result: i32[] = [];
    let i: i32 = 0;
    while i < n {
        result.push(a[i] + b[i]);
        i = i + 1;
    }
    return result;
}

/// Moving average of a f64 array with window size `w`.
#[wasm]
export fn moving_avg(v: f64[], n: i32, w: i32): f64[] {
    let result: f64[] = [];
    let i: i32 = 0;
    while i <= n - w {
        let s: f64 = 0.0;
        let k: i32 = 0;
        while k < w {
            s = s + v[i + k];
            k = k + 1;
        }
        result.push(s / w);
        i = i + 1;
    }
    return result;
}
