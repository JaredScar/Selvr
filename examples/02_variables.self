// 02 — Variables and bindings
// const = immutable (like TypeScript const)
// let   = mutable   (like TypeScript let)

const MAX: i32 = 100;

fn main(): void {
    // Immutable — type inferred as i32
    const x = 42;

    // Explicit type annotation
    const name: string = "SELVR";

    // Mutable binding — reassignable
    let count = 0;
    count += 1;
    count += 1;

    // Shadowing — a new const with the same name is valid
    const doubled = x * 2;

    console.log(`x = ${x}`);           // 42
    console.log(`doubled = ${doubled}`); // 84
    console.log(`name = ${name}`);
    console.log(`count = ${count}`);    // 2
    console.log(`MAX = ${MAX}`);        // 100
}
