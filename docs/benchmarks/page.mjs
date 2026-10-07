import init, { SELVR_load, SELVR_call } from "./vm/selvr_vm.js";
import * as fibMod from "./selvr/fib.js";
import * as sieveMod from "./selvr/sieve.js";
import * as matmulMod from "./selvr/matmul.js";
import {
  FIB_N,
  SIEVE_LIMIT,
  MATMUL_N,
  fib,
  sieve,
  matmul,
  matrices,
  sum,
  close,
  f64Array,
  measure,
  measurePrepared,
  fmtTime,
  fmtRatio,
} from "./browser-bench.mjs";

const statusEl = document.querySelector("#status");
const runBtn = document.querySelector("#run");
const checksEl = document.querySelector("#checks");

const bc = {};
let vmReady = false;
let vmError = "";

function setStatus(text) {
  statusEl.textContent = text;
}

function paint() {
  return new Promise((resolve) => {
    requestAnimationFrame(() => setTimeout(resolve, 0));
  });
}

function fill(id, values) {
  const row = document.querySelector(id);
  for (const [cls, text] of Object.entries(values)) {
    const cell = row.querySelector("." + cls);
    cell.textContent = text;
    cell.classList.toggle("fail", text === "mismatch");
  }
}

async function loadBytecode() {
  for (const name of ["fib", "sieve", "matmul"]) {
    const res = await fetch(`./selvr/${name}.vlxc`);
    if (!res.ok) throw new Error(`could not fetch selvr/${name}.vlxc (${res.status})`);
    bc[name] = new Uint8Array(await res.arrayBuffer());
  }
}

function vmCall(name, fn, argsJson) {
  SELVR_load(bc[name]);
  return SELVR_call(fn, argsJson);
}

function fmtResult(n) {
  if (!Number.isFinite(n)) return "mismatch";
  if (Number.isInteger(n)) return String(n);
  return n.toFixed(4);
}

function agree(a, b, c) {
  return close(a, b) && close(b, c);
}

const jobs = [
  {
    id: "#row-fib",
    expect: 102334155,
    check() {
      const vm = Number(vmCall("fib", "fib", "[10]"));
      const js = fibMod.fib(10);
      const plain = fib(10);
      return { label: "fib(10) = 55", ok: vm === 55 && js === 55 && plain === 55, vm, js, plain };
    },
    time() {
      const vm = measurePrepared(
        () => SELVR_load(bc.fib),
        () => Number(SELVR_call("fib", `[${FIB_N}]`)),
      );
      const js = measure(() => fibMod.fib(FIB_N));
      const plain = measure(() => fib(FIB_N));
      return { vm, js, plain };
    },
  },
  {
    id: "#row-sieve",
    expect: 1229,
    checks() {
      const out = [];
      for (const [n, expected] of [[10, 4], [100, 25]]) {
        const vm = Number(vmCall("sieve", "sieve", `[${n}]`));
        const js = sieveMod.sieve(n);
        const plain = sieve(n);
        out.push({
          label: `sieve(${n}) = ${expected}`,
          ok: vm === expected && js === expected && plain === expected,
        });
      }
      return out;
    },
    time() {
      const arg = `[${SIEVE_LIMIT}]`;
      const vm = measurePrepared(
        () => SELVR_load(bc.sieve),
        () => Number(SELVR_call("sieve", arg)),
      );
      const js = measure(() => sieveMod.sieve(SIEVE_LIMIT));
      const plain = measure(() => sieve(SIEVE_LIMIT));
      return { vm, js, plain };
    },
  },
  {
    id: "#row-matmul",
    expect: null,
    check() {
      const [a, b] = matrices(2);
      const args = `[${f64Array(a)},${f64Array(b)},2]`;
      const vm = sum(JSON.parse(vmCall("matmul", "matmul", args)));
      const js = sum(matmulMod.matmul(a, b, 2));
      const plain = sum(matmul(a, b, 2));
      const ok = close(vm, 2.75) && close(js, 2.75) && close(plain, 2.75);
      return { label: "matmul 2×2 = 2.75", ok, vm, js, plain };
    },
    time() {
      const [a, b] = matrices(MATMUL_N);
      const args = `[${f64Array(a)},${f64Array(b)},${MATMUL_N}]`;
      const vm = measurePrepared(
        () => SELVR_load(bc.matmul),
        () => sum(JSON.parse(SELVR_call("matmul", args))),
      );
      const js = measure(() => sum(matmulMod.matmul(a, b, MATMUL_N)));
      const plain = measure(() => sum(matmul(a, b, MATMUL_N)));
      return { vm, js, plain };
    },
  },
];

function renderRow(id, timed, expect) {
  const ok = agree(timed.vm.sink, timed.js.sink, timed.plain.sink)
    && (expect == null || close(timed.vm.sink, expect));
  fill(id, {
    result: ok ? fmtResult(timed.vm.sink) : "mismatch",
    vm: fmtTime(timed.vm.ns),
    js: fmtTime(timed.js.ns),
    plain: fmtTime(timed.plain.ns),
    ratio: fmtRatio(timed.vm.ns, timed.plain.ns),
  });
  return ok;
}

async function run() {
  runBtn.disabled = true;
  checksEl.textContent = "";
  for (const id of ["#row-fib", "#row-sieve", "#row-matmul"]) {
    fill(id, { result: "—", vm: "—", js: "—", plain: "—", ratio: "—" });
  }
  const notes = [];
  try {
    setStatus("Loading selvr-vm…");
    await paint();
    if (!vmReady) {
      await init();
      await loadBytecode();
      vmReady = true;
    }
    setStatus("Checking fib(10), sieve, and 2×2 matmul…");
    await paint();
    const checkItems = [
      jobs[0].check(),
      ...jobs[1].checks(),
      jobs[2].check(),
    ];
    for (const item of checkItems) notes.push(item.ok ? item.label : `${item.label} failed`);
    const checksOk = checkItems.every((item) => item.ok);

    let timedOk = true;
    for (const job of jobs) {
      setStatus(`Timing ${job.id.slice(5)}…`);
      await paint();
      const timed = job.time();
      timedOk = renderRow(job.id, timed, job.expect) && timedOk;
    }
    checksEl.innerHTML = notes.map((text) => {
      const ok = !text.endsWith("failed");
      return `<span class="${ok ? "ok" : "fail"}">${text}</span>`;
    }).join(" · ");
    if (checksOk && timedOk) {
      setStatus("Measured in this browser. The three implementations agreed.");
    } else {
      setStatus("A result did not match. The times above are still from this run.");
    }
  } catch (err) {
    vmError = err && err.message ? err.message : String(err);
    setStatus(vmError);
  } finally {
    runBtn.disabled = false;
  }
}

runBtn.addEventListener("click", () => { run(); });
await run();
