// std/test/test.self
// Selvr built-in test framework.
//
// USAGE
// ─────
// Annotate functions with #[test] and run them with `selvr test`.
// The test runner collects all #[test] functions, executes them,
// and reports pass/fail counts.
//
// Example:
//   import { assert_eq, assert, assert_ne } from "std/test";
//
//   #[test]
//   fn test_fib() {
//       assert_eq(fib(0), 0);
//       assert_eq(fib(1), 1);
//       assert_eq(fib(10), 55);
//   }
//
//   #[test]
//   fn test_sieve() {
//       assert_eq(sieve(10), 4);   // 2, 3, 5, 7
//       assert_eq(sieve(100), 25);
//   }
//
// Run with:  selvr test my_module.self

// ── Assertion helpers ────────────────────────────────────────────────────────

/// Assert that `condition` is true. Panics with `msg` on failure.
#[js]
export fn assert(condition: boolean, msg: string): void {
    if !condition {
        console.error(msg);
    }
}

/// Assert that two i32 values are equal.
#[js]
export fn assert_eq_i32(a: i32, b: i32): void {
    if a !== b {
        console.error("assertion failed: " + a.toString() + " !== " + b.toString());
    }
}

/// Assert that two f64 values are approximately equal (within `eps`).
#[js]
export fn assert_approx_eq(a: f64, b: f64, eps: f64): void {
    let diff: f64 = a - b;
    if diff < 0.0 { diff = diff * -1.0; }
    if diff > eps {
        console.error("approx eq failed: |" + a.toString() + " - " + b.toString() + "| > " + eps.toString());
    }
}

/// Assert that two i32 values are NOT equal.
#[js]
export fn assert_ne_i32(a: i32, b: i32): void {
    if a === b {
        console.error("assertion failed: expected " + a.toString() + " !== " + b.toString());
    }
}

/// Assert that a boolean is true.
#[js]
export fn assert_true(val: boolean): void {
    if !val {
        console.error("assertion failed: expected true, got false");
    }
}

/// Assert that a boolean is false.
#[js]
export fn assert_false(val: boolean): void {
    if val {
        console.error("assertion failed: expected false, got true");
    }
}

/// Assert that two strings are equal.
#[js]
export fn assert_eq_str(a: string, b: string): void {
    if a !== b {
        console.error("assertion failed: \"" + a + "\" !== \"" + b + "\"");
    }
}

// ── Test runner primitives ────────────────────────────────────────────────────
// The `selvr test` CLI command uses these at runtime.

/// Record a test result. Called by the CLI test harness.
#[js]
export fn record_pass(name: string): void {
    console.log("  ✓ " + name);
}

/// Record a test failure with a message.
#[js]
export fn record_fail(name: string, msg: string): void {
    console.error("  ✗ " + name + ": " + msg);
}

/// Print the final test summary.
#[js]
export fn print_summary(passed: i32, failed: i32): void {
    let total: i32 = passed + failed;
    console.log("\n" + passed.toString() + "/" + total.toString() + " tests passed.");
    if failed > 0 {
        console.error(failed.toString() + " test(s) FAILED.");
    } else {
        console.log("All tests passed.");
    }
}
