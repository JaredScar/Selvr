/**
 * Selvr browser devtools runtime.
 *
 * Injected by `selvr build --debug` at the top of every output bundle.
 * Installs the `window.__selvr` namespace and wires the DevTools panel.
 *
 * All functions are inert in production builds (the compiler strips them).
 */
(function () {
  'use strict';

  if (typeof window === 'undefined') return;  // SSR / Node — skip

  // ── Public namespace ────────────────────────────────────────────────────────

  /** @type {Record<string,{target:'wasm'|'js',score:number,reason:string}>} */
  const targetMap = {};

  /** @type {Array<{name:string,runtime:'wasm'|'js',durationMs:number,startMs:number}>} */
  const calls = [];

  let tracing  = false;
  let handleId = 0;

  /** @type {Map<number,{name:string,runtime:string,startMs:number}>} */
  const pending = new Map();

  window.__selvr = {
    targetMap,
    calls,
    get tracing()       { return tracing; },
    set tracing(v)      { tracing = !!v; console.info(`Selvr tracing ${tracing ? 'ON' : 'OFF'}`); },
    clearCalls()        { calls.length = 0; },
    printReport()       { printReport(); },
    version:            '0.1.0',
  };

  // ── Internal API (called by compiled Selvr code) ────────────────────────────

  window.__getTargetManifest = function () {
    const entries = Object.values(targetMap);
    return {
      total: entries.length,
      wasm:  entries.filter(e => e.target === 'wasm').length,
      js:    entries.filter(e => e.target === 'js').length,
    };
  };

  /**
   * Called at the start of each traced function.
   * @param {string} name     Function name.
   * @param {string} runtime  "wasm" | "js"
   * @returns {number}        Opaque handle for endTrace.
   */
  window.__traceStart = function (name, runtime) {
    if (!tracing) return 0;
    const h = ++handleId;
    pending.set(h, { name, runtime, startMs: performance.now() });
    return h;
  };

  /**
   * Called at the end of each traced function.
   * @param {number} handle  Handle from __traceStart.
   * @param {string} name    Function name (for verification).
   */
  window.__traceEnd = function (handle, name) {
    if (!tracing || handle === 0) return;
    const entry = pending.get(handle);
    if (!entry) return;
    pending.delete(handle);
    const durationMs = performance.now() - entry.startMs;
    calls.push({ name: entry.name, runtime: entry.runtime, durationMs, startMs: entry.startMs });
    if (calls.length > 1000) calls.shift();  // ring buffer
  };

  /**
   * Register a function's targeting decision.
   * Called once per function at module initialisation time.
   * @param {string} name
   * @param {'wasm'|'js'} target
   * @param {number} score
   * @param {string} reason
   */
  window.__selvrRegisterTarget = function (name, target, score, reason) {
    targetMap[name] = { target, score, reason };
  };

  // ── Report printer ──────────────────────────────────────────────────────────

  function printReport() {
    const entries = Object.entries(targetMap);
    if (entries.length === 0) {
      console.warn('Selvr devtools: no target map registered (debug build only).');
      return;
    }
    console.group('%cSelvr target map', 'color:#00d4aa;font-weight:bold');
    const fmt = (t) => t === 'wasm' ? '%c⚙ wasm%c' : '%c⚡ js%c';
    const col = (t) => [t === 'wasm' ? 'color:#a5b4fc' : 'color:#fbbf24', 'color:inherit'];
    for (const [name, { target, score, reason }] of entries) {
      console.log(
        `${fmt(target)} ${name.padEnd(24)} score=${String(score).padStart(4)}  ${reason}`,
        ...col(target),
      );
    }
    console.groupEnd();

    if (calls.length > 0) {
      console.group('%cRecent calls', 'color:#64748b');
      for (const c of calls.slice(-20)) {
        const icon = c.runtime === 'wasm' ? '⚙' : '⚡';
        console.log(`  ${icon} ${c.name.padEnd(24)} ${c.durationMs.toFixed(3)} ms`);
      }
      console.groupEnd();
    }
  }

  // ── DevTools panel (Chrome Extension Message API) ──────────────────────────
  // If the Selvr Chrome extension is installed, it will receive these messages
  // and render a dedicated panel. Otherwise they are silently ignored.

  function postToDevTools(type, payload) {
    try {
      window.postMessage({ source: 'selvr-devtools', type, payload }, '*');
    } catch (_) {}
  }

  // Send the target map once it's populated (after module init).
  setTimeout(() => {
    if (Object.keys(targetMap).length > 0) {
      postToDevTools('TARGET_MAP', targetMap);
    }
  }, 500);

  // Forward live call traces.
  const _origTraceEnd = window.__traceEnd;
  window.__traceEnd = function (handle, name) {
    _origTraceEnd(handle, name);
    if (calls.length % 10 === 0) {  // batch every 10 calls
      postToDevTools('CALLS_BATCH', calls.slice(-10));
    }
  };

})();
