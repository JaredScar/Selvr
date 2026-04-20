// Selvr — fibonacci benchmark
// Compiled by: selvr build docs/benchmarks/selvr/fib.self

// Iterative fibonacci — O(n) time, O(1) space.
// The targeting pass scores this as WASM: tight integer loop.
export fn fib(n: i32): i32 {
    let a: i32 = 0;
    let b: i32 = 1;
    let i: i32 = 0;
    while i < n {
        let tmp: i32 = a + b;
        a = b;
        b = tmp;
        i = i + 1;
    }
    return a;
}

// Recursive fibonacci — deliberately slow, shows compiler output for recursion.
export fn fib_rec(n: i32): i32 {
    if n <= 1 {
        return n;
    }
    return fib_rec(n - 1) + fib_rec(n - 2);
}
