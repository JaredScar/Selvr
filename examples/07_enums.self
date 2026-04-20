// 07 — Enums
// Enums can carry data — each variant is its own "shape".
// SELVR enums are much more powerful than TypeScript string/number enums.

enum Shape {
    Circle(f64),                          // tuple variant — holds radius
    Rectangle { width: f64; height: f64 }, // struct variant
    Triangle  { base: f64;  height: f64 },
    Point,                                // unit variant — no data
}

impl Shape {
    fn area(): f64 {
        return match this {
            Shape.Circle(r)                         => Math.PI * r * r,
            Shape.Rectangle { width, height }       => width * height,
            Shape.Triangle  { base, height }        => 0.5 * base * height,
            Shape.Point                             => 0.0,
        };
    }

    fn name(): string {
        return match this {
            Shape.Circle(_)         => "Circle",
            Shape.Rectangle { .. }  => "Rectangle",
            Shape.Triangle  { .. }  => "Triangle",
            Shape.Point             => "Point",
        };
    }
}

// Recursive enum — a simple expression tree.
// SELVR automatically boxes recursive variants.
enum Expr {
    Num(f64),
    Add(Expr, Expr),
    Mul(Expr, Expr),
    Neg(Expr),
}

fn eval(e: Expr): f64 {
    return match e {
        Expr.Num(n)     => n,
        Expr.Add(a, b)  => eval(a) + eval(b),
        Expr.Mul(a, b)  => eval(a) * eval(b),
        Expr.Neg(inner) => -eval(inner),
    };
}

fn main(): void {
    const shapes: Shape[] = [
        Shape.Circle(5.0),
        Shape.Rectangle { width: 4.0, height: 6.0 },
        Shape.Triangle  { base: 3.0,  height: 8.0 },
    ];

    for shape in shapes {
        console.log(`${shape.name()}: area = ${shape.area().toFixed(2)}`);
    }

    // Eval: -(2 + 3) * 4 = -20
    const expr = Expr.Mul(
        Expr.Neg(Expr.Add(Expr.Num(2.0), Expr.Num(3.0))),
        Expr.Num(4.0),
    );
    console.log(`result = ${eval(expr)}`); // -20
}
