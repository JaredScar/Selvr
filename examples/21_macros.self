// 21 — Macros
// Macros are expanded at compile time — zero runtime cost.
// Think of them as TypeScript decorators or code generation, but more powerful.

// User-defined macro — generates a struct at compile time
macro makePoint(name) {
    export struct $name {
        x: f64;
        y: f64;
    }

    impl $name {
        fn new(x: f64, y: f64): $name {
            return $name { x, y };
        }

        fn distanceTo(other: $name): f64 {
            const dx = this.x - other.x;
            const dy = this.y - other.y;
            return Math.sqrt(dx * dx + dy * dy);
        }
    }
}

makePoint!(Point2D);

// Built-in macros
fn fib(n: i32): i64 {
    if n <= 1 { return n as i64; }
    let a: i64 = 0;
    let b: i64 = 1;
    for _ in 2..=n {
        const c = a + b;
        a = b;
        b = c;
    }
    return b;
}

fn main(): void {
    // format! — like TypeScript's template literals but for non-string contexts
    const greeting = format!("Hello, {}!", "world");
    console.log(greeting);

    // assert! — compile-time assertion (caught at compile time, not runtime)
    assert!(1 + 1 === 2, "math is broken");

    // User macro
    const p1 = Point2D.new(0.0, 0.0);
    const p2 = Point2D.new(3.0, 4.0);
    console.log(p1.distanceTo(p2));  // 5.0

    // todo! — marks unimplemented code (panics at runtime if reached)
    // todo!("implement this later");

    // vec! shorthand — same as TypeScript's [] literal but for SELVR's Vec
    const v = vec![1, 2, 3, 4, 5];
    console.log(v);
}
