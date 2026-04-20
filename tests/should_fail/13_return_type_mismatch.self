// ERROR: return type mismatch — declared string, returned i32
// EXPECT: TypeMismatch

fn greet(name: string): string {
    return 42;  // ERROR: expected string, found i32
}

fn main(): void {
    console.log(greet("world"));
}
