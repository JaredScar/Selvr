// ERROR: use of moved value
// EXPECT: UseAfterMove

struct Buffer {
    data: string;
}

fn consume(b: Buffer): void {
    console.log(b.data);
}

fn main(): void {
    const buf = Buffer { data: "hello" };
    consume(buf);     // buf is moved here
    console.log(buf.data);  // ERROR: buf was moved
}
