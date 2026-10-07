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
const NG_COMMON_URL = "https://esm.sh/@angular/common@19.2.15";
const NG_BROWSER_URL = "https://esm.sh/@angular/platform-browser@19.2.15";
const NG_SIZE_URLS = [
  "https://esm.sh/@angular/compiler@19.2.15/es2022/compiler.bundle.mjs",
  "https://esm.sh/@angular/core@19.2.15/es2022/core.bundle.mjs",
  "https://esm.sh/@angular/common@19.2.15/es2022/common.bundle.mjs",
  "https://esm.sh/@angular/platform-browser@19.2.15/es2022/platform-browser.bundle.mjs",
  "https://esm.sh/zone.js@0.15.1/es2022/zone.bundle.mjs",
];

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
    const num = cell.querySelector(".num");
    (num || cell).textContent = text;
    cell.classList.toggle("fail", text === "mismatch" || text === "failed");
  }
  paintBars(row);
}

function paintBars(row) {
  const fills = [...row.querySelectorAll(".fill")];
  if (!fills.length) return;
  const amounts = fills.map((fill) => shownAmount(fill.closest("td").querySelector(".num").textContent));
  const max = Math.max(0, ...amounts);
  fills.forEach((fill, i) => {
    fill.style.width = max > 0 && amounts[i] > 0 ? Math.max(6, (100 * amounts[i]) / max) + "%" : "0%";
  });
}

function shownAmount(text) {
  const match = /([\d.]+)\s*(ms|us|ns|MB|KB|B)/.exec(text || "");
  if (!match) return 0;
  const n = Number(match[1]);
  switch (match[2]) {
    case "ms": return n * 1e6;
    case "us": return n * 1e3;
    case "ns": return n;
    case "MB": return n * 1e6;
    case "KB": return n * 1e3;
    default: return n;
  }
}

