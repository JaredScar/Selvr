// std/time/datetime.self
// Date and time utilities for Selvr.
//
// Wraps the browser's built-in Date API.
// All functions are scored as JS by the targeting pass.

/// Return the current Unix timestamp in milliseconds.
#[js]
export fn timestamp_ms(): f64 {
    return Date.now();
}

/// Return the current Unix timestamp in seconds (f64).
#[js]
export fn timestamp_s(): f64 {
    return Date.now() / 1000.0;
}

/// Return a high-resolution timestamp in milliseconds (from navigation start).
#[js]
export fn perf_now(): f64 {
    return performance.now();
}

/// Return the current year (Gregorian calendar, local timezone).
#[js]
export fn year(): i32 {
    return new Date().getFullYear();
}

/// Return the current month (1–12, local timezone).
#[js]
export fn month(): i32 {
    return new Date().getMonth() + 1;
}

/// Return the current day of the month (1–31, local timezone).
#[js]
export fn day(): i32 {
    return new Date().getDate();
}

/// Return the current hour (0–23, local timezone).
#[js]
export fn hour(): i32 {
    return new Date().getHours();
}

/// Return the current minute (0–59, local timezone).
#[js]
export fn minute(): i32 {
    return new Date().getMinutes();
}

/// Return the current second (0–59, local timezone).
#[js]
export fn second(): i32 {
    return new Date().getSeconds();
}

/// Format the current date as "YYYY-MM-DD" (ISO 8601 date portion).
#[js]
export fn today_iso(): string {
    return new Date().toISOString().slice(0, 10);
}

/// Format a Unix timestamp (ms) as a locale-specific date string.
#[js]
export fn format_date(ts_ms: f64): string {
    return new Date(ts_ms).toLocaleDateString();
}

/// Format a Unix timestamp (ms) as a locale-specific time string.
#[js]
export fn format_time(ts_ms: f64): string {
    return new Date(ts_ms).toLocaleTimeString();
}

/// Measure elapsed time between two performance.now() samples (in ms).
#[js]
export fn elapsed_ms(start: f64, end: f64): f64 {
    return end - start;
}

/// Compute the difference in days between two Unix timestamps (ms).
#[wasm]
export fn diff_days(a_ms: f64, b_ms: f64): i32 {
    let diff: f64 = a_ms - b_ms;
    if diff < 0.0 { diff = diff * -1.0; }
    return diff / 86400000.0;
}
