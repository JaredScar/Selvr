// 15 — Ownership and memory model
// SELVR manages memory deterministically — no GC, no manual free().
// The rules are enforced at compile time, invisibly for everyday code.
//
// Key rules:
//   const x = value  →  x owns the value
//   const y = x      →  value MOVES to y; x is no longer valid (non-Copy types)
//   value.clone()    →  explicit deep copy; both are valid

struct Buffer {
    data: i32[];
}

impl Buffer {
    fn new(): Buffer {
        return Buffer { data: [] };
    }

    fn push(value: i32): void {
        this.data.push(value);
    }

    fn length(): i32 {
        return this.data.length as i32;
    }

    fn sum(): i32 {
        return this.data.reduce((acc, x) => acc + x, 0);
    }
}

fn inspect(buf: Buffer): i32 {
    // Compiler borrows buf implicitly — caller retains ownership
    return buf.length();
}

fn consume(buf: Buffer): i32 {
    // buf is moved in — caller no longer owns it after this call
    return buf.sum();
}

fn main(): void {
    // --- Primitive types are Copy: assignment copies the value ---
    const x: i32 = 5;
    const y = x;   // x is copied, not moved
    console.log(x); // still valid
    console.log(y);

    // --- Structs move by default ---
    const a = Buffer.new();
    const b = a.clone(); // explicit deep copy
    // After this: both a and b are valid

    console.log(inspect(b.clone())); // borrow via clone

    // consume() takes ownership — a cannot be used after this line
    const total = consume(a);
    console.log(`sum = ${total}`);
    // console.log(a); // compile error: a was moved

    // --- const prevents reassignment; let allows it ---
    let count = 0;
    count = 1;      // ok — let is mutable
    // x = 10;      // compile error — const cannot be reassigned

    console.log("Ownership demo complete");
}
