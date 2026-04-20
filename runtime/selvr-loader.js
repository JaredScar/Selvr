/**
 * Selvr Runtime Loader  v0.1
 *
 * Drop-in <script> tag that boots the Selvr VM in the browser:
 *
 *   <script src="SELVR-loader.js" data-src="app.vlxc" data-main="main"></script>
 *
 * Attributes:
 *   data-src   — URL of the compiled .vlxc bytecode file (required).
 *   data-main  — Exported Selvr function to call on load (default: "main").
 *   data-args  — JSON array of arguments to pass to the entry point (default: []).
 *
 * Global API (available as `window.Selvr` after load):
 *   Selvr.call(name, ...args)  → Promise<any>
 *   Selvr.output               → string[]   (captured console.log lines)
 *   Selvr.vm                   → raw wasm-bindgen instance
 */
(async function SelvrLoader() {
  "use strict";

  const script = document.currentScript;
  const bytecodeUrl = script?.dataset?.src;
  const entryFn     = script?.dataset?.main ?? "main";
  const entryArgs   = JSON.parse(script?.dataset?.args ?? "[]");

  // ── 1. Locate the WASM module ──────────────────────────────────────────────
  //
  // Convention: the .wasm sits next to SELVR-loader.js as "selvr_vm_bg.wasm".
  // wasm-pack generates `selvr_vm.js` (the JS glue) and `selvr_vm_bg.wasm`.
  const loaderBase = (() => {
    const src = script?.src ?? "";
    return src.substring(0, src.lastIndexOf("/") + 1);
  })();

  // ── 2. Import the wasm-bindgen glue ────────────────────────────────────────
  let vm;
  try {
    vm = await import(`${loaderBase}selvr_vm.js`);
    await vm.default(`${loaderBase}selvr_vm_bg.wasm`);
  } catch (e) {
    console.warn("[Selvr] WASM module not yet built. Run `wasm-pack build` first.", e);
    // Expose a stub so pages don't hard-crash.
    window.Selvr = { call: async () => null, output: [], vm: null, loaded: false };
    return;
  }

  // ── 3. Load the bytecode ────────────────────────────────────────────────────
  if (bytecodeUrl) {
    const resp  = await fetch(bytecodeUrl);
    const bytes = new Uint8Array(await resp.arrayBuffer());
    try {
      vm.SELVR_load(bytes);
    } catch (e) {
      console.error("[Selvr] Failed to load bytecode:", e);
    }
  }

  // ── 4. Expose the public API ────────────────────────────────────────────────
  const Selvr = {
    loaded: true,
    vm,
    output: [],

    /**
     * Call an exported Selvr function.
     * @param {string} name   Selvr function name.
     * @param {...any} args   Arguments (primitives and plain objects).
     * @returns {Promise<any>}
     */
    async call(name, ...args) {
      const argsJson = JSON.stringify(args);
      let result;
      try {
        result = vm.SELVR_call(name, argsJson);
      } catch (e) {
        throw new Error(`[Selvr] Runtime error in '${name}': ${e}`);
      }
      // Drain any console.log output produced during the call.
      const out = vm.SELVR_drain_output();
      if (out) {
        for (const line of out.split("\n")) {
          if (line) {
            this.output.push(line);
            console.log("[Selvr]", line);
          }
        }
      }
      return parseSelvrResult(result);
    },

    /** Return the runtime version string. */
    version() { return vm.SELVR_version(); },

    /**
     * Resume a suspended async coroutine (called by JS promise handlers).
     * @param {number} id  Coroutine ID.
     */
    resume(id) { vm.SELVR_resume(id); },
  };

  window.Selvr = Selvr;

  // ── 5. Call the entry point ─────────────────────────────────────────────────
  if (entryFn) {
    try {
      await Selvr.call(entryFn, ...entryArgs);
    } catch (e) {
      console.error("[Selvr] Entry point error:", e);
    }
  }

  // ── 6. Dispatch a ready event ───────────────────────────────────────────────
  document.dispatchEvent(new CustomEvent("SELVR:ready", { detail: Selvr }));

  // ── Helpers ─────────────────────────────────────────────────────────────────

  function parseSelvrResult(json) {
    if (json === null || json === undefined) return null;
    try { return JSON.parse(json); } catch { return json; }
  }
})();
