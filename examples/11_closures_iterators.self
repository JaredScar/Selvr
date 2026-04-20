// 11 — Closures and iterators
// Closures use arrow syntax — exactly like TypeScript arrow functions.
// Iterator methods (map, filter, reduce) work the same as in TypeScript.

fn apply<T>(f: (T) => T, x: T): T {
    return f(x);
}

fn main(): void {
    // Arrow function closures — identical to TypeScript
    const double  = (x: i32) => x * 2;
    const addOne  = (x: i32) => x + 1;

    console.log(apply(double, 5));   // 10
    console.log(apply(addOne, 5));   // 6

    // Capturing from the enclosing scope
    const factor = 7;
    const multiply = (x: i32) => x * factor;
    console.log(multiply(6));  // 42

    // Iterator chain — map, filter, reduce work just like TypeScript's Array methods
    const numbers: i32[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    const result = numbers
        .filter((n) => n % 2 === 0)
        .map((n) => n * n);

    console.log(result); // [4, 16, 36, 64, 100]

    // sum, count
    const sum: i32 = (1..=100).sum();
    console.log(`Sum 1..100 = ${sum}`); // 5050

    const evenCount = (1..=100).filter((n) => n % 2 === 0).count();
    console.log(`Even numbers 1..100: ${evenCount}`); // 50

    // flatMap — same as TypeScript
    const sentences = ["hello world", "foo bar"];
    const allWords = sentences.flatMap((s) => s.split(" "));
    console.log(allWords); // ["hello", "world", "foo", "bar"]

    // reduce — equivalent to TypeScript's Array.reduce
    const product = [1, 2, 3, 4, 5].reduce((acc, n) => acc * n, 1);
    console.log(`5! = ${product}`); // 120

    // some / every — same names as TypeScript
    const hasNegative = [-1, 2, 3].some((n) => n < 0);
    const allPositive = [1, 2, 3].every((n) => n > 0);
    console.log(`hasNegative = ${hasNegative}`); // true
    console.log(`allPositive = ${allPositive}`); // true

    // zip two arrays together
    const names  = ["Alice", "Bob", "Carol"];
    const scores = [95, 87, 92];
    for [name, score] in names.zip(scores) {
        console.log(`${name}: ${score}`);
    }
}
