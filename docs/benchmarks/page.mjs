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
  median,
} from "./browser-bench.mjs";

const statusEl = document.querySelector("#status");
const runBtn = document.querySelector("#run");

const REACT_URL = "https://cdn.jsdelivr.net/npm/react@18.3.1/umd/react.production.min.js";
const REACT_DOM_URL = "https://cdn.jsdelivr.net/npm/react-dom@18.3.1/umd/react-dom.production.min.js";
const VUE_URL = "https://cdn.jsdelivr.net/npm/vue@3.5.22/dist/vue.runtime.global.prod.js";
const ZONE_URL = "https://esm.sh/zone.js@0.15.1";
const NG_COMPILER_URL = "https://esm.sh/@angular/compiler@19.2.15";
const NG_CORE_URL = "https://esm.sh/@angular/core@19.2.15";
const NG_BROWSER_URL = "https://esm.sh/@angular/platform-browser@19.2.15";

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

function withFrame(setup) {
  const iframe = document.createElement("iframe");
  iframe.setAttribute("aria-hidden", "true");
  iframe.style.cssText = "position:absolute;left:-9999px;width:1px;height:1px;border:0";
  document.body.appendChild(iframe);
  const doc = iframe.contentDocument;
  const win = iframe.contentWindow;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error("timed out"));
    }, 20000);
    function cleanup() {
      clearTimeout(timer);
      window.removeEventListener("message", onMsg);
      iframe.remove();
    }
    function onMsg(ev) {
      if (ev.source !== win) return;
      cleanup();
      if (ev.data && ev.data.ok) resolve(ev.data.text);
      else reject(new Error((ev.data && ev.data.error) || "startup failed"));
    }
    window.addEventListener("message", onMsg);
    try {
      setup(doc, win);
    } catch (err) {
      cleanup();
      reject(err);
    }
  });
}

function addScript(doc, code, type) {
  const script = doc.createElement("script");
  if (type) script.type = type;
  script.textContent = code;
  doc.body.appendChild(script);
}

let reactSrc = "";
let reactDomSrc = "";
let vueSrc = "";

async function prefetchFrameworks() {
  const load = async (url) => {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`could not fetch ${url} (${res.status})`);
    return res.text();
  };
  [reactSrc, reactDomSrc, vueSrc] = await Promise.all([
    load(REACT_URL),
    load(REACT_DOM_URL),
    load(VUE_URL),
    load(ZONE_URL),
    load(NG_COMPILER_URL),
    load(NG_CORE_URL),
    load(NG_BROWSER_URL),
  ]).then((parts) => parts);
}

function startSelvr() {
  const vmUrl = new URL("./vm/selvr_vm.js", import.meta.url).href;
  const bcUrl = new URL("./selvr/fib.vlxc", import.meta.url).href;
  return withFrame((doc) => {
    addScript(doc, `
      import init, { SELVR_load, SELVR_call } from ${JSON.stringify(vmUrl)};
      try {
        await init();
        const res = await fetch(${JSON.stringify(bcUrl)});
        const bytes = new Uint8Array(await res.arrayBuffer());
        SELVR_load(bytes);
        const n = SELVR_call("fib", "[10]");
        parent.postMessage({ ok: n === "55", text: n, error: "fib(10)=" + n }, "*");
      } catch (err) {
        parent.postMessage({ ok: false, error: String(err && err.message || err) }, "*");
      }
    `, "module");
  });
}

function startReact() {
  return withFrame((doc) => {
    addScript(doc, reactSrc);
    addScript(doc, reactDomSrc);
    addScript(doc, `
      try {
        const el = document.createElement("div");
        document.body.appendChild(el);
        const root = ReactDOM.createRoot(el);
        ReactDOM.flushSync(() => {
          root.render(React.createElement("span", null, "55"));
        });
        parent.postMessage({ ok: el.textContent === "55", text: el.textContent, error: el.textContent }, "*");
      } catch (err) {
        parent.postMessage({ ok: false, error: String(err && err.message || err) }, "*");
      }
    `);
  });
}

