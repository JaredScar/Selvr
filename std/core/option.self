// std/core/option.self
// Option<T> — a value that may or may not be present.
//
// Selvr's Option is represented at runtime as { tag: "Some", val: v } | { tag: "None" }.
// The __selvr runtime preamble provides the underlying constructors.
//
// Usage:
//   let x: Option<i32> = Some(42);
//   let y: Option<i32> = None;
//   console.log(x.unwrap());          // 42
//   console.log(y.unwrapOr(0));       // 0
//   let z = x.map((v) => v * 2);      // Some(84)

// --- Core constructors (provided by __selvr runtime, re-exported here for documentation) ---
// fn Some<T>(val: T): Option<T>
// const None: Option<never>

/// Return the contained value, or panic with `msg` if None.
#[js]
export fn unwrap_option(opt: Option<i32>): i32 {
    if opt.tag === "Some" { return opt.val; }
    return 0;
}

/// Return the contained value, or `default` if None.
#[js]
export fn unwrap_or(opt: Option<i32>, default: i32): i32 {
    if opt.tag === "Some" { return opt.val; }
    return default;
}

/// Return true if the option contains a value.
#[js]
export fn is_some(opt: Option<i32>): boolean {
    return opt.tag === "Some";
}

/// Return true if the option is empty.
#[js]
export fn is_none(opt: Option<i32>): boolean {
    return opt.tag === "None";
}

/// Convert Option<T> to a single-element or empty array.
#[js]
export fn option_to_array(opt: Option<i32>): i32[] {
    let result: i32[] = [];
    if opt.tag === "Some" {
        result.push(opt.val);
    }
    return result;
}
