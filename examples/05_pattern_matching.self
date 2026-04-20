// 05 — Pattern matching
// match is exhaustive — the compiler rejects missing cases.
// Think of it as a supercharged switch that actually works.

enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn valueInCents(coin: Coin): i32 {
    return match coin {
        Coin.Penny   => 1,
        Coin.Nickel  => 5,
        Coin.Dime    => 10,
        Coin.Quarter => 25,
    };
}

fn classify(n: i32): string {
    return match n {
        0          => "zero",
        1..=9      => "single digit",
        10..=99    => "double digit",
        x if x < 0 => "negative",
        _           => "large",
    };
}

// Tuple pattern matching
fn describePoint(x: i32, y: i32): string {
    return match [x, y] {
        [0, 0] => "origin",
        [x, 0] => `on x-axis at ${x}`,
        [0, y] => `on y-axis at ${y}`,
        [x, y] => `at (${x}, ${y})`,
    };
}

// Or-patterns
fn isWeekend(day: string): boolean {
    return match day {
        "Saturday" | "Sunday" => true,
        _                     => false,
    };
}

fn main(): void {
    console.log(valueInCents(Coin.Quarter));   // 25
    console.log(classify(0));                  // zero
    console.log(classify(42));                 // double digit
    console.log(classify(-5));                 // negative
    console.log(describePoint(3, 0));          // on x-axis at 3
    console.log(isWeekend("Saturday"));        // true
    console.log(isWeekend("Monday"));          // false
}
