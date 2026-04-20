// 16 — Collections
// Array<T> (written T[]), Map<K, V>, Set<T>
// Method names mirror TypeScript's built-in collection APIs where possible.

import { Map, Set } from "std/collections";

fn wordCount(text: string): Map<string, i32> {
    const counts: Map<string, i32> = Map.new();
    for word in text.split(" ") {
        const count = counts.get(word).unwrapOr(0);
        counts.set(word, count + 1);
    }
    return counts;
}

fn uniqueSorted(numbers: i32[]): i32[] {
    const set: Set<i32> = Set.from(numbers);
    const v = set.toArray();
    v.sort();
    return v;
}

fn main(): void {
    // Array — methods match TypeScript's Array API
    let v: i32[] = [];
    for i in 0..10 { v.push(i * i); }
    console.log(v);                          // [0, 1, 4, 9, 16, 25, 36, 49, 64, 81]
    console.log(v.includes(25));             // true
    v = v.filter((x) => x % 2 === 0);       // keep only even squares
    console.log(v);                          // [0, 4, 16, 36, 64]

    // Slicing — same as TypeScript's .slice()
    const slice = v.slice(1, 3);
    console.log(slice);                      // [4, 16]

    // Map — same API as TypeScript's Map
    const counts = wordCount("the quick brown fox jumps over the lazy dog the");
    console.log(counts.get("the"));          // Some(3)
    console.log(counts.get("cat"));          // None
    for [word, count] in counts.entries() {
        if count > 1 {
            console.log(`"${word}" appears ${count} times`);
        }
    }

    // Set — like TypeScript's Set
    const a: Set<i32> = Set.from([1, 2, 3, 4, 5]);
    const b: Set<i32> = Set.from([3, 4, 5, 6, 7]);
    const intersection = a.intersection(b).toArray();
    const union        = a.union(b).toArray();
    console.log(`intersection: ${intersection}`);
    console.log(`union: ${union}`);

    const dupes = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
    console.log(uniqueSorted(dupes)); // [1, 2, 3, 4, 5, 6, 9]
}
