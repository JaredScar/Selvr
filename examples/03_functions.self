// 03 — Functions
// Return type goes after the parameters with a colon — just like TypeScript.

fn add(a: i32, b: i32): i32 {
    return a + b;
}

fn square(n: i32): i32 {
    return n * n;
}

// Multiple return values via a tuple
fn divmod(a: i32, b: i32): [i32, i32] {
    return [a / b, a % b];
}

// Void return — colon : void is optional but explicit is clear
fn greet(name: string): void {
    console.log(`Hello, ${name}!`);
}

// Recursive function
fn factorial(n: i32): i32 {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

fn main(): void {
    console.log(add(3, 4));       // 7
    console.log(square(9));       // 81
    greet("world");               // Hello, world!

    const [q, r] = divmod(17, 5);
    console.log(`17 / 5 = ${q} remainder ${r}`);

    console.log(factorial(10));   // 3628800
}
