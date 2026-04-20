// std/collections/string.self
// String utilities for Selvr.
//
// Selvr strings are immutable UTF-16 sequences (identical to JS strings).
// All operations delegate to the V8 built-ins; these wrappers provide
// idiomatic Selvr APIs and are scored as JS by the targeting pass
// (no numeric loops, uses string methods).

/// Return the number of UTF-16 code units in `s`.
#[js]
export fn len(s: string): i32 {
    return s.length;
}

/// Return true if `s` starts with `prefix`.
#[js]
export fn starts_with(s: string, prefix: string): boolean {
    return s.startsWith(prefix);
}

/// Return true if `s` ends with `suffix`.
#[js]
export fn ends_with(s: string, suffix: string): boolean {
    return s.endsWith(suffix);
}

/// Return true if `s` contains `needle`.
#[js]
export fn contains(s: string, needle: string): boolean {
    return s.includes(needle);
}

/// Return `s` converted to uppercase.
#[js]
export fn to_upper(s: string): string {
    return s.toUpperCase();
}

/// Return `s` converted to lowercase.
#[js]
export fn to_lower(s: string): string {
    return s.toLowerCase();
}

/// Strip leading and trailing whitespace.
#[js]
export fn trim(s: string): string {
    return s.trim();
}

/// Repeat `s` n times.
#[js]
export fn repeat(s: string, n: i32): string {
    return s.repeat(n);
}

/// Return the character at index `i`.
#[js]
export fn char_at(s: string, i: i32): string {
    return s.charAt(i);
}

/// Return the first index of `needle` in `s`, or -1 if not found.
#[js]
export fn index_of(s: string, needle: string): i32 {
    return s.indexOf(needle);
}

/// Return the substring from `start` to `end` (exclusive).
#[js]
export fn substring(s: string, start: i32, end: i32): string {
    return s.substring(start, end);
}

/// Replace the first occurrence of `from` with `to`.
#[js]
export fn replace_first(s: string, from: string, to: string): string {
    return s.replace(from, to);
}

/// Replace all occurrences of `from` with `to`.
#[js]
export fn replace_all(s: string, from: string, to: string): string {
    return s.replaceAll(from, to);
}

/// Split `s` by `sep` and return the parts as a string[].
#[js]
export fn split(s: string, sep: string): string[] {
    return s.split(sep);
}

/// Join a string[] into a single string with `sep` between elements.
#[js]
export fn join(parts: string[], sep: string): string {
    return parts.join(sep);
}

/// Parse an integer from a string. Returns 0 if parsing fails.
#[js]
export fn parse_i32(s: string): i32 {
    let n: i32 = parseInt(s, 10);
    return n;
}

/// Parse a float from a string. Returns 0.0 if parsing fails.
#[js]
export fn parse_f64(s: string): f64 {
    let n: f64 = parseFloat(s);
    return n;
}

/// Convert an i32 to its decimal string representation.
#[js]
export fn i32_to_string(n: i32): string {
    return n.toString();
}

/// Pad a string on the left to `width` with `pad_char`.
#[js]
export fn pad_start(s: string, width: i32, pad_char: string): string {
    return s.padStart(width, pad_char);
}

/// Pad a string on the right to `width` with `pad_char`.
#[js]
export fn pad_end(s: string, width: i32, pad_char: string): string {
    return s.padEnd(width, pad_char);
}