function startVue() {
  return withFrame((doc) => {
    addScript(doc, vueSrc);
    addScript(doc, `
      try {
        const el = document.createElement("div");
        document.body.appendChild(el);
        Vue.createApp({ render: () => Vue.h("span", "55") }).mount(el);
        parent.postMessage({ ok: el.textContent === "55", text: el.textContent, error: el.textContent }, "*");
      } catch (err) {
        parent.postMessage({ ok: false, error: String(err && err.message || err) }, "*");
      }
    `);
  });
}

function startAngular() {
  return withFrame((doc) => {
    addScript(doc, `
      try {
        await import(${JSON.stringify(ZONE_URL)});
        await import(${JSON.stringify(NG_COMPILER_URL)});
        const core = await import(${JSON.stringify(NG_CORE_URL)});
        const browser = await import(${JSON.stringify(NG_BROWSER_URL)});
        const App = core.Component({
          selector: "bench-root",
          standalone: true,
          template: "<span>55</span>",
        })(class {});
        document.body.appendChild(document.createElement("bench-root"));
        await browser.bootstrapApplication(App);
        const host = document.querySelector("bench-root");
        const text = host ? host.textContent : "";
        parent.postMessage({ ok: text === "55", text, error: text || "no host" }, "*");
      } catch (err) {
        parent.postMessage({ ok: false, error: String(err && err.message || err) }, "*");
      }
    `, "module");
  });
}

async function timeStartup(once) {
  await once();
  const times = [];
  for (let i = 0; i < 5; i++) {
    const t0 = performance.now();
    await once();
    times.push((performance.now() - t0) * 1e6);
  }
  return median(times);
}

async function fillStartup(cls, once) {
  const cell = document.querySelector("#row-startup ." + cls);
  try {
    cell.textContent = fmtTime(await timeStartup(once));
    cell.classList.remove("fail");
    cell.removeAttribute("title");
  } catch (err) {
    cell.textContent = "failed";
    cell.classList.add("fail");
    cell.title = err && err.message ? err.message : String(err);
  }
}

async function run() {
  runBtn.disabled = true;
  for (const id of ["#row-fib", "#row-sieve", "#row-matmul"]) {
    fill(id, { result: "—", vm: "—", js: "—", plain: "—", ratio: "—" });
  }
  fill("#row-startup", { vm: "—", react: "—", vue: "—", angular: "—" });
  try {
    setStatus("Loading selvr-vm…");
    await paint();
    if (!vmReady) {
      await init();
      await loadBytecode();
      vmReady = true;
    }
    const checkItems = [
      jobs[0].check(),
      ...jobs[1].checks(),
      jobs[2].check(),
    ];
    const checksOk = checkItems.every((item) => item.ok);

    let timedOk = true;
    for (const job of jobs) {
      setStatus(`Timing ${job.id.slice(5)}…`);
      await paint();
      const timed = job.time();
      timedOk = renderRow(job.id, timed, job.expect) && timedOk;
    }

    const startups = [
      ["vm", "Selvr", startSelvr],
      ["react", "React", startReact],
      ["vue", "Vue", startVue],
      ["angular", "Angular", startAngular],
    ];
    setStatus("Loading React, Vue, and Angular…");
    await paint();
    let startupOk = true;
    try {
      await prefetchFrameworks();
    } catch (err) {
      startupOk = false;
      for (const cls of ["react", "vue", "angular"]) {
        const cell = document.querySelector("#row-startup ." + cls);
        cell.textContent = "failed";
        cell.classList.add("fail");
        cell.title = err && err.message ? err.message : String(err);
      }
      startups.length = 1;
    }
    for (const [cls, label, once] of startups) {
      setStatus(`Starting ${label}…`);
      await paint();
      await fillStartup(cls, once);
      if (document.querySelector("#row-startup ." + cls).textContent === "failed") startupOk = false;
    }

    if (checksOk && timedOk && startupOk) setStatus("Measured in this browser.");
    else if (!checksOk || !timedOk) setStatus("A result did not match.");
    else setStatus("A startup did not finish.");
  } catch (err) {
    vmError = err && err.message ? err.message : String(err);
    setStatus(vmError);
  } finally {
    runBtn.disabled = false;
  }
}

runBtn.addEventListener("click", () => { run(); });
await run();
