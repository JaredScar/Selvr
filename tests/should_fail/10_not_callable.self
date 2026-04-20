// ERROR: value is not a function
// EXPECT: NotCallable

fn main(): void {
    const x = 42;
    const result = x(1, 2);  // ERROR: i32 is not callable
    console.log(result);
}
