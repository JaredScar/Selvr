// ERROR: function declared i32 return but no return in all paths
// EXPECT: MissingReturn

fn sign(n: i32): i32 {
    if n > 0 {
        return 1;
    }
    // ERROR: no return for n <= 0 branch
}

fn main(): void {
    console.log(sign(5));
}
