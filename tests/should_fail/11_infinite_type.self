// ERROR: occurs check — infinite recursive type
// EXPECT: InfiniteType

fn selfRef(x: i32): i32 {
    // The inferred type of `f` would be `fn(fn(...)) -> i32` — infinite.
    const f = (g) => g(g);
    return f(f);
}

fn main(): void {
    console.log(selfRef(0));
}
