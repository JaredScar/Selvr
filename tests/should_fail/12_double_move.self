// ERROR: double move — value moved twice
// EXPECT: UseAfterMove

struct Owned {
    value: string;
}

fn take(o: Owned): void {
    console.log(o.value);
}

fn main(): void {
    const obj = Owned { value: "data" };
    take(obj);  // first move
    take(obj);  // ERROR: already moved
}
