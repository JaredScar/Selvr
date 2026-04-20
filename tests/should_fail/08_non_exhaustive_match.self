// ERROR: non-exhaustive match — missing None arm
// EXPECT: NonExhaustiveMatch

fn unwrapDouble(x: Option<f64>): f64 {
    return match x {
        Some(v) => v * 2.0,
        // ERROR: None arm is missing
    };
}

fn main(): void {
    console.log(unwrapDouble(Some(3.14)));
}
