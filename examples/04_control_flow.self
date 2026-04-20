// 04 — Control flow
// if/else expressions, while, loop, for..in, ranges.

fn fizzbuzz(n: i32): string {
    if n % 15 === 0 {
        return "FizzBuzz";
    } else if n % 3 === 0 {
        return "Fizz";
    } else if n % 5 === 0 {
        return "Buzz";
    } else {
        return `${n}`;
    }
}

fn main(): void {
    // if as an expression — both branches must have the same type
    const label = if 42 > 0 { "positive" } else { "non-positive" };
    console.log(label);

    // for..in with a range (exclusive end)
    for i in 1..=20 {
        console.log(fizzbuzz(i));
    }

    // while loop
    let n = 1;
    while n < 1024 {
        n *= 2;
    }
    console.log(`First power of 2 >= 1024: ${n}`);

    // loop with a break value
    let attempts = 0;
    const result = loop {
        attempts += 1;
        if attempts >= 5 {
            break attempts;
        }
    };
    console.log(`Stopped after ${result} attempts`);

    // Iterating over a collection
    const words = ["SELVR", "is", "fast"];
    for word in words {
        console.log(word);
    }
}
