// 06 — Structs
// Structs are value types with named fields.
// Methods live in an impl block — `this` refers to the current instance.

struct Point {
    x: f64;
    y: f64;
}

impl Point {
    // Static constructor method
    fn new(x: f64, y: f64): Point {
        return Point { x, y };
    }

    fn distanceFromOrigin(): f64 {
        return Math.sqrt(this.x * this.x + this.y * this.y);
    }

    fn translate(dx: f64, dy: f64): Point {
        return Point { x: this.x + dx, y: this.y + dy };
    }

    fn midpoint(other: Point): Point {
        return Point {
            x: (this.x + other.x) / 2.0,
            y: (this.y + other.y) / 2.0,
        };
    }
}

struct Rectangle {
    topLeft: Point;
    bottomRight: Point;
}

impl Rectangle {
    fn width(): f64     { return this.bottomRight.x - this.topLeft.x; }
    fn height(): f64    { return this.bottomRight.y - this.topLeft.y; }
    fn area(): f64      { return this.width() * this.height(); }
    fn perimeter(): f64 { return 2.0 * (this.width() + this.height()); }
}

fn main(): void {
    const a = Point.new(0.0, 0.0);
    const b = Point.new(3.0, 4.0);

    console.log(b.distanceFromOrigin());      // 5.0

    const mid = a.midpoint(b);
    console.log(`midpoint: (${mid.x}, ${mid.y})`); // (1.5, 2.0)

    // Struct update syntax — copy b but override x
    const d = Point { x: 10.0, ..b };
    console.log(`d = (${d.x}, ${d.y})`);     // (10.0, 4.0)

    const rect = Rectangle {
        topLeft:     Point.new(0.0, 10.0),
        bottomRight: Point.new(5.0, 0.0),
    };
    console.log(`area = ${rect.area()}`);         // 50.0
    console.log(`perimeter = ${rect.perimeter()}`); // 30.0
}
