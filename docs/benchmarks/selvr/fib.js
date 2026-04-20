
// Selvr runtime — DO NOT EDIT (generated)
const __selvr = {
  Some: (v) => ({ tag: "Some", val: v }),
  None: { tag: "None" },
  isSome: (o) => o.tag === "Some",
  isNone: (o) => o.tag === "None",
  unwrap: (o) => { if (o.tag !== "Some") throw new Error("unwrap called on None"); return o.val; },
  Ok: (v) => ({ tag: "Ok", val: v }),
  Err: (e) => ({ tag: "Err", err: e }),
  panic: (msg) => { throw new Error("SELVR panic: " + msg); },
  print: (...args) => console.log(...args),
};

export function fib(n) {
  let a = 0;
  let b = 1;
  let i = 0;
  while ((i < n)) {
    let tmp = (a + b);
    a = b;
    b = tmp;
    i = (i + 1);
  }
  return a;
}

export function fib_rec(n) {
  if ((n <= 1)) {
    return n;
  }
  return (fib_rec((n - 1)) + fib_rec((n - 2)));
}


//# sourceMappingURL=output.js.map
