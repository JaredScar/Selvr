// Selvr — Sieve of Eratosthenes benchmark
// Compiled by: selvr build docs/benchmarks/selvr/sieve.self
//
// Uses push-based array construction (no index-assignment) to stay within
// the currently supported parser subset.

// Build a boolean flags array using push, count primes up to limit.
export fn sieve(limit: i32): i32 {
    // Build composite[] — true means composite (not prime).
    let composite: boolean[] = [];
    let i: i32 = 0;
    while i <= limit {
        composite.push(false);
        i = i + 1;
    }

    let count: i32 = 0;
    let p: i32 = 2;
    while p <= limit {
        if !composite[p] {
            count = count + 1;
            let multiple: i32 = p + p;
            while multiple <= limit {
                composite[multiple] = true;
                multiple = multiple + p;
            }
        }
        p = p + 1;
    }
    return count;
}
