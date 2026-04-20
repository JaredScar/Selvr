// 08 — Option<T> and Result<T, E>
// SELVR has no null or undefined. Absence is modelled with Option<T>.
// Fallibility is modelled with Result<T, E>.
// Both are enums — no special syntax needed to work with them.

enum ParseError {
    EmptyInput,
    InvalidChar(char),
    Overflow,
}

fn parsePositive(s: string): Result<i32, ParseError> {
    if s.length === 0 {
        return Err(ParseError.EmptyInput);
    }
    let result: i32 = 0;
    for c in s.chars() {
        if c < '0' || c > '9' {
            return Err(ParseError.InvalidChar(c));
        }
        result = result * 10 + (c as i32 - '0' as i32);
        if result < 0 {
            return Err(ParseError.Overflow);
        }
    }
    return Ok(result);
}

fn doubleIfEven(n: i32): Option<i32> {
    if n % 2 === 0 {
        return Some(n * 2);
    }
    return None;
}

// The ? operator propagates errors up automatically — like TypeScript's try/catch
// but at compile time with no runtime overhead.
fn pipeline(input: string): Result<i32, ParseError> {
    const n = parsePositive(input)?;
    const doubled = doubleIfEven(n).unwrapOr(n);
    return Ok(doubled);
}

fn main(): void {
    // Option — presence or absence of a value
    const maybe: Option<i32> = Some(21);
    const mapped = maybe.map((x) => x * 2);        // Some(42)
    const flat   = maybe.andThen(doubleIfEven);     // None (21 is odd)
    const orVal  = (None as Option<i32>).unwrapOr(99); // 99

    match mapped {
        Some(v) => console.log(`mapped: ${v}`),
        None    => console.log("nothing"),
    }

    // Result — success or a typed error
    match parsePositive("123") {
        Ok(n)  => console.log(`parsed: ${n}`),
        Err(_) => console.log("parse failed"),
    }

    match parsePositive("12x3") {
        Ok(n)                          => console.log(`parsed: ${n}`),
        Err(ParseError.InvalidChar(c)) => console.log(`bad char: ${c}`),
        Err(_)                         => console.log("other error"),
    }

    match pipeline("42") {
        Ok(n)  => console.log(`pipeline: ${n}`),  // 84
        Err(_) => console.log("failed"),
    }
}
