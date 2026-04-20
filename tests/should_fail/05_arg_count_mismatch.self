// ERROR: wrong number of arguments
// EXPECT: ArgCountMismatch

fn greet(name: string, title: string): string {
    return `Hello, ${title} ${name}!`;
}

fn main(): void {
    console.log(greet("Alice"));  // ERROR: expects 2 args, got 1
}
