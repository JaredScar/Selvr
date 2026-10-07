# Benchmarks

`docs/benchmarks/index.html` runs the three programs in the browser when you open it. It also times startup, a text update, a 1,000-row list, a remount, and the loaded code size for Selvr against the published React 18, Vue 3, and Angular 19 builds. That Selvr column loads the JavaScript from `selvr build` and writes the DOM directly. The runtime table is the WebAssembly interpreter. `selvr-js` is the committed output of `selvr build`. `selvr-vm` is this interpreter compiled to WebAssembly, executing the committed `.vlxc` files from `selvr build --emit bc`. `plain-js` is the same algorithm written by hand. `vm/` is produced by `cargo build -p selvr-vm --release --target wasm32-unknown-unknown` and `wasm-bindgen --target web`.

`selvr bench` times the same programs with the native interpreter (no WebAssembly, no JSON argument boundary):

For each one it:

1. Runs this `selvr` binary: `selvr build --emit js` and `selvr build --emit bc`.
2. Executes the `.vlxc` file on the native `selvr-vm` bytecode interpreter.
3. Executes the emitted `.js` file in Node.
4. Executes a hand-written plain-JS copy of the same algorithm in Node (`time-js.mjs`).

The three results have to match (`fib(10) = 55`, `sieve(10) = 4`, `sieve(100) = 25`, `sieve(10000) = 1229`, `matmul` 2×2 sum `2.75`, and the timed runs have to agree with each other). A mismatch fails the command. Times are the median of 5 samples. Input arrays are built before the clock starts.

```bash
cargo run -p selvr-cli --release -- bench
```

or `./docs/benchmarks/run.sh`.

The VM column is the Rust interpreter, not that interpreter hosted inside WebAssembly. Hosting it in WASM would add overhead. On these loops V8 JIT-compiles the JavaScript, so the interpreter is expected to take longer. That is a normal result for a bytecode VM.

`compile.sh` is separate. It times `selvr build` on generated files of N tiny functions, and times `tsc` and `esbuild` only when those tools are installed. A missing `selvr` binary or a failed compile stops the script.

## Withdrawn pages

`startup.html`, `tti.html`, and `hybrid.html` used to show charts for parse time, time-to-interactive, FPS, and a WASM split. They did not run the compiler or the VM. The framework bundles were sized filler, the particle loop was handwritten JavaScript, and the WASM module was assembled in the page. Those pages now say so and point here.
