// selvr/devtools — Browser devtools integration for Selvr.
//
// When compiled with `selvr build --debug`, the compiler injects a call to
// `__selvr_devtools_init()` at the top of the output bundle. This module
// installs the `window.__selvr` namespace and a DevTools panel that shows
// which Selvr functions are currently running and on which runtime (WASM/JS).
//
// Usage:
//   import { init, trace, endTrace } from "selvr/devtools";
//
// The compiler injects these calls automatically in debug builds; you don't
// need to call them from user code.

/// Initialise the Selvr devtools namespace.
/// Called once at startup in debug builds.
export fn init(): void {
    const manifest = __getTargetManifest();
    console.groupCollapsed(
        "%cSelvr devtools%c loaded — %d functions (%d WASM, %d JS)",
        "color:#00d4aa;font-weight:bold",
        "color:inherit",
        manifest.total,
        manifest.wasm,
        manifest.js
    );
    console.log("  window.__selvr.targetMap — full WASM/JS split report");
    console.log("  window.__selvr.tracing   — live call trace (true/false toggle)");
    console.log("  window.__selvr.calls     — array of recent calls with timing");
    console.groupEnd();
}

/// Record the start of a function call (injected by the compiler in debug mode).
export fn trace(name: string, runtime: string): i32 {
    return __traceStart(name, runtime);
}

/// Record the end of a function call (injected by the compiler in debug mode).
export fn endTrace(handle: i32, name: string): void {
    __traceEnd(handle, name);
}
