// Selvr — matrix multiply benchmark (N×N f64)
// Compiled by: selvr build docs/benchmarks/selvr/matmul.self
//
// This function is a prime WASM candidate: float-heavy triple loop, no DOM.
// `selvr explain matmul.self` will assign it Target::Wasm.

// Square matrix multiply: c = a * b, all row-major f64[].
export fn matmul(a: f64[], b: f64[], n: i32): f64[] {
    let c: f64[] = [];
    let init: i32 = 0;
    while init < n * n {
        c.push(0.0);
        init = init + 1;
    }
    let row: i32 = 0;
    while row < n {
        let k: i32 = 0;
        while k < n {
            let aik: f64 = a[row * n + k];
            let col: i32 = 0;
            while col < n {
                c[row * n + col] = c[row * n + col] + aik * b[k * n + col];
                col = col + 1;
            }
            k = k + 1;
        }
        row = row + 1;
    }
    return c;
}

// Dot product of two f64 vectors — simplest float loop for quick benchmarking.
export fn dot(a: f64[], b: f64[], n: i32): f64 {
    let s: f64 = 0.0;
    let i: i32 = 0;
    while i < n {
        s = s + a[i] * b[i];
        i = i + 1;
    }
    return s;
}
