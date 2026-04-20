
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

export function sieve(limit) {
  let composite = [];
  let i = 0;
  while ((i <= limit)) {
    composite.push(false);
    i = (i + 1);
  }
  let count = 0;
  let p = 2;
  while ((p <= limit)) {
    if (!(composite[p])) {
      count = (count + 1);
      let multiple = (p + p);
      while ((multiple <= limit)) {
        composite[multiple] = true;
        multiple = (multiple + p);
      }
    }
    p = (p + 1);
  }
  return count;
}


//# sourceMappingURL=output.js.map
