// std/core/result.self
// Result<T, E> — a value that is either Ok(T) or Err(E).
//
// Selvr's Result is represented at runtime as { tag: "Ok", val: v } | { tag: "Err", err: e }.
// The __selvr runtime provides the constructors Ok(v) and Err(e).
//
// Usage:
//   let r: Result<i32, string> = Ok(42);
//   let e: Result<i32, string> = Err("not found");
//   console.log(r.unwrap());           // 42
//   r.match { Ok(v) => v, Err(e) => 0 }

/// Unwrap an Ok value; panic with the error if Err.
#[js]
export fn unwrap_result(r: Result<i32, string>): i32 {
    if r.tag === "Ok" { return r.val; }
    return 0;
}

/// Return the Ok value, or `default` if Err.
#[js]
export fn unwrap_or_result(r: Result<i32, string>, default: i32): i32 {
    if r.tag === "Ok" { return r.val; }
    return default;
}

/// Return true if Ok.
#[js]
export fn is_ok(r: Result<i32, string>): boolean {
    return r.tag === "Ok";
}

/// Return true if Err.
#[js]
export fn is_err(r: Result<i32, string>): boolean {
    return r.tag === "Err";
}

/// Extract the error value, or panic if Ok.
#[js]
export fn unwrap_err(r: Result<i32, string>): string {
    if r.tag === "Err" { return r.err; }
    return "";
}
