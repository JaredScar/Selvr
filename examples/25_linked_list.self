// 25 — Singly linked list
// Demonstrates recursive enums, generics, and ownership.
// SELVR automatically boxes recursive enum variants — no Box<T> needed.

enum List<T> {
    Cons(T, List<T>),
    Nil,
}

impl<T: Clone> List<T> {
    fn new(): List<T> {
        return List.Nil;
    }

    fn prepend(value: T): List<T> {
        return List.Cons(value, this);
    }

    fn length(): i32 {
        return match this {
            List.Nil           => 0,
            List.Cons(_, tail) => 1 + tail.length(),
        };
    }

    fn head(): Option<T> {
        return match this {
            List.Nil        => None,
            List.Cons(h, _) => Some(h),
        };
    }

    fn tail(): List<T> {
        return match this {
            List.Nil           => List.Nil,
            List.Cons(_, rest) => rest,
        };
    }

    fn contains(target: T): boolean where T: Eq {
        return match this {
            List.Nil             => false,
            List.Cons(h, rest)   => h === target || rest.contains(target),
        };
    }

    fn toArray(): T[] {
        let v: T[] = [];
        let current = this;
        loop {
            match current {
                List.Nil           => break,
                List.Cons(h, rest) => {
                    v.push(h);
                    current = rest;
                }
            }
        }
        return v;
    }

    fn map<U>(f: (T) => U): List<U> {
        return match this {
            List.Nil             => List.Nil,
            List.Cons(h, rest)   => List.Cons(f(h), rest.map(f)),
        };
    }

    fn filter(pred: (T) => boolean): List<T> {
        return match this {
            List.Nil => List.Nil,
            List.Cons(h, rest) => {
                const filteredRest = rest.filter(pred);
                if pred(h.clone()) {
                    return List.Cons(h, filteredRest);
                }
                return filteredRest;
            }
        };
    }
}

fn main(): void {
    // Build: 5 → 4 → 3 → 2 → 1 → Nil
    const list = List.new<i32>()
        .prepend(1)
        .prepend(2)
        .prepend(3)
        .prepend(4)
        .prepend(5);

    console.log(`length: ${list.length()}`);              // 5
    console.log(`head: ${list.head()}`);                  // Some(5)
    console.log(`contains 3: ${list.contains(3)}`);       // true
    console.log(`contains 9: ${list.contains(9)}`);       // false

    const doubled = list.clone().map((x) => x * 2);
    console.log(`doubled: ${doubled.toArray()}`);         // [10, 8, 6, 4, 2]

    const odds = list.clone().filter((x) => x % 2 !== 0);
    console.log(`odds: ${odds.toArray()}`);               // [5, 3, 1]

    const tail = list.tail();
    console.log(`tail length: ${tail.length()}`);         // 4
}
