// Shared pieces of the fib / sieve / matmul comparison.
//
// plain fib, sieve, and matmul are hand-written copies of selvr/*.self.
// The Selvr side is the JavaScript `selvr build` emitted, or the bytecode
// the VM runs. This file does not time anything by itself.

export const FIB_N = 40;
export const SIEVE_LIMIT = 10000;
export const MATMUL_N = 32;
export const SAMPLES = 5;

export function fib(n) {
  let a = 0;
  let b = 1;
  let i = 0;
  while (i < n) {
    const tmp = a + b;
    a = b;
    b = tmp;
    i = i + 1;
  }
  return a;
}

export function sieve(limit) {
  const composite = [];
  let i = 0;
  while (i <= limit) {
    composite.push(false);
    i = i + 1;
  }
  let count = 0;
  let p = 2;
  while (p <= limit) {
    if (!composite[p]) {
      count = count + 1;
      let multiple = p + p;
      while (multiple <= limit) {
        composite[multiple] = true;
        multiple = multiple + p;
      }
    }
    p = p + 1;
  }
  return count;
}

export function matmul(a, b, n) {
  const c = [];
  let init = 0;
  while (init < n * n) {
    c.push(0.0);
    init = init + 1;
  }
  let row = 0;
  while (row < n) {
    let k = 0;
    while (k < n) {
      const aik = a[row * n + k];
      let col = 0;
      while (col < n) {
        c[row * n + col] = c[row * n + col] + aik * b[k * n + col];
        col = col + 1;
      }
      k = k + 1;
    }
    row = row + 1;
  }
  return c;
}

export function matrices(n) {
  const len = n * n;
  const a = new Array(len);
  const b = new Array(len);
  for (let i = 0; i < len; i++) {
    a[i] = (i % 7) * 0.5;
    b[i] = (i % 5) * 0.25;
  }
  return [a, b];
}

export function sum(xs) {
  let s = 0;
  for (let i = 0; i < xs.length; i++) s += xs[i];
  return s;
}

export function close(actual, expected) {
  const scale = Math.max(1, Math.abs(expected));
  return Math.abs(actual - expected) <= scale * 1e-6;
}

/** JSON number token that the VM parser reads as f64, including whole values. */
export function f64Token(n) {
  const s = String(n);
  if (s.includes(".") || s.includes("e") || s.includes("E")) return s;
  return s + ".0";
}

export function f64Array(xs) {
  let out = "[";
  for (let i = 0; i < xs.length; i++) {
    if (i) out += ",";
    out += f64Token(xs[i]);
  }
  return out + "]";
}

export function median(xs) {
  const sorted = [...xs].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
}

function clockMs() {
  if (typeof performance !== "undefined" && typeof performance.now === "function") {
    return performance.now();
  }
  return Number(process.hrtime.bigint()) / 1e6;
}

/**
 * Median nanoseconds per call.
 * Repeats the call until one batch lasts at least `minMs`, so a coarse
 * browser timer still sees the work. The result is still per call.
 */
function calibrate(runBatch, { minMs, maxIters }) {
  let iters = 1;
  let sink;
  let elapsed = 0;
  while (iters <= maxIters) {
    const t0 = clockMs();
    sink = runBatch(iters);
    elapsed = clockMs() - t0;
    if (elapsed >= minMs || iters === maxIters) break;
    const next = iters * 2;
    if (next > maxIters) break;
    iters = next;
  }
  return { iters, sink };
}

export function measure(fn, opts) {
  return measurePrepared(null, fn, opts);
}

/** `prepare` runs outside the clock (reload the VM, rebuild inputs). */
export function measurePrepared(prepare, fn, { minMs = 8, samples = SAMPLES, maxIters = 2_000_000 } = {}) {
  const runBatch = (iters) => {
    let sink;
    for (let i = 0; i < iters; i++) sink = fn();
    return sink;
  };
  if (prepare) prepare();
  const found = calibrate(runBatch, { minMs, maxIters });
  const batches = [];
  let sink = found.sink;
  for (let s = 0; s < samples; s++) {
    if (prepare) prepare();
    const t0 = clockMs();
    sink = runBatch(found.iters);
    batches.push(((clockMs() - t0) * 1e6) / found.iters);
  }
  return { sink, ns: median(batches), iters: found.iters };
}

export function fmtTime(ns) {
  if (ns >= 100_000) return `${(ns / 1e6).toFixed(3)} ms`;
  return `${(ns / 1e3).toFixed(3)} us`;
}

export function fmtRatio(vmNs, plainNs) {
  if (!(plainNs > 0)) return "n/a";
  return (vmNs / plainNs).toFixed(2);
}
