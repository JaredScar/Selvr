/**
 * Minimal WASM modules for browser demos — same technique as docs/benchmarks/hybrid.html.
 * Loaded before selvr/game.js; exposes window.SelvrGamesWasm.
 */
(function (global) {
  const I32 = 0x7f;
  const F64 = 0x7c;

  function uleb(n) {
    const o = [];
    do {
      let b = n & 0x7f;
      n >>>= 7;
      if (n) b |= 0x80;
      o.push(b);
    } while (n);
    return o;
  }

  function sleb(n) {
    const o = [];
    let m = true;
    while (m) {
      let b = n & 0x7f;
      n >>= 7;
      if ((n === 0 && (b & 0x40) === 0) || (n === -1 && (b & 0x40) !== 0)) m = false;
      else b |= 0x80;
      o.push(b);
    }
    return o;
  }

  function section(id, body) {
    const b = [...body];
    return [id, ...uleb(b.length), ...b];
  }

  function vec(items) {
    return [...uleb(items.length), ...items.flat(Infinity)];
  }

  function funcType(p, r) {
    return [0x60, ...vec(p), ...vec(r)];
  }

  function locals(t, c) {
    return [...uleb(c), t];
  }

  function wasmModule({ types, funcs, mems, exports_, code }) {
    const secs = [
      section(0x01, vec(types)),
      section(
        0x03,
        vec(funcs.map(([ti]) => uleb(ti)))
      ),
      ...(mems ? [section(0x05, vec(mems))] : []),
      section(
        0x07,
        vec(
          exports_.map(([n, k, i]) => [
            ...uleb(n.length),
            ...n.split("").map((c) => c.charCodeAt(0)),
            k,
            ...uleb(i),
          ])
        )
      ),
      section(
        0x0a,
        vec(
          code.map((b) => {
            const [lg, ...ins] = b;
            const lc = [...uleb(lg.length), ...lg.flat(Infinity)];
            const bc = [...lc, ...ins.flat(Infinity)];
            return [...uleb(bc.length), ...bc];
          })
        )
      ),
    ];
    return new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, ...secs.flat()]);
  }

  const O = {
    block: (t = 0x40) => [0x02, t],
    loop: (t = 0x40) => [0x03, t],
    if_: (t = 0x40) => [0x04, t],
    end: [0x0b],
    br: (d) => [0x0c, ...uleb(d)],
    br_if: (d) => [0x0d, ...uleb(d)],
    lg: (i) => [0x20, ...uleb(i)],
    ls: (i) => [0x21, ...uleb(i)],
    k: (v) => [0x41, ...sleb(v)],
    fk: (v) => {
      const b = new Uint8Array(8);
      new DataView(b.buffer).setFloat64(0, v, true);
      return [0x44, ...b];
    },
    i32_ge_s: [0x4e],
    i32_gt_s: [0x4a],
    i32_eqz: [0x45],
    i32_add: [0x6a],
    i32_mul: [0x6c],
    f64_add: [0xa0],
    f64_mul: [0xa2],
    f64_load: (a = 3, o = 0) => [0x2b, ...uleb(a), ...uleb(o)],
    i32_load8_u: (a = 0, o = 0) => [0x2d, ...uleb(a), ...uleb(o)],
    i32_store8: (a = 0, o = 0) => [0x3a, ...uleb(a), ...uleb(o)],
    memory_fill: [0xfc, ...uleb(0x0b), 0x00],
  };

  function buildFibWasm() {
    return wasmModule({
      types: [funcType([I32], [I32])],
      funcs: [[0]],
      exports_: [["fib", 0, 0]],
      code: [
        [
          [locals(I32, 4)],
          O.k(0),
          O.ls(1),
          O.k(1),
          O.ls(2),
          O.k(0),
          O.ls(3),
          O.block(),
          O.loop(),
          O.lg(3),
          O.lg(0),
          O.i32_ge_s,
          O.br_if(1),
          O.lg(1),
          O.lg(2),
          O.i32_add,
          O.ls(4),
          O.lg(2),
          O.ls(1),
          O.lg(4),
          O.ls(2),
          O.lg(3),
          O.k(1),
          O.i32_add,
          O.ls(3),
          O.br(0),
          O.end,
          O.end,
          O.lg(1),
          O.end,
        ],
      ],
    });
  }

  function buildDotWasm() {
    return wasmModule({
      types: [funcType([I32, I32, I32], [F64])],
      funcs: [[0]],
      mems: [[0x00, 0x01]],
      exports_: [
        ["memory", 2, 0],
        ["dot", 0, 0],
      ],
      code: [
        [
          [locals(F64, 1), locals(I32, 1)],
          O.fk(0.0),
          O.ls(3),
          O.k(0),
          O.ls(4),
          O.block(),
          O.loop(),
          O.lg(4),
          O.lg(2),
          O.i32_ge_s,
          O.br_if(1),
          O.lg(3),
          O.lg(0),
          O.lg(4),
          O.k(8),
          O.i32_mul,
          O.i32_add,
          O.f64_load(),
          O.lg(1),
          O.lg(4),
          O.k(8),
          O.i32_mul,
          O.i32_add,
          O.f64_load(),
          O.f64_mul,
          O.f64_add,
          O.ls(3),
          O.lg(4),
          O.k(1),
          O.i32_add,
          O.ls(4),
          O.br(0),
          O.end,
          O.end,
          O.lg(3),
          O.end,
        ],
      ],
    });
  }

  async function tryInst(bytes) {
    try {
      if (!WebAssembly.validate(bytes)) throw new Error("invalid");
      const m = await WebAssembly.instantiate(bytes);
      return m.instance.exports;
    } catch (e) {
      console.error("wasm:", e);
      return null;
    }
  }

  global.SelvrGamesWasm = {
    buildFibWasm,
    buildDotWasm,
    tryInst,
  };
})(typeof window !== "undefined" ? window : globalThis);
