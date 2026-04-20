// ERROR: type mismatch — expected i32, found bool
// EXPECT: TypeMismatch

fn add(a: i32, b: i32): i32 {
    return a + b;
}

fn main(): void {
    const result = add(1, true);  // bool is not i32
    console.log(result);
}
