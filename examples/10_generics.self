// 10 — Generics
// SELVR generics work like TypeScript generics — <T> syntax, constraints with :.
// Unlike TypeScript, generics are monomorphised at compile time (zero runtime cost).

fn first<T>(xs: T[]): Option<T> {
    if xs.length === 0 {
        return None;
    }
    return Some(xs[0]);
}

fn last<T>(xs: T[]): Option<T> {
    if xs.length === 0 {
        return None;
    }
    return Some(xs[xs.length - 1]);
}

// Generic struct — like a TypeScript generic class/interface
struct Pair<A, B> {
    first: A;
    second: B;
}

impl<A, B> Pair<A, B> {
    fn new(first: A, second: B): Pair<A, B> {
        return Pair { first, second };
    }

    fn swap(): Pair<B, A> {
        return Pair { first: this.second, second: this.first };
    }
}

// Generic stack data structure
struct Stack<T> {
    items: T[];
}

impl<T> Stack<T> {
    fn new(): Stack<T> {
        return Stack { items: [] };
    }

    fn push(item: T): void {
        this.items.push(item);
    }

    fn pop(): Option<T> {
        return this.items.pop();
    }

    fn peek(): Option<T> {
        return this.items.last();
    }

    fn isEmpty(): boolean {
        return this.items.length === 0;
    }

    fn size(): i32 {
        return this.items.length as i32;
    }
}

// Where clause — cleaner when bounds get complex
fn zip<A, B>(a: A[], b: B[]): Pair<A, B>[]
where
    A: Clone,
    B: Clone,
{
    const len = Math.min(a.length, b.length);
    let result: Pair<A, B>[] = [];
    for i in 0..len {
        result.push(Pair.new(a[i].clone(), b[i].clone()));
    }
    return result;
}

fn main(): void {
    const nums = [1, 2, 3, 4, 5];
    console.log(first(nums));  // Some(1)
    console.log(last(nums));   // Some(5)

    const pair = Pair.new(42, "hello");
    console.log(pair.first);   // 42
    const swapped = pair.swap();
    console.log(swapped.first); // hello

    const stack: Stack<i32> = Stack.new();
    stack.push(10);
    stack.push(20);
    stack.push(30);
    console.log(stack.peek());  // Some(30)
    console.log(stack.pop());   // Some(30)
    console.log(stack.size());  // 2
}
