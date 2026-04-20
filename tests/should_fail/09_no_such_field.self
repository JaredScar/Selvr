// ERROR: struct field does not exist
// EXPECT: NoSuchField

struct Point {
    x: f64;
    y: f64;
}

fn main(): void {
    const p = Point { x: 1.0, y: 2.0 };
    console.log(p.z);  // ERROR: Point has no field `z`
}