function fmtBytes(n) {
  if (n >= 1e6) return (n / 1e6).toFixed(2) + " MB";
  if (n >= 1e3) return (n / 1e3).toFixed(1) + " KB";
  return n + " B";
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
  const ratio = fmtRatio(timed.vm.ns, timed.plain.ns);
  fill(id, {
    result: ok ? fmtResult(timed.vm.sink) : "mismatch",
    vm: fmtTime(timed.vm.ns),
    js: fmtTime(timed.js.ns),
    plain: fmtTime(timed.plain.ns),
    ratio,
  });
  const stat = document.querySelector("#stat-" + id.slice(5));
  if (stat) stat.textContent = ratio + "×";
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
      if (ev.data && ev.data.ok) resolve(ev.data);
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
let reactBytes = 0;
let vueBytes = 0;
let angularBytes = 0;
let selvrBytes = 0;

async function prefetchFrameworks() {
  const load = async (url) => {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`could not fetch ${url} (${res.status})`);
    return res.text();
  };
  const [react, reactDom, vue, ...angularParts] = await Promise.all([
    load(REACT_URL),
    load(REACT_DOM_URL),
    load(VUE_URL),
    ...NG_SIZE_URLS.map(load),
    load(ZONE_URL),
    load(NG_COMPILER_URL),
    load(NG_CORE_URL),
    load(NG_COMMON_URL),
    load(NG_BROWSER_URL),
  ]);
  reactSrc = react;
  reactDomSrc = reactDom;
  vueSrc = vue;
  reactBytes = react.length + reactDom.length;
  vueBytes = vue.length;
  angularBytes = angularParts.slice(0, NG_SIZE_URLS.length).reduce((sum, text) => sum + text.length, 0);
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

const TIMING = `
function __median(xs){const s=xs.slice().sort((a,b)=>a-b);return s[(s.length/2)|0];}
function __reps(run, inner){
  let reps = 1;
  let elapsed = 0;
  while (reps <= 32 && elapsed < 8) {
    const t0 = performance.now();
    for (let r = 0; r < reps; r++) run(inner);
    elapsed = performance.now() - t0;
    if (elapsed >= 8 || reps === 32) break;
    reps *= 2;
  }
  return reps;
}
function __time(fn, samples, inner){
  const run = (n) => { for (let i = 0; i < n; i++) fn(i); };
  const reps = __reps(run, inner);
  const batches = [];
  for (let s = 0; s < samples; s++) {
    const t0 = performance.now();
    for (let r = 0; r < reps; r++) run(inner);
    batches.push((performance.now() - t0) * 1e6 / (reps * inner));
  }
  return __median(batches);
}
async function __timeAsync(fn, samples, inner){
  const run = async (n) => { for (let i = 0; i < n; i++) await fn(i); };
  let reps = 1;
  let elapsed = 0;
  while (reps <= 32 && elapsed < 8) {
    const t0 = performance.now();
    for (let r = 0; r < reps; r++) await run(inner);
    elapsed = performance.now() - t0;
    if (elapsed >= 8 || reps === 32) break;
    reps *= 2;
  }
  const batches = [];
  for (let s = 0; s < samples; s++) {
    const t0 = performance.now();
    for (let r = 0; r < reps; r++) await run(inner);
    batches.push((performance.now() - t0) * 1e6 / (reps * inner));
  }
  return __median(batches);
}`;

function runDomSession(scripts, body, type) {
  return withFrame((doc) => {
    for (const src of scripts) addScript(doc, src);
    addScript(doc, TIMING + body, type);
  });
}

function measureSelvrDom() {
  return runDomSession([], `
    try {
      const host = document.createElement("div");
      document.body.appendChild(host);
      const updateNs = __time(() => { for (let i = 0; i < 1000; i++) host.textContent = String(i); }, 5, 1);
      let listed = 0;
      function paintList() {
        host.textContent = "";
        for (let i = 0; i < 1000; i++) {
          const row = document.createElement("div");
          row.textContent = String(i);
          host.appendChild(row);
        }
        listed = host.childElementCount;
      }
      const listNs = __time(() => paintList(), 5, 1);
      const replaceNs = __time(() => paintList(), 5, 1);
      if (listed !== 1000) throw new Error("dom check failed");
      parent.postMessage({ ok: true, updateNs, listNs, replaceNs }, "*");
    } catch (err) {
      parent.postMessage({ ok: false, error: String(err && err.message || err) }, "*");
    }
  `);
}

function measureReact() {
  return runDomSession([reactSrc, reactDomSrc], `
    try {
      const host = document.createElement("div");
      document.body.appendChild(host);
      let root = ReactDOM.createRoot(host);
      function show(text) {
        ReactDOM.flushSync(() => root.render(React.createElement("span", null, text)));
      }
      function showList() {
        const kids = new Array(1000);
        for (let i = 0; i < 1000; i++) kids[i] = React.createElement("div", { key: i }, String(i));
        ReactDOM.flushSync(() => root.render(React.createElement("div", null, kids)));
      }
      show("0");
      const updateNs = __time(() => { for (let i = 0; i < 1000; i++) show(String(i)); }, 5, 1);
      let listed = 0;
      const listNs = __time(() => { showList(); listed = host.querySelectorAll("div").length; }, 5, 1);
      const replaceNs = __time(() => { root.unmount(); root = ReactDOM.createRoot(host); showList(); listed = host.querySelectorAll("div").length; }, 5, 1);
      if (listed < 1000) throw new Error("react check failed");
      parent.postMessage({ ok: true, updateNs, listNs, replaceNs }, "*");
    } catch (err) {
      parent.postMessage({ ok: false, error: String(err && err.message || err) }, "*");
    }
  `);
}

function measureVue() {
  return runDomSession([vueSrc], `
    (async () => {
      try {
        const host = document.createElement("div");
        document.body.appendChild(host);
        const state = Vue.reactive({ n: "0", rows: [] });
        const options = { render() {
          return state.rows.length
            ? Vue.h("div", state.rows.map((row) => Vue.h("div", String(row))))
            : Vue.h("span", String(state.n));
        } };
        let app = Vue.createApp(options);
        app.mount(host);
        const updateNs = await __timeAsync(async () => {
          for (let i = 0; i < 1000; i++) { state.rows = []; state.n = String(i); await Vue.nextTick(); }
        }, 5, 1);
        let listed = 0;
        async function paintList() {
          state.rows = Array.from({ length: 1000 }, (_, i) => i);
          await Vue.nextTick();
          listed = host.querySelectorAll("div").length;
        }
        const listNs = await __timeAsync(() => paintList(), 5, 1);
        const replaceNs = await __timeAsync(async () => {
          app.unmount();
          app = Vue.createApp(options);
          app.mount(host);
          await paintList();
        }, 5, 1);
        if (listed < 1000) throw new Error("vue check failed " + listed);
        parent.postMessage({ ok: true, updateNs, listNs, replaceNs }, "*");
      } catch (err) {
        parent.postMessage({ ok: false, error: String(err && err.message || err) }, "*");
      }
    })();
  `);
}

function measureAngular() {
  return runDomSession([], `
    try {
      await import(${JSON.stringify(ZONE_URL)});
      await import(${JSON.stringify(NG_COMPILER_URL)});
      const core = await import(${JSON.stringify(NG_CORE_URL)});
      const common = await import(${JSON.stringify(NG_COMMON_URL)});
      const browser = await import(${JSON.stringify(NG_BROWSER_URL)});
      class App { n = "0"; rows = []; }
      const Cmp = core.Component({
        selector: "bench-root",
        standalone: true,
        imports: [common.CommonModule],
        template: '<span>{{n}}</span><div *ngFor="let row of rows">{{row}}</div>',
      })(App);
      document.body.appendChild(document.createElement("bench-root"));
      let appRef = await browser.bootstrapApplication(Cmp);
      const inst = () => appRef.components[0].instance;
      function tick(mut) { mut(inst()); appRef.tick(); }
      const updateNs = __time(() => { for (let i = 0; i < 1000; i++) tick((o) => { o.n = String(i); o.rows = []; }); }, 5, 1);
      let listed = 0;
      const listNs = __time(() => { tick((o) => { o.rows = Array.from({ length: 1000 }, (_, i) => i); }); listed = document.querySelectorAll("bench-root div").length; }, 5, 1);
      const samples = [];
      for (let s = 0; s < 5; s++) {
        const t0 = performance.now();
        appRef.destroy();
        if (!document.querySelector("bench-root")) document.body.appendChild(document.createElement("bench-root"));
        appRef = await browser.bootstrapApplication(Cmp);
        tick((o) => { o.rows = Array.from({ length: 1000 }, (_, i) => i); });
        samples.push((performance.now() - t0) * 1e6);
      }
      const replaceNs = samples.slice().sort((a, b) => a - b)[(samples.length / 2) | 0];
      const host = document.querySelector("bench-root");
      listed = document.querySelectorAll("bench-root div").length;
      if (listed !== 1000 || !host) throw new Error("angular check failed");
      parent.postMessage({ ok: true, updateNs, listNs, replaceNs }, "*");
    } catch (err) {
      parent.postMessage({ ok: false, error: String(err && err.message || err) }, "*");
    }
  `, "module");
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

function cellText(sel) {
  const cell = document.querySelector(sel);
  const num = cell.querySelector(".num");
  return (num || cell).textContent;
}

function failColumn(cls, err) {
  const message = err && err.message ? err.message : String(err);
  for (const row of ["#row-startup", "#row-update", "#row-list", "#row-replace", "#row-size"]) {
    fill(row, { [cls]: "failed" });
    document.querySelector(row + " ." + cls).title = message;
  }
}

async function fillStartup(cls, once) {
  const cell = document.querySelector("#row-startup ." + cls);
  try {
    fill("#row-startup", { [cls]: fmtTime(await timeStartup(once)) });
    cell.classList.remove("fail");
    cell.removeAttribute("title");
  } catch (err) {
    fill("#row-startup", { [cls]: "failed" });
    cell.title = err && err.message ? err.message : String(err);
  }
}

async function run() {
  runBtn.disabled = true;
  for (const id of ["#row-fib", "#row-sieve", "#row-matmul"]) {
    fill(id, { result: "—", vm: "—", js: "—", plain: "—", ratio: "—" });
  }
  for (const id of ["#row-startup", "#row-update", "#row-list", "#row-replace", "#row-size"]) {
    fill(id, { selvr: "—", react: "—", vue: "—", angular: "—" });
  }
  for (const id of ["stat-fib", "stat-sieve", "stat-matmul", "stat-startup"]) {
    document.querySelector("#" + id).textContent = "—";
  }
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

    const columns = [
      ["selvr", "Selvr", startSelvr, measureSelvrDom],
      ["react", "React", startReact, measureReact],
      ["vue", "Vue", startVue, measureVue],
      ["angular", "Angular", startAngular, measureAngular],
    ];
    setStatus("Loading React, Vue, and Angular…");
    await paint();
    let startupOk = true;
    try {
      const [wasm, glue] = await Promise.all([
        fetch(new URL("./vm/selvr_vm_bg.wasm", import.meta.url)),
        fetch(new URL("./vm/selvr_vm.js", import.meta.url)),
      ]);
      selvrBytes = (await wasm.arrayBuffer()).byteLength + (await glue.arrayBuffer()).byteLength;
      fill("#row-size", { selvr: fmtBytes(selvrBytes) });
    } catch (err) {
      fill("#row-size", { selvr: "failed" });
      document.querySelector("#row-size .selvr").title = err && err.message ? err.message : String(err);
    }
    try {
      await prefetchFrameworks();
      fill("#row-size", {
        selvr: fmtBytes(selvrBytes),
        react: fmtBytes(reactBytes),
        vue: fmtBytes(vueBytes),
        angular: fmtBytes(angularBytes),
      });
    } catch (err) {
      startupOk = false;
      for (const cls of ["react", "vue", "angular"]) failColumn(cls, err);
      columns.length = 1;
    }
    for (const [cls, label, startup, session] of columns) {
      setStatus(`Starting ${label}…`);
      await paint();
      await fillStartup(cls, startup);
      if (cellText("#row-startup ." + cls) === "failed") startupOk = false;
      setStatus(`Updating ${label}…`);
      await paint();
      try {
        const data = await session();
        fill("#row-update", { [cls]: fmtTime(data.updateNs) });
        fill("#row-list", { [cls]: fmtTime(data.listNs) });
        fill("#row-replace", { [cls]: fmtTime(data.replaceNs) });
      } catch (err) {
        startupOk = false;
        for (const row of ["#row-update", "#row-list", "#row-replace"]) {
          const cell = document.querySelector(row + " ." + cls);
          const num = cell.querySelector(".num");
          num.textContent = "failed";
          cell.classList.add("fail");
          cell.title = err && err.message ? err.message : String(err);
        }
      }
    }
    const startupStat = document.querySelector("#stat-startup");
    startupStat.textContent = cellText("#row-startup .selvr");

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
