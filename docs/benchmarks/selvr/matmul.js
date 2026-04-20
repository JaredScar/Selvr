
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

export function matmul(a, b, n) {
  let c = [];
  let init = 0;
  while ((init < (n * n))) {
    c.push(0.0);
    init = (init + 1);
  }
  let row = 0;
  while ((row < n)) {
    let k = 0;
    while ((k < n)) {
      let aik = a[((row * n) + k)];
      let col = 0;
      while ((col < n)) {
        c[((row * n) + col)] = (c[((row * n) + col)] + (aik * b[((k * n) + col)]));
        col = (col + 1);
      }
      k = (k + 1);
    }
    row = (row + 1);
  }
  return c;
}

export function dot(a, b, n) {
  let s = 0.0;
  let i = 0;
  while ((i < n)) {
    s = (s + (a[i] * b[i]));
    i = (i + 1);
  }
  return s;
}


//# sourceMappingURL=output.js.map
