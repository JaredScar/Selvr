// Times one workload two ways:
//   selvr-js  — the module produced by `selvr build` (passed as argv)
//   plain-js  — the hand-written functions in browser-bench.mjs
//
// Usage:
//   node time-js.mjs <compiled.js> <fib|sieve|matmul> <param> <inner> <samples>
//
// Prints one JSON object on stdout. Times are the median nanoseconds per call.
// Matrix inputs are built before the clock starts. The return value is folded
// into a number so the call cannot be deleted.

import { pathToFileURL } from "node:url";
import { resolve } from "node:path";
import { fib, sieve, matmul, matrices, sum, median } from "./browser-bench.mjs";

const [jsFile, kind, paramRaw, innerRaw, samplesRaw] = process.argv.slice(2);
const param = Number(paramRaw);
const inner = Number(innerRaw);
const samples = Number(samplesRaw);

if (!jsFile || !kind || !Number.isFinite(param) || !inner || !samples) {
  console.error("usage: node time-js.mjs <compiled.js> <fib|sieve|matmul> <param> <inner> <samples>");
  process.exit(1);
}

const compiled = await import(pathToFileURL(resolve(jsFile)).href);

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
