# Selvr Language Specification

**Version:** 0.1 (Draft)
**Status:** Living document — updated as the compiler is built.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Lexical Structure](#2-lexical-structure)
3. [Types](#3-types)
4. [Variables and Bindings](#4-variables-and-bindings)
5. [Functions](#5-functions)
6. [Control Flow](#6-control-flow)
7. [Pattern Matching](#7-pattern-matching)
8. [Structs](#8-structs)
9. [Enums](#9-enums)
10. [Traits and Implementations](#10-traits-and-implementations)
11. [Generics](#11-generics)
12. [Option and Result](#12-option-and-result)
13. [Collections](#13-collections)
14. [Closures](#14-closures)
15. [Async / Await](#15-async--await)
16. [The Module System](#16-the-module-system)
17. [Macros](#17-macros)
18. [Memory Model](#18-memory-model)
19. [The Standard Library](#19-the-standard-library)
20. [The Web Platform](#20-the-web-platform)
21. [Grammar Reference](#21-grammar-reference)
22. [Non-goals](#22-non-goals)

---

## 1. Overview

Selvr is a statically-typed, compiled language designed exclusively for the web platform.  Its defining feature is the **hybrid WASM/JS targeting system**: the compiler automatically analyses each function at compile time and decides whether it should run as WebAssembly (for compute-heavy code) or as native JavaScript (for DOM manipulation and UI logic).  The developer writes a single `.self` source file; the compiler emits the optimal split.

```selvr
// One language, two runtimes — the compiler decides automatically.
fn blur(pixels: f64[], radius: i32): f64[] { … }       // → WASM  (float loop)
fn onClick(e: Event): void { document.querySelector(…) } // → JS   (DOM API)
```

```
selvr build app.self
  → app.wasm       (blur + other math-heavy functions)
  → app.js         (onClick + DOM/UI functions + bridge wrappers)
  → app.loader.js  (boots WASM, wires both halves together)
```

### Design principles

| Principle | What it means in practice |
|---|---|
| **Sound type system** | No `any`, no `unsafe` type escapes. If it compiles, it is type-safe. |
| **Zero-cost abstractions** | Iterators, closures, and generics are all monomorphised at compile time. |
| **Deterministic memory** | Values are freed when their owner goes out of scope — no garbage collector. |
| **Great errors** | Every error message cites the span, explains the problem, and suggests a fix. |
| **Web-first** | DOM, fetch, timers, and events are first-class — not bolted-on via an FFI. |
| **Compiler-owned targeting** | WASM vs. JS is a compiler decision, not a developer decision. Overrides (`#[wasm]`, `#[js]`) are the exception. |

### How the targeting pass works

1. **Score** — each function is scored on a set of heuristics (see §23).  Float-heavy loops, `Math.*` calls, and numeric array I/O push the score positive (WASM).  DOM API calls and event handler signatures push it negative (JS).
2. **Decide** — functions scoring ≥ 50 are assigned to WASM; the rest go to JS.
3. **Propagate** — the call graph is traversed.  A JS function called from WASM is upgraded if profitable; a WASM function that calls a DOM API is downgraded.
4. **Emit** — `selvr-codegen` transpiles JS-targeted functions; `selvr-bytecode` + `selvr-vm` compile WASM-targeted ones.  `selvr-bridge` generates the glue.

Developers can inspect every decision with `selvr explain app.self`.

### Non-goals (see §22 for the full list)

- General-purpose systems programming (use Rust).
- Being a JavaScript superset (Selvr is a clean-slate language).
- Running on the server without a browser runtime (server target is a stretch goal).

---

## 2. Lexical Structure

### 2.1 Source files

Selvr source files use the `.self` extension and are encoded as UTF-8. There is no BOM requirement.

### 2.2 Comments

```
// This is a line comment.

/* This is a block comment.
   Block comments /* nest */ correctly. */
```

Comments are preserved in the token stream for IDE tooling but are invisible to the parser.

### 2.3 Identifiers

An identifier starts with a Unicode letter or `_`, followed by any number of Unicode letters, digits, or `_`. By convention, identifiers use **camelCase** (matching TypeScript style).

```
foo        _bar        myStruct    π
```

**Reserved keywords** (may not be used as identifiers):

```
fn      let     const   if      else    match   return
struct  enum    impl    trait   for     in      while
loop    break   continue import export mod     type
as      async   await   macro   where   this    true    false
```

**Reserved primitive type names:**

```
i32   i64   f32   f64   number   boolean   string   char   void
```

### 2.4 Literals

#### Integer literals

```
42          // decimal
0xFF        // hexadecimal
0b1010_1010 // binary
1_000_000   // underscores allowed anywhere as separators
```

Default type: `i32`. Suffix controls the type: `42i64`, `255u8` (u8 is a stretch goal).

#### Float literals

```
3.14
1.0e-9
6.022e23
```

Default type: `f64`. Suffix: `3.14f32`.

#### String literals

Double-quoted. Supports the following escape sequences:

| Escape | Meaning |
|--------|---------|
| `\n`   | newline |
| `\t`   | tab |
| `\r`   | carriage return |
| `\\`   | backslash |
| `\"`   | double quote |
| `\0`   | null byte |
| `\u{XXXX}` | Unicode scalar value |

Multi-line string literals use triple-quotes `"""`:

```
let s = """
  Hello,
  world!
""";
```

#### Character literals

Single-quoted, exactly one Unicode scalar value:

```
let c: char = 'a';
let newline: char = '\n';
```

#### Boolean literals

```
true   false
```

### 2.5 Operators

```
+   -   *   /   %         // arithmetic
&   |   ^   ~   !         // bitwise / logical
<<  >>                    // shifts
===  !==  <   >   <=  >=  // strict equality and comparison (TypeScript style)
==   !=                   // loose equality (discouraged; prefer ===)
&&  ||                    // short-circuit logical
=   +=  -=  *=  /=  %=   // assignment
=>                        // arrow function / match arm
..  ..=                   // range (exclusive / inclusive)
?                         // error propagation (unwrap or early-return)
.                         // field access and enum variant (Enum.Variant)
```

---

## 3. Types

### 3.1 Primitive types

| Type      | Description | Notes |
|-----------|-------------|-------|
| `i32`     | 32-bit signed integer | Default integer type |
| `i64`     | 64-bit signed integer | For large integers |
| `f32`     | 32-bit IEEE 754 float | |
| `f64`     | 64-bit IEEE 754 float | Default float type |
| `number`  | Alias for `f64` | Familiar to TypeScript developers |
| `boolean` | Boolean (`true` / `false`) | TypeScript-compatible name |
| `char`    | Unicode scalar value | |
| `string`  | UTF-8 string | TypeScript-compatible name |
| `void`    | No meaningful value (return type only) | Same as TypeScript |

### 3.2 Compound types

#### Arrays

Written as `T[]` — matching TypeScript's array type syntax.

```SELVR
const nums: i32[] = [1, 2, 3, 4];
const names: string[] = ["Alice", "Bob"];
```

#### Tuples

Fixed-length heterogeneous sequences. Written as `[T, U, ...]` (square brackets, like TypeScript destructuring).

```SELVR
const pair: [i32, string] = [42, "hello"];
const [x, y] = pair;   // destructuring — same as TypeScript
```

#### Function types

```SELVR
const add: (i32, i32) => i32 = (a, b) => a + b;
```

### 3.3 Named types

Structs, enums, and type aliases produce named types. See §8–9.

### 3.4 Generic types

```SELVR
let maybe: Option<i32> = Some(42);
let result: Result<str, Error> = Ok("success");
let v: Vec<i32> = Vec::new();
```

### 3.5 Type inference

Type annotations are **optional** when the compiler can infer the type. The compiler uses Hindley-Milner inference with let-polymorphism.

```SELVR
const x = 5;          // inferred: i32
const s = "hello";    // inferred: string
const v: i32[] = [];  // explicit annotation required when element type is ambiguous
```

When inference is ambiguous, the compiler requires an explicit annotation and emits a descriptive error.

---

## 4. Variables and Bindings

Selvr uses the same `const` / `let` distinction as TypeScript — no mutation annotations needed.

### 4.1 `const` — immutable binding

`const` declares a binding that cannot be reassigned. This is the default and preferred choice.

```SELVR
const x = 42;
const name: string = "Alice";
```

Shadowing is allowed — a new `const` with the same name creates a fresh binding:

```SELVR
const x = 5;
const x = x + 1; // new binding — original x is gone
```

### 4.2 `let` — mutable binding

`let` declares a binding that may be reassigned after its initial value.

```SELVR
let count = 0;
count += 1;   // ok
count = 10;   // ok
```

### 4.3 Compile-time constants

Module-level `const` declarations are evaluated at compile time.

```SELVR
const MAX_ITEMS: i32 = 1024;
const PI: f64 = 3.141592653589793;
```

### 4.4 Destructuring

Any pattern can appear on the left of `const` or `let`:

```SELVR
const [a, b] = [1, 2];          // array/tuple destructuring
const [head, ..rest] = items;
const Point { x, y } = point;   // struct destructuring
```

---

## 5. Functions

### 5.1 Basic functions

The return type comes **after the parameter list with a colon** — exactly like TypeScript.

```SELVR
fn add(a: i32, b: i32): i32 {
    return a + b;
}
```

The `: void` annotation is optional when the function returns nothing:

```SELVR
fn greet(name: string): void {
    console.log(`Hello, ${name}!`);
}

fn greet(name: string) {   // also valid — void is implicit
    console.log(`Hello, ${name}!`);
}
```

### 5.2 Implicit return

The last expression in a block with no trailing semicolon is the return value:

```SELVR
fn square(n: i32): i32 {
    n * n   // no semicolon — returned automatically
}
```

### 5.3 Multiple return values

```SELVR
fn divmod(a: i32, b: i32): [i32, i32] {
    return [a / b, a % b];
}

const [quotient, remainder] = divmod(17, 5);
```

### 5.4 Exported functions

```SELVR
export fn add(a: i32, b: i32): i32 {
    return a + b;
}
```

### 5.5 Async functions

```SELVR
async fn fetchUser(id: i32): Result<User, Error> {
    const resp = fetch(`/api/users/${id}`).await?;
    const user = resp.json<User>().await?;
    return Ok(user);
}
```

### 5.6 Methods

Methods live in an `impl` block. There is no explicit `self` parameter — use `this` inside the body, just like TypeScript classes.

```SELVR
impl Point {
    fn distanceFromOrigin(): f64 {
        return Math.sqrt(this.x * this.x + this.y * this.y);
    }
}
```

---

## 6. Control Flow

### 6.1 `if` / `else`

```SELVR
if x > 0 {
    console.log("positive");
} else if x < 0 {
    console.log("negative");
} else {
    console.log("zero");
}
```

`if` is an **expression** — it can produce a value:

```SELVR
const label = if score > 90 { "A" } else { "B" };
```

Both branches must have the same type.

### 6.2 `while`

```SELVR
let i = 0;
while i < 10 {
    console.log(i);
    i += 1;
}
```

### 6.3 `loop`

An infinite loop. Use `break` to exit, optionally with a value:

```SELVR
const result = loop {
    const input = readLine();
    if input === "quit" {
        break 0;
    }
};
```

### 6.4 `for` … `in`

Iterates over any value implementing the `Iterator` trait:

```SELVR
for item in collection {
    console.log(item);
}

for i in 0..10 {        // exclusive range
    console.log(i);
}

for i in 0..=10 {       // inclusive range
    console.log(i);
}
```

### 6.5 `break` and `continue`

```SELVR
for i in 0..100 {
    if i % 2 === 0 { continue; }
    if i > 20      { break; }
    console.log(i);
}
```

Loops may be labelled:

```SELVR
'outer: for i in 0..10 {
    for j in 0..10 {
        if i == j { break 'outer; }
    }
}
```

### 6.6 `return`

```SELVR
fn find(xs: i32[], target: i32): Option<i32> {
    for x in xs {
        if x === target { return Some(x); }
    }
    return None;
}
```

---

## 7. Pattern Matching

### 7.1 `match` expression

`match` is exhaustive — the compiler rejects non-exhaustive patterns. Think of it as a `switch` that actually works.

```SELVR
match status {
    200 => console.log("OK"),
    404 => console.log("Not Found"),
    500 => console.log("Server Error"),
    _   => console.log("Unknown"),
}
```

`match` is an expression:

```SELVR
const label = match status {
    200 => "OK",
    404 => "Not Found",
    _   => "Other",
};
```

### 7.2 Pattern kinds

#### Literal patterns

```SELVR
match x {
    0     => "zero",
    1..=9 => "single digit",
    _     => "other",
}
```

#### Variable binding

```SELVR
match point {
    Point { x: 0, y } => console.log(`on y-axis at ${y}`),
    Point { x, y: 0 } => console.log(`on x-axis at ${x}`),
    Point { x, y }    => console.log(`at (${x}, ${y})`),
}
```

#### Enum patterns

```SELVR
match maybe {
    Some(value) => console.log(`got ${value}`),
    None        => console.log("nothing"),
}
```

#### Tuple patterns

```SELVR
match (a, b) {
    (0, 0) => "origin",
    (x, 0) => "x-axis at {x}",
    (0, y) => "y-axis at {y}",
    (x, y) => "({x}, {y})",
}
```

#### Guard clauses

```SELVR
match n {
    x if x < 0  => "negative",
    x if x == 0 => "zero",
    x            => "positive: {x}",
}
```

#### Or patterns

```SELVR
match c {
    'a' | 'e' | 'i' | 'o' | 'u' => "vowel",
    _                            => "consonant",
}
```

### 7.3 `let`-pattern destructuring

Any `match` pattern can appear in a `let` binding:

```SELVR
let Some(value) = maybe else { return; };
let (first, second) = pair;
let [head, ..tail] = list;
```

---

## 8. Structs

Struct fields use semicolons as terminators — consistent with TypeScript interface properties.

### 8.1 Definition

```SELVR
struct Point {
    x: f64;
    y: f64;
}

export struct User {
    name: string;
    email: string;
    age: i32;
}
```

### 8.2 Instantiation

```SELVR
const p = Point { x: 1.0, y: 2.0 };
const p2 = Point { x: 3.0, ..p }; // struct update syntax — copies remaining fields from p
```

### 8.3 Field access

```SELVR
const distance = Math.sqrt(p.x * p.x + p.y * p.y);
```

### 8.4 Unit structs

```SELVR
struct Marker;  // zero-size type — useful as trait markers
```

---

## 9. Enums

### 9.1 Definition

```SELVR
enum Direction {
    North,
    South,
    East,
    West,
}

enum Shape {
    Circle(f64),                           // tuple variant
    Rectangle { width: f64; height: f64; }, // struct variant
    Point,                                  // unit variant
}
```

### 9.2 Constructing variants

Enum variants are accessed with dot notation — the same as TypeScript enum access.

```SELVR
const dir    = Direction.North;
const circle = Shape.Circle(5.0);
const rect   = Shape.Rectangle { width: 10.0, height: 4.0 };
```

### 9.3 Matching enums

```SELVR
const area = match shape {
    Shape.Circle(r)                   => Math.PI * r * r,
    Shape.Rectangle { width, height } => width * height,
    Shape.Point                       => 0.0,
};
```

---

## 10. Traits and Implementations

Traits are like TypeScript interfaces — but they can have default method bodies and be used as generic constraints.

### 10.1 Defining a trait

```SELVR
trait Describable {
    fn describe(): string;
    fn shortDesc(): string {  // default implementation
        return this.describe();
    }
}
```

### 10.2 Implementing a trait

```SELVR
impl Describable for Point {
    fn describe(): string {
        return `Point(${this.x}, ${this.y})`;
    }
}
```

### 10.3 Inherent impl (methods without a trait)

Methods use `this` to access the current instance — no explicit `self` parameter.

```SELVR
impl Point {
    fn new(x: f64, y: f64): Point {
        return Point { x, y };
    }

    fn distanceFromOrigin(): f64 {
        return Math.sqrt(this.x * this.x + this.y * this.y);
    }
}

const p = Point.new(3.0, 4.0);
const d = p.distanceFromOrigin(); // 5.0
```

### 10.4 Trait bounds

```SELVR
fn print_all<T: Describable>(items: [T]) {
    for item in items {
        print(item.describe());
    }
}
```

### 10.5 Built-in traits

| Trait | Meaning |
|-------|---------|
| `Clone` | Value can be explicitly duplicated |
| `Copy` | Value is bitwise-copyable (no explicit `.clone()` needed) |
| `Display` | Can be formatted as a string |
| `Debug` | Can be formatted for debug output |
| `Iterator` | Produces a sequence of values |
| `Eq` / `PartialEq` | Value equality |
| `Ord` / `PartialOrd` | Value ordering |
| `Hash` | Value can be hashed |
| `Default` | Has a sensible zero/empty value |

---

## 11. Generics

Selvr generics use the same `<T>` syntax as TypeScript. Unlike TypeScript, generics are monomorphised at compile time — no type erasure.

### 11.1 Generic functions

```SELVR
fn first<T>(xs: T[]): Option<T> {
    if xs.length === 0 {
        return None;
    }
    return Some(xs[0]);
}
```

### 11.2 Generic structs

```SELVR
struct Pair<A, B> {
    first: A;
    second: B;
}

const p = Pair { first: 1, second: "one" };
```

### 11.3 Multiple bounds

Use `&` to combine bounds (similar to TypeScript's `A & B` intersection):

```SELVR
fn printSorted<T: Ord & Display>(xs: T[]): void {
    xs.sort();
    for x in xs { console.log(x); }
}
```

### 11.4 Where clauses

For complex bounds, `where` improves readability:

```SELVR
fn zip<A, B>(a: A[], b: B[]): [A, B][]
where
    A: Clone,
    B: Clone,
{
    // ...
}
```

---

## 12. Option and Result

There is no `null` in Selvr. Absence and fallibility are modelled with two standard enum types.

### 12.1 `Option<T>`

```SELVR
enum Option<T> {
    Some(T),
    None,
}
```

Usage:

```SELVR
fn findUser(id: i32): Option<User> {
    if id === 0 { return None; }
    return Some(db.get(id));
}

match findUser(42) {
    Some(user) => console.log(user.name),
    None       => console.log("not found"),
}

// ? operator propagates None up as a return value
const name = findUser(42)?.name;
```

### 12.2 `Result<T, E>`

```SELVR
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Usage:

```SELVR
fn parseInt(s: string): Result<i32, ParseError> {
    // ...
}

const n = parseInt("42")?;  // ? propagates Err, unwraps Ok
```

### 12.3 The `?` operator

Within a function returning `Option<T>` or `Result<T, E>`, the `?` postfix operator:
- For `Option`: returns `None` from the enclosing function if the value is `None`.
- For `Result`: returns `Err(e)` from the enclosing function if the value is `Err(e)`.
- Otherwise: unwraps the inner value.

---

## 13. Collections

Standard collections live in `std/collections`. Method names mirror TypeScript/JavaScript's built-in APIs where possible.

### `T[]` — growable array

```SELVR
let v: i32[] = [];
v.push(1);
v.push(2);
v.push(3);
console.log(v.length);  // 3

// Array methods — same as TypeScript
const doubled  = v.map((x) => x * 2);
const evens    = v.filter((x) => x % 2 === 0);
const total    = v.reduce((acc, x) => acc + x, 0);
```

### `Map<K, V>` — hash map

```SELVR
import { Map } from "std/collections";

const scores: Map<string, i32> = Map.new();
scores.set("Alice", 100);
scores.set("Bob", 90);
console.log(scores.get("Alice")); // Some(100)
```

### `Set<T>` — hash set

```SELVR
import { Set } from "std/collections";

const seen: Set<i32> = Set.new();
seen.add(1);
const isNew     = seen.add(2); // true — was not present
const alreadyIn = seen.add(1); // false — already present
```

---

## 14. Closures

Closures use **arrow function syntax** — identical to TypeScript.

```SELVR
const multiplier = 3;
const triple = (x: i32): i32 => x * multiplier;

console.log(triple(7)); // 21
```

Single-expression closures omit braces and `return`:

```SELVR
const doubled = numbers.map((x) => x * 2);
const evens   = numbers.filter((x) => x % 2 === 0);
const total   = numbers.reduce((acc, x) => acc + x, 0);
```

Multi-statement closures use a block body:

```SELVR
const process = (x: i32): string => {
    const doubled = x * 2;
    return `result: ${doubled}`;
};
```

Optional type annotations (inferred when possible):

```SELVR
const add = (a: i32, b: i32): i32 => a + b;  // fully annotated
const inc = (x) => x + 1;                     // types inferred
```

---

## 15. Async / Await

Selvr's async model integrates directly with the browser event loop — no runtime thread pool needed.

### 15.1 Declaring async functions

```SELVR
async fn getData(url: string): Result<string, FetchError> {
    const response = fetch(url).await?;
    const text = response.text().await?;
    Ok(text)
}
```

### 15.2 Awaiting

The `.await` postfix keyword suspends the current async function until the future resolves.

```SELVR
let data = get_data("/api/data").await?;
```

### 15.3 Concurrency

Run multiple futures concurrently with `join!` and `select!`:

```SELVR
let (a, b) = join!(fetch_a(), fetch_b()).await;

let first = select! {
    a = fetch_a().await => a,
    b = fetch_b().await => b,
};
```

### 15.4 Restrictions

- `await` is only valid inside an `async fn`.
- The top-level entry point on the web is `async fn main()`.

---

## 16. The Module System

Selvr uses **ES-module style** `import` / `export` — identical to TypeScript.

### 16.1 Exporting items

Items are private by default. Prefix with `export` to make them visible to other modules:

```SELVR
export struct Point { x: f64; y: f64; }
export fn distance(a: Point, b: Point): f64 { ... }
export const VERSION: string = "1.0.0";
```

### 16.2 Importing items

```SELVR
import { Point, distance }   from "./geometry";
import { Map, Set }          from "std/collections";
import { fetch }             from "std/web";
import * from "std/web/dom"; // import everything into scope
```

### 16.3 Aliased imports

```SELVR
import { reallyLongName as short } from "./utils";
```

### 16.4 Inline modules

Inline `mod` blocks work like TypeScript namespaces:

```SELVR
mod geometry {
    export struct Point { x: f64; y: f64; }
    export fn distance(a: Point, b: Point): f64 { ... }
}
```

Or declared as a file — `mod geometry;` in the parent loads `geometry.self`.

### 16.5 Visibility summary

| Keyword  | Visible to |
|----------|-----------|
| (none)   | Current module only |
| `export` | All importers |

### 16.6 The standard library

The standard library uses slash-delimited paths:

```
std/collections   // Map, Set, Vec, etc.
std/string        // String manipulation
std/option        // Option<T> helpers
std/result        // Result<T, E> helpers
std/iter          // Iterator combinators
std/fmt           // Formatting
std/web/dom       // DOM API bindings
std/web/fetch     // HTTP client
std/web/events    // Event listeners
std/web/socket    // WebSocket API
```

---

## 17. Macros

Macros are expanded at compile time and produce zero runtime overhead.

### 17.1 Built-in macros

| Macro | Description |
|-------|-------------|
| `print!(...)` | Print to console |
| `println!(...)` | Print to console with newline |
| `format!(...)` | Format a string |
| `vec![...]` | Create a Vec |
| `assert!(cond)` | Panic at compile time if false |
| `todo!()` | Marks unfinished code; panics at runtime |
| `unreachable!()` | Marks code that should never execute |
| `include_str!(path)` | Embed a file's contents as a `str` at compile time |

### 17.2 User-defined macros (declarative)

```SELVR
macro html(tag, content) {
    dom::create_element($tag).set_inner_html($content)
}

let div = html!("div", "<p>Hello</p>");
```

Procedural macros (arbitrary compile-time code generation) are a Phase 3 feature.

---

## 18. Memory Model

Selvr uses an **ownership model** with compiler-inferred ownership rather than explicit borrow syntax.

### 18.1 Ownership rules

1. Each value has exactly one owner.
2. When the owner goes out of scope, the value is freed (deterministically).
3. Moving a value transfers ownership — the original binding is invalid.

```SELVR
const a: i32[] = [1, 2, 3];
const b = a;       // a is moved into b
// console.log(a); // compile error: use of moved value
```

### 18.2 Copies vs. moves

Types that implement `Copy` are implicitly copied rather than moved. All primitive types (`i32`, `f64`, `boolean`, etc.) are `Copy`.

```SELVR
const x: i32 = 5;
const y = x;          // x is copied, not moved
console.log(x);       // valid — x still owns its value
```

### 18.3 Cloning

Non-`Copy` types can be explicitly duplicated with `.clone()`:

```SELVR
const a = [1, 2, 3];
const b = a.clone();  // deep copy
console.log(a);       // valid — a was cloned, not moved
```

### 18.4 Immutable references (implicit)

The compiler automatically borrows values for function calls when possible. Developers rarely need to think about borrowing in everyday code.

```SELVR
fn len(s: string): i32 { return s.length as i32; }
let s = "hello";
let n = len(s);     // s is borrowed implicitly; s is still valid
console.log(s);     // valid — compiler borrowed s implicitly
```

### 18.5 Mutation

A `let` binding can be mutated. Pass by value to functions that need to modify data:

```SELVR
fn pushThree(v: i32[]): void {
    v.push(1);
    v.push(2);
    v.push(3);
}
```

### 18.6 No dangling pointers

The compiler rejects any code that could produce a reference that outlives the value it points to.

---

## 19. The Standard Library

A full API reference will ship separately. Key types and modules:

### `std::string`

- `string` — owned, growable UTF-8 string (TypeScript-compatible name).
- String interpolation uses **backtick template literals** — identical to TypeScript:

```SELVR
const name = "world";
const greeting = `Hello, ${name}!`;
const expr = `2 + 2 = ${2 + 2}`;
```

### `std::iter`

```SELVR
const total: i32 = (1..=100)
    .filter((n) => n % 2 === 0)
    .map((n) => n * n)
    .sum();
```

### `std::fmt`

```SELVR
let s = format!("{:.2}", 3.14159); // "3.14"
```

### `std::io`

Reading from the DOM and writing to the console:

```SELVR
print!("hello");
println!("world");
let line = dom::read_value("#my-input");
```

---

## 20. The Web Platform

Selvr treats the browser as its native runtime. DOM, events, and fetch are not an FFI — they are part of the standard library with a familiar TypeScript-like API.

### 20.1 DOM manipulation

```SELVR
import { query, create } from "std/web/dom";

const button = query("#submit").unwrap();
button.setText("Click me");
button.addClass("active");
button.on("click", (_) => {
    query("#output").unwrap().setText("Clicked!");
});
```

### 20.2 HTTP / fetch

```SELVR
import { fetch } from "std/web";

async fn loadPosts(): Result<Post[], FetchError> {
    const posts = fetch("https://api.example.com/posts")
        .await?
        .json<Post[]>()
        .await?;
    return Ok(posts);
}
```

### 20.3 Events

```SELVR
import { KeyboardEvent } from "std/web/events";

window.on("keydown", (e: KeyboardEvent) => {
    if e.key === "Enter" {
        submitForm();
    }
});
```

### 20.4 Timers

```SELVR
import { setTimeout, setInterval, clearInterval } from "std/web/time";

setTimeout(() => { console.log("fired!"); }, 1000);
const handle = setInterval(() => { tick(); }, 16);
clearInterval(handle);
```

### 20.5 WebSockets

```SELVR
import { WebSocket } from "std/web/socket";

const ws = WebSocket.connect("wss://example.com/chat").await?;
ws.send("hello").await?;
ws.onMessage((msg) => { console.log(msg.data); });
```

### 20.6 Entry point

Every Selvr web application defines a `main` function. The runtime calls it after the DOM is ready:

```SELVR
async fn main() {
    let app = App::new();
    app.mount("#root").await;
}
```

---

## 21. Grammar Reference

A simplified EBNF grammar. Square brackets `[…]` denote optional elements, `{…}` denotes zero or more repetitions, `|` denotes alternatives.

```ebnf
module      = { item } ;
item        = visibility ( fn_def | struct_def | enum_def | trait_def
                         | impl_block | type_alias | use_decl
                         | mod_decl | const_item | macro_def ) ;
visibility  = [ "pub" ] ;

fn_def      = [ "async" ] "fn" IDENT generic_params fn_params [ "->" type ] block ;
fn_params   = "(" [ param { "," param } ] ")" ;
param       = IDENT ":" type ;

struct_def  = "struct" IDENT generic_params "{" { struct_field "," } "}" ;
struct_field = visibility IDENT ":" type ;

enum_def    = "enum" IDENT generic_params "{" { variant "," } "}" ;
variant     = IDENT [ "(" { type "," } ")" | "{" { struct_field "," } "}" ] ;

trait_def   = "trait" IDENT generic_params "{" { trait_item } "}" ;
trait_item  = fn_sig | fn_def | type_assoc ;
fn_sig      = [ "async" ] "fn" IDENT generic_params fn_params [ "->" type ] ";" ;

impl_block  = "impl" [ generic_params type "for" ] type "{" { impl_item } "}" ;
impl_item   = fn_def | const_item | type_assoc ;

type_alias  = "type" IDENT generic_params "=" type ";" ;
use_decl    = "use" use_tree ";" ;
mod_decl    = "mod" IDENT ( "{" { item } "}" | ";" ) ;
const_item  = "const" IDENT [ ":" type ] "=" expr ";" ;

generic_params = [ "<" generic_param { "," generic_param } ">" ] ;
generic_param  = IDENT { ":" type { "+" type } } ;

type        = primitive | IDENT [ "<" type { "," type } ">" ]
            | "(" { type "," } ")"
            | "[" type [ ";" expr ] "]"
            | "fn" "(" { type "," } ")" "->" type
            | "void" ;

block       = "{" { stmt } [ expr ] "}" ;
stmt        = let_stmt | expr_stmt | item ;
let_stmt    = "let" [ "mut" ] pattern [ ":" type ] [ "=" expr ] ";" ;
expr_stmt   = expr ";" ;

expr        = assign | range | or | and | equality | comparison
            | additive | multiplicative | unary | postfix | primary ;

pattern     = "_" | IDENT | literal | tuple_pat | struct_pat | array_pat | or_pat ;
```

---

## 22. Non-goals

The following are explicit non-goals for Selvr 1.0. They may be revisited in future versions.

1. **JavaScript interoperability.** Selvr does not aim to call existing JS libraries. An FFI layer is a future stretch goal.
2. **Server-side / CLI runtime.** Selvr targets browsers. A Node.js-style server runtime is a stretch goal.
3. **Metaprogramming beyond declarative macros.** Procedural macros and reflection are deferred to a later version.
4. **Weak types or runtime type coercion.** `1 + "1"` is a compile error.
5. **Undefined behaviour.** Selvr programs that compile have fully defined semantics.
6. **Backwards compatibility with JavaScript.** Selvr is a clean-slate language. It does not inherit JS's type coercion, hoisting, or `this` semantics.
7. **A package registry for 1.0.** The package manager CLI will ship, but the public registry launches after 1.0 with curated packages only.

---

## 23. Hybrid WASM/JS Targeting System

### 23.1 Overview

Every function in a Selvr module is assigned to exactly one runtime target:

| Target | Artefact | Best for |
|--------|----------|----------|
| `wasm` | `app.wasm` | Float-heavy loops, matrix math, image/audio DSP, 3D rendering |
| `js`   | `app.js`   | DOM queries, event handlers, form logic, lightweight string ops |

The developer writes one `.self` file.  The compiler decides.

### 23.2 Targeting attributes

Developers can override the compiler's decision with function attributes:

```selvr
#[wasm]
fn convolve(kernel: f64[], image: f64[]): f64[] { … }

#[js]
fn renderList(items: string[]): void {
    document.querySelector("#list").innerHTML = items.join("<li>");
}
```

A `#[wasm]` function that calls a DOM API is a **compile error** — not a warning.  This prevents accidental bridge overhead from proliferating.

### 23.3 Auto-inference scoring

Functions without an explicit attribute are scored:

**WASM signals (positive)**

| Condition | Score |
|-----------|-------|
| Returns `f64[]` or `i32[]` | +40 |
| Parameter is `f64[]` or `i32[]` | +20 per param |
| Struct parameter with ≥ 4 numeric fields | +30 |
| Contains a nested numeric `for` loop | +50 |
| Calls `Math.*` ≥ 3 times | +25 |
| Multiply inside a loop (FMA pattern) | +30 |
| ≥ 10 numeric ops inside a loop body | +50 |
| Annotated `#[wasm]` | +∞ (forced) |

**JS signals (negative)**

| Condition | Score |
|-----------|-------|
| Calls `document.*` / `window.*` | −∞ (forced JS) |
| References `Event`, `Element`, `HTMLElement` | −500 |
| Is an `async` event handler | −200 |
| Calls `console.log` | −10 |
| Annotated `#[js]` | −∞ (forced) |

**Decision rule:** score ≥ 50 → WASM, otherwise → JS (conservative default).

### 23.4 Call-graph propagation

After initial scoring, the compiler traverses the call graph and applies two propagation rules:

1. **Downgrade:** a WASM-scored function that calls a JS-only function is downgraded to JS to avoid a mandatory bridge crossing at every call site.
2. **Upgrade:** a JS-scored function whose *entire* caller set is WASM, and whose own score is positive, is upgraded to WASM to eliminate the bridge crossing.

The propagation runs to a fixed point (no more changes).

### 23.5 Bridge generation

For every WASM-targeted function that is exported from a module, the compiler generates a JS wrapper in `app.bridge.js`:

```js
// auto-generated by selvr-bridge
export async function convolve(kernel, image) {
  const _args = JSON.stringify([kernel, image]);   // or zero-copy (see §23.6)
  const _res  = selvr_vm.selvr_call("convolve", _args);
  return JSON.parse(_res);
}
```

For every JS-targeted function called *from* a WASM function, the compiler generates a WASM import stub that the browser satisfies at `WebAssembly.instantiate` time.

### 23.6 Zero-copy typed-array fast path

When a bridge call's parameter or return type is `f64[]` or `i32[]`, and the JS caller already holds a `Float64Array` / `Int32Array`, the bridge skips JSON serialisation:

1. Allocates a region in WASM linear memory (`selvr_alloc`).
2. Writes the typed array's buffer directly via a `DataView` on `WebAssembly.Memory`.
3. Passes only a pointer + length to the WASM function.
4. Reads the result region back via the same `DataView`.
5. Frees the region (`selvr_free`).

This reduces bridge overhead for a 1 M-element `f64[]` from ~8 ms (JSON) to ~50 µs.

### 23.7 Developer visibility

```bash
selvr explain app.self
```

Prints a human-readable report for every function:

```
=== Selvr targeting report ===

  ⚙ wasm  blur
         reason: ≥10 numeric ops inside loop body; calls Math.sqrt 4 times

  ⚡ js   onClick
         reason: calls a DOM API (document.querySelector)

  ⚙ wasm  matMul
         reason: multiply inside a loop (FMA-like pattern); returns f64[]

Summary: 2 wasm  1 js  0 undecided
```

`selvr build --emit hybrid` writes a `split-report.json` with the full machine-readable breakdown.
