// ERROR: cannot assign to immutable binding
// EXPECT: ImmutableAssign

fn main(): void {
    const x = 10;
    x = 20;  // ERROR: x is const
    console.log(x);
}
