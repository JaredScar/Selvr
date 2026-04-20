// Parser snapshot tests — validates that the parser produces the same output
// for every example file across refactors.
//
// HOW SNAPSHOTS WORK
// ──────────────────
// Each test case calls `check(src, expected_items, expected_errors)`.
// `expected_items` is the count of top-level AST items in a clean parse.
// `expected_errors` is the number of parse errors for files with known issues.
//
// When the parser is refactored, run:
//   cargo test -p selvr-parser -- --nocapture
// to see any mismatches, then update the expected counts here.

use selvr_lexer::Lexer;
use selvr_parser::Parser;

fn parse_src(src: &str) -> (usize, usize) {
    let (tokens, lex_errors) = Lexer::new(src, 0).tokenize();
    let (module, parse_errors) = Parser::new(tokens, 0).parse();
    (module.items.len(), lex_errors.len() + parse_errors.len())
}

fn check(label: &str, src: &str, expected_items: usize, expected_errors: usize) {
    let (items, errors) = parse_src(src);
    assert_eq!(
        items, expected_items,
        "[{label}] expected {expected_items} top-level items, got {items}"
    );
    assert_eq!(
        errors, expected_errors,
        "[{label}] expected {expected_errors} parse errors, got {errors}"
    );
}

// ── 01 hello world ────────────────────────────────────────────────────────────
#[test]
fn hello_world() {
    check("01_hello_world", r#"
fn main(): void {
    console.log("Hello, Selvr!");
}
"#, 1, 0);
}

// ── 02 variables ──────────────────────────────────────────────────────────────
#[test]
fn variables() {
    check("02_variables", r#"
fn demo(): void {
    let x: i32 = 42;
    let y: f64 = 3.14;
    let z: boolean = true;
    let s: string = "hello";
    x = x + 1;
    y = y * 2.0;
}
"#, 1, 0);
}

// ── 03 functions ──────────────────────────────────────────────────────────────
#[test]
fn functions_basic() {
    check("03_functions_basic", r#"
fn add(a: i32, b: i32): i32 {
    return a + b;
}
fn multiply(a: i32, b: i32): i32 {
    return a * b;
}
fn main(): void {
    console.log(add(3, 4));
    console.log(multiply(5, 6));
}
"#, 3, 0);
}

// ── 04 control flow ───────────────────────────────────────────────────────────
#[test]
fn control_flow() {
    check("04_control_flow", r#"
fn abs(n: i32): i32 {
    if n < 0 {
        return n * -1;
    }
    return n;
}
fn factorial(n: i32): i32 {
    let result: i32 = 1;
    let i: i32 = 1;
    while i <= n {
        result = result * i;
        i = i + 1;
    }
    return result;
}
fn main(): void {
    console.log(abs(-5));
    console.log(factorial(10));
}
"#, 3, 0);
}

// ── 05 if/else expression ─────────────────────────────────────────────────────
#[test]
fn if_else_expr() {
    check("05_if_else_expr", r#"
fn sign(n: i32): i32 {
    if n > 0 { return 1; }
    if n < 0 { return -1; }
    return 0;
}
"#, 1, 0);
}

// ── 06 fibonacci (iterative + recursive) ──────────────────────────────────────
#[test]
fn fibonacci() {
    check("06_fibonacci", r#"
fn fib(n: i32): i32 {
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
fn fib_rec(n: i32): i32 {
    if n <= 1 { return n; }
    return fib_rec(n - 1) + fib_rec(n - 2);
}
"#, 2, 0);
}

// ── 07 array operations ───────────────────────────────────────────────────────
#[test]
fn array_operations() {
    check("07_array_ops", r#"
fn sum(v: i32[]): i32 {
    let s: i32 = 0;
    let i: i32 = 0;
    while i < v.length {
        s = s + v[i];
        i = i + 1;
    }
    return s;
}
fn push_range(n: i32): i32[] {
    let v: i32[] = [];
    let i: i32 = 0;
    while i < n {
        v.push(i);
        i = i + 1;
    }
    return v;
}
"#, 2, 0);
}

// ── 08 sieve of eratosthenes ──────────────────────────────────────────────────
#[test]
fn sieve() {
    check("08_sieve", r#"
fn sieve(limit: i32): i32 {
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
            let m: i32 = p + p;
            while m <= limit {
                composite[m] = true;
                m = m + p;
            }
        }
        p = p + 1;
    }
    return count;
}
"#, 1, 0);
}

// ── 09 dot product ────────────────────────────────────────────────────────────
#[test]
fn dot_product() {
    check("09_dot_product", r#"
fn dot(a: f64[], b: f64[], n: i32): f64 {
    let s: f64 = 0.0;
    let i: i32 = 0;
    while i < n {
        s = s + a[i] * b[i];
        i = i + 1;
    }
    return s;
}
"#, 1, 0);
}

// ── 10 exports ────────────────────────────────────────────────────────────────
#[test]
fn exports() {
    check("10_exports", r#"
export fn add(a: i32, b: i32): i32 { return a + b; }
export fn sub(a: i32, b: i32): i32 { return a - b; }
export fn mul(a: i32, b: i32): i32 { return a * b; }
"#, 3, 0);
}

// ── 11 nested while loops ─────────────────────────────────────────────────────
#[test]
fn nested_while() {
    check("11_nested_while", r#"
fn matmul_sum(n: i32): f64 {
    let total: f64 = 0.0;
    let i: i32 = 0;
    while i < n {
        let j: i32 = 0;
        while j < n {
            total = total + 1.0;
            j = j + 1;
        }
        i = i + 1;
    }
    return total;
}
"#, 1, 0);
}

// ── 12 multiple returns ───────────────────────────────────────────────────────
#[test]
fn multiple_returns() {
    check("12_multi_return", r#"
fn clamp(v: i32, lo: i32, hi: i32): i32 {
    if v < lo { return lo; }
    if v > hi { return hi; }
    return v;
}
"#, 1, 0);
}

// ── 13 boolean logic ─────────────────────────────────────────────────────────
#[test]
fn boolean_logic() {
    check("13_bool_logic", r#"
fn all_positive(v: i32[]): boolean {
    let i: i32 = 0;
    while i < v.length {
        if v[i] <= 0 { return false; }
        i = i + 1;
    }
    return true;
}
"#, 1, 0);
}

// ── 14 wasm attribute ─────────────────────────────────────────────────────────
#[test]
fn wasm_attribute() {
    check("14_wasm_attr", r#"
#[wasm]
fn heavy_compute(n: i32): i32 {
    let s: i32 = 0;
    let i: i32 = 0;
    while i < n {
        s = s + i * i;
        i = i + 1;
    }
    return s;
}

#[js]
fn render(result: i32): void {
    console.log(result);
}
"#, 2, 0);
}

// ── 15 unary negation ─────────────────────────────────────────────────────────
#[test]
fn unary_ops() {
    check("15_unary", r#"
fn negate(n: i32): i32 { return n * -1; }
fn logical_not(b: boolean): boolean { return !b; }
"#, 2, 0);
}

// ── 16 compound assignment ────────────────────────────────────────────────────
#[test]
fn compound_assign() {
    check("16_compound_assign", r#"
fn run(): void {
    let x: i32 = 10;
    x += 5;
    x -= 2;
    x *= 3;
    x %= 7;
}
"#, 1, 0);
}

// ── 17 f64 arithmetic ─────────────────────────────────────────────────────────
#[test]
fn float_arithmetic() {
    check("17_float_arith", r#"
fn circle_area(r: f64): f64 {
    return r * r * 3.14159265;
}
fn lerp(a: f64, b: f64, t: f64): f64 {
    return a + (b - a) * t;
}
"#, 2, 0);
}

// ── 18 boolean array ─────────────────────────────────────────────────────────
#[test]
fn bool_array() {
    check("18_bool_array", r#"
fn count_trues(flags: boolean[]): i32 {
    let n: i32 = 0;
    let i: i32 = 0;
    while i < flags.length {
        if flags[i] {
            n = n + 1;
        }
        i = i + 1;
    }
    return n;
}
"#, 1, 0);
}

// ── 19 multiple functions calling each other ──────────────────────────────────
#[test]
fn mutual_calls() {
    check("19_mutual_calls", r#"
fn is_even(n: i32): boolean {
    if n == 0 { return true; }
    return is_odd(n - 1);
}
fn is_odd(n: i32): boolean {
    if n == 0 { return false; }
    return is_even(n - 1);
}
"#, 2, 0);
}

// ── 20 array indexing and mutation ────────────────────────────────────────────
#[test]
fn array_index_mutation() {
    check("20_array_index_mut", r#"
fn reverse(v: i32[], n: i32): void {
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
"#, 1, 0);
}
