// 23 — Type system soundness
// Cases where SELVR is stricter than TypeScript.
// Each commented-out line would be a compile error — the comment explains why.
// TypeScript developers: these are the gaps that SELVR plugs.

// ── Case 1: No implicit numeric coercion ──────────────────────────────────────
fn addInts(a: i32, b: i32): i32 { return a + b; }

// addInts(1, 1.5);
// ERROR: expected i32, found f64
// TypeScript: 1 + 1.5 === 2.5 silently (all numbers are `number`)

// ── Case 2: No null / undefined ───────────────────────────────────────────────
fn getName(): Option<string> { return None; }

const name = getName();
// console.log(name.length);
// ERROR: Option<string> has no property `length` — must unwrap first
// TypeScript: string | undefined — you can access .length with no guard

match name {
    Some(n) => console.log(n.length),  // safe inside Some branch
    None    => console.log("no name"),
}

// ── Case 3: Exhaustive match — no silent fall-through ─────────────────────────
enum Status { Active, Inactive, Pending }

fn describe(s: Status): string {
    return match s {
        Status.Active   => "active",
        Status.Inactive => "inactive",
        Status.Pending  => "pending",
        // Removing any arm above is a COMPILE ERROR in SELVR
        // TypeScript switch: a missing case silently returns undefined
    };
}

// ── Case 4: No unsound type assertions ────────────────────────────────────────
// TypeScript: `(x as string).toUpperCase()` compiles even if x is a number.
// SELVR: casting is only legal between compatible types.

const n: i32 = 42;
const f: f64 = n as f64;  // valid: widening numeric cast
// const s: string = n as string;  // ERROR: cannot cast i32 to string

// ── Case 5: const prevents reassignment — no silent mutation ──────────────────
const xs = [1, 2, 3];
// xs = [4, 5, 6];  // ERROR: xs is const — cannot be reassigned
let ys = [1, 2, 3];
ys = [4, 5, 6];    // fine — ys is let

// ── Case 6: Generics are truly generic — no `any` escape hatch ───────────────
fn identity<T>(x: T): T { return x; }

const i = identity(42);      // i is i32
const s2 = identity("hello"); // s2 is string
// const combined = i + s2;  // ERROR: cannot add i32 and string
// TypeScript with `any`: this compiles and blows up at runtime

fn main(): void {
    console.log(describe(Status.Active));
    console.log(f);
    console.log(ys);
}
