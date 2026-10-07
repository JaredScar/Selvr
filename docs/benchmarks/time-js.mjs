// Times one workload two ways:
//   selvr-js  — the module produced by `selvr build` (passed as argv)
//   plain-js  — the hand-written functions below, the same algorithms as the .self files
//
// Usage:
//   node time-js.mjs <compiled.js> <fib|sieve|matmul> <param> <inner> <samples>
//
// Prints one JSON object on stdout. Times are the median nanoseconds per call.
// Matrix inputs are built before the clock starts. The return value is folded
// into a number so the call cannot be deleted.

import { pathToFileURL } from "node:url";
import { resolve } from "node:path";

const [jsFile, kind, paramRaw, innerRaw, samplesRaw] = process.argv.slice(2);
const param = Number(paramRaw);
const inner = Number(innerRaw);
const samples = Number(samplesRaw);

if (!jsFile || !kind || !Number.isFinite(param) || !inner || !samples) {
  console.error("usage: node time-js.mjs <compiled.js> <fib|sieve|matmul> <param> <inner> <samples>");
  process.exit(1);
}

const compiled = await import(pathToFileURL(resolve(jsFile)).href);

function fib(n) {
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

function sieve(limit) {
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

function matmul(a, b, n) {
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

function matrices(n) {
  const len = n * n;
  const a = new Array(len);
  const b = new Array(len);
  for (let i = 0; i < len; i++) {
    a[i] = (i % 7) * 0.5;
    b[i] = (i % 5) * 0.25;
  }
  return [a, b];
}

function sum(xs) {
  let s = 0;
  for (let i = 0; i < xs.length; i++) s += xs[i];
  return s;
}

function calls() {
  if (kind === "fib") {
    const selvrFn = compiled.fib;
    if (typeof selvrFn !== "function") throw new Error("compiled module has no fib export");
    return {
      selvr: () => selvrFn(param),
      plain: () => fib(param),
    };
  }
  if (kind === "sieve") {
    const selvrFn = compiled.sieve;
    if (typeof selvrFn !== "function") throw new Error("compiled module has no sieve export");
    return {
      selvr: () => selvrFn(param),
      plain: () => sieve(param),
    };
  }
  if (kind === "matmul") {
    const selvrFn = compiled.matmul;
    if (typeof selvrFn !== "function") throw new Error("compiled module has no matmul export");
    const [a, b] = matrices(param);
    return {
      selvr: () => sum(selvrFn(a, b, param)),
      plain: () => sum(matmul(a, b, param)),
    };
  }
  throw new Error(`unknown workload ${kind}`);
}

function median(xs) {
  const sorted = [...xs].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
}

function measure(fn) {
  let sink = 0;
  for (let i = 0; i < inner; i++) sink = fn();
  const batches = [];
  for (let s = 0; s < samples; s++) {
    const t0 = process.hrtime.bigint();
    for (let i = 0; i < inner; i++) sink = fn();
    const t1 = process.hrtime.bigint();
    batches.push(Number(t1 - t0) / inner);
  }
  return { sink, ns: median(batches) };
}

const fns = calls();
const selvr = measure(fns.selvr);
const plain = measure(fns.plain);

process.stdout.write(JSON.stringify({
  selvr: selvr.sink,
  plain: plain.sink,
  selvr_ns: selvr.ns,
  plain_ns: plain.ns,
}) + "\n");
