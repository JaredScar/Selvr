# Selvr — Build Plan

A compiled, browser-native language that **automatically decides** whether each function runs as WebAssembly or JavaScript — giving you WASM performance where it matters and native JS ergonomics where it doesn't.

**Tagline:** One language. Two runtimes. Zero compromise.

---

## The core idea

Every existing option forces a choice:

- **JavaScript** — fast for UI & DOM, slow for math and heavy computation.
- **WebAssembly (raw)** — fast for computation, awful for DOM interaction.
- **Rust → WASM** — great throughput, no DOM ergonomics, steep learning curve.

**Selvr eliminates the choice entirely.**

The developer writes a single `.self` source file. At compile time, Selvr's **targeting pass** analyses each function and automatically assigns it to the optimal runtime:

| Function characteristic | Assigned runtime | Why |
|------------------------|-----------------|-----|
| Float-heavy loops, matrix math, image processing | **WASM** | Hardware-native throughput, no GC pauses |
| 3D / canvas rendering, audio DSP | **WASM** | Predictable frame budget |
| DOM queries, event listeners, form logic | **JS** | V8-native; no JS↔WASM bridge overhead |
| `fetch`, timers, lightweight string ops | **JS** | Already optimal in V8 |
| Mixed (calls both DOM and heavy math) | **JS shell + WASM worker** | Auto-split at the boundary |

The compiler emits **two artefacts** and a thin **bridge layer** that wires them together. From the developer's perspective, calling a WASM function looks identical to calling a JS function — the bridge is invisible.

```selvr
// Selvr source — one file, one language
fn blur(pixels: f64[], radius: i32): f64[] { … }   // → WASM (float loop)
fn onClick(e: Event): void { document.querySelector("#out").text = "done"; } // → JS
```

```
selvr build app.self
  → app.wasm   (blur, and all math-heavy functions)
  → app.js     (onClick, DOM logic, glue bridge)
  → loader.js  (boots WASM, wires the two halves)
```

---

## Why Selvr?

### vs. JavaScript
| Problem with JS | Selvr's answer |
|---|---|
| GC pauses cause unpredictable frame drops | Ownership model — memory freed deterministically, zero GC pauses |
| Parsed and JIT-compiled at runtime (slow startup) | Ships pre-compiled bytecode — browser skips parse-and-optimise entirely |
| `typeof null === "object"` — decades of unfixable quirks | Clean-slate semantics with no legacy baggage |
| No native pattern matching | First-class `match` blocks replace tangled `if/else` chains |
| Metaprogramming happens at runtime (`eval`, `Proxy`) | Compile-time macros — zero runtime cost |
| Heavy computation is slow unless you reach for WASM manually | **Compiler auto-routes hot functions to WASM automatically** |

### vs. TypeScript
TypeScript is JavaScript with types bolted on. It has intentional unsoundness (`any`, type assertions, structural holes) because it must stay compatible with JS. Selvr's type system is **sound from the ground up** — if it compiles, it is type-safe, with no escape hatches. You also get real compiled performance and the hybrid WASM/JS split, not types that disappear at runtime.

### vs. Rust (in the browser)
Rust via WASM is the closest technical comparison. Selvr's advantages:
- **Hybrid by default** — JS-facing code stays in JS; only compute-heavy code goes to WASM
- Purpose-built for the web — DOM, events, and fetch are first-class
- Friendlier syntax designed for web developers, not systems programmers
- Faster compile times via an incremental AOT compiler tuned for the browser dev loop

### vs. raw WebAssembly
Writing WASM by hand or compiling C/C++ to it gives you performance but zero DOM ergonomics. You spend significant effort marshalling data across the JS↔WASM boundary. Selvr handles all of that boundary crossing automatically.

### vs. existing hybrid approaches (Worklets, SharedArrayBuffer tricks)
These are manual, fragile, and require the developer to reason about two different mental models simultaneously. Selvr makes the split a **compiler responsibility, not a developer responsibility**.

### The core bet
Every existing option makes you choose between **fast** and **pleasant to write**, or between **WASM performance** and **JS ergonomics**. Selvr's thesis: a language designed from scratch for the browser, with an auto-targeting compiler, can give you both at the same time.

---

## Differentiation tasks
These are the tasks that prove the competitive claims above. Treat them as the most important items in the plan.

- [x] Produce a GC-pause benchmark: Selvr animation loop vs. JS animation loop under memory pressure (`examples/24_benchmark_animation.self`)
- [x] Produce a startup benchmark: Selvr bytecode cold start vs. equivalent JS bundle parse time (`docs/benchmarks/startup.html`)
- [x] Produce a type soundness test suite demonstrating cases TypeScript allows but Selvr rejects (`examples/23_type_system_soundness.self`)
- [x] Write a side-by-side code comparison page (JS vs. Selvr for 5 representative tasks) (`docs/comparison.html`)
- [x] Write a "Why Selvr?" page on the official site backed by the benchmark data above (`docs/why-selvr.html`)
- [x] Produce a hybrid-split benchmark: same algorithm run fully in JS vs. auto-split WASM+JS (`docs/benchmarks/hybrid.html`)

---

## Phase 1 — Language design & parser

Goal: lock in the language spec and produce a working transpiler to JavaScript so you can run Selvr code in any browser today.

### 1.1 Language specification
- [x] Write a one-page vision document (goals, non-goals, target audience) — `SPEC.md` §1 & §22
- [x] Define core syntax (expressions, statements, functions, blocks) — `SPEC.md` §4–6
- [x] Define the type system (primitives, structs, enums, generics) — `SPEC.md` §3, §8–11
- [x] Decide on ownership/borrowing rules for memory safety — `SPEC.md` §18
- [x] Specify pattern matching and destructuring syntax — `SPEC.md` §7
- [x] Specify async/await and concurrency primitives — `SPEC.md` §15
- [x] Specify the module system and import/export syntax — `SPEC.md` §16
- [x] Write 20–30 example programs that exercise the full language — `examples/` (25 programs)
- [x] Publish the spec as a living document (`SPEC.md` in the repo) — `SPEC.md`

### 1.2 Lexer & parser
- [x] Choose implementation host language (Rust) — Rust workspace
- [x] Set up the project repo, CI, and test harness — Cargo workspace with crates
- [x] Implement the lexer (tokeniser) with full error reporting — `crates/selvr-lexer/`
- [x] Implement the parser producing a clean AST — `crates/selvr-parser/`
- [x] Implement source maps and span tracking for error messages — `Span`, `Spanned<T>`, `sourcemap.rs`
- [x] Write parser snapshot tests for every example program — `crates/selvr-parser/tests/snapshots.rs` (20 cases)
- [x] Build a pretty-printer (AST → source) for debugging — `selvr dump` via `{:#?}` AST output

### 1.3 Transpiler to JavaScript
- [x] Implement AST → JavaScript code generator — `crates/selvr-codegen/src/js.rs`
- [x] Handle runtime polyfills for Selvr-specific features — `SELVR_RUNTIME_PREAMBLE`
- [x] Wire up a CLI: `selvr build file.self → file.js` — `crates/selvr-cli/`
- [x] Produce source maps so debuggers point at `.self` files — `sourcemap.rs`
- [x] Ship a browser playground so people can try it online (`docs/playground/index.html`)

---

## Phase 2 — Native compiler & browser VM

Goal: bypass JavaScript entirely for compute-heavy code. Selvr compiles to a compact bytecode that runs in a WebAssembly-hosted VM inside the browser.

### 2.1 Type checker
- [x] Implement name resolution and scope analysis — `crates/selvr-typechecker/src/resolver.rs`
- [x] Implement the inference engine (Hindley-Milner style) — `crates/selvr-typechecker/src/infer.rs`
- [x] Implement borrow checker and ownership rules — `crates/selvr-typechecker/src/borrow.rs`
- [x] Produce human-readable type errors with suggestions — `crates/selvr-typechecker/src/error.rs`
- [x] Build a test suite of "should fail" programs with expected errors — `tests/should_fail/` (13 cases)

### 2.2 Bytecode & compiler backend
- [x] Design the bytecode format (instruction set, constant pool, layout) — `crates/selvr-bytecode/src/opcode.rs`
- [x] Document the bytecode spec as a separate file — `docs/BYTECODE.md`
- [x] Implement AST → IR lowering pass — `crates/selvr-ir/src/lower.rs`
- [x] Implement IR → bytecode emitter — `crates/selvr-bytecode/src/emit.rs`
- [x] Add optimisation passes (constant folding, dead code elimination, inlining) — `crates/selvr-bytecode/src/opt.rs`
- [x] Support incremental compilation (only rebuild what changed) — `crates/selvr-bytecode/src/incr.rs`
- [x] Benchmark compile times against TypeScript and esbuild — `docs/benchmarks/compile.sh`

### 2.3 Browser runtime (VM)
- [x] Build the VM core in Rust, compiled to WebAssembly — `crates/selvr-vm/`
- [x] Implement the interpreter loop for bytecode execution — `crates/selvr-vm/src/vm.rs`
- [ ] Add a simple baseline JIT for hot paths (stretch goal — Phase 3)
- [x] Implement the memory manager (arena + ownership semantics) — `crates/selvr-vm/src/mem.rs`
- [x] Implement DOM bindings (so Selvr can manipulate the page) — `crates/selvr-vm/src/dom.rs`
- [x] Implement fetch, timers, and event loop integration — `crates/selvr-vm/src/runtime.rs`
- [x] Publish the runtime as a single `.wasm` file included with a `<script>` tag — `runtime/selvr-loader.js`
- [x] Benchmark against V8 (Chrome) on representative workloads — `docs/benchmarks/tti.html`, `docs/benchmarks/hybrid.html`

### 2.4 End-to-end integration
- [x] Write a sample Selvr TodoMVC app — `examples/19_todo_app.self`
- [x] Write a sample Selvr game or interactive demo — `examples/20_canvas_game.self`
- [x] Measure cold start, time-to-interactive, and runtime FPS vs. JS — `docs/benchmarks/tti.html` (live measurements: parse time, TTI, 5000-particle FPS)

---

## Phase 2.5 — Hybrid WASM / JS Targeting ★ new

Goal: make the compiler the one responsible for deciding *where* each function runs. The developer writes one language; the compiler emits optimally-split WASM + JS + a bridge that wires them together invisibly.

### 2.5.1 Targeting analysis pass
- [x] Design the `Target` IR annotation — `selvr_ir::Target` enum in `crates/selvr-ir/src/ir.rs`; `target: Target` field on `IrFn`
  - `Target::Wasm` — function routed to bytecode VM / compiled WASM
  - `Target::Js`   — function transpiled to native JavaScript
  - `Target::Auto` — default before the pass runs
- [x] Implement automatic target inference — `crates/selvr-target/src/infer.rs`
  - **WASM heuristics**: numeric loop density, `Math.*` call count, FMA pattern, `f64[]`/`i32[]` return type, `#[wasm]` attribute
  - **JS heuristics**: DOM API calls (`document.*`, `window.*`, `addEventListener`), async event-handler naming, `#[js]` attribute
  - **Tie-breaking rule**: default to JS (score < 50 → JS, score ≥ 50 → WASM)
- [x] Propagate targets through call graphs — `crates/selvr-target/src/propagate.rs` (fixed-point upgrade/downgrade rules)
- [x] Emit target annotations into IR nodes — `IrFn.target` field set by `infer_targets()` and `propagate_targets()`

### 2.5.2 Bridge code generation
- [x] Design the bridge interface — `crates/selvr-bridge/src/lib.rs`
  - JS wrapper emitted for every WASM-targeted export: serialises args, calls `selvr_vm.selvr_call()`, deserialises result
  - WASM import stubs generated for JS-targeted functions called from WASM
- [x] Generate the bridge glue module — `crates/selvr-bridge/src/codegen.rs`
- [x] Add `selvr build --emit hybrid` flag to the CLI — `crates/selvr-cli/src/main.rs` (`EmitMode::Hybrid`)
- [x] Auto-detect zero-copy paths — `crates/selvr-bridge/src/zerocopy.rs` (`TransferMode::ZeroCopy` for `f64[]`/`i32[]` params)

### 2.5.3 Split-compilation output
- [x] Emit `app.bridge.js` — JS wrappers for WASM exports + bridge stubs (`BridgeEmitter::emit_js`)
- [x] Emit `app.loader.js` — bootstraps WASM, wires both halves, exposes `Selvr.call()` (`BridgeEmitter::emit_loader`)
- [x] Emit `app.split-report.json` — JSON targeting decision for every function (`BridgeEmitter::emit_report`)
- [x] CLI flag `selvr explain app.self` — prints human-readable targeting report to stdout (`cmd_explain`)

### 2.5.4 Developer overrides
- [x] Support `#[wasm]` attribute — parsed by `selvr-parser`, propagated through `IrFn.attrs`, respected by `infer_targets` (+1000 score → forced Wasm)
- [x] Support `#[js]` attribute — same path (−1000 score → forced JS)
- [x] Support `#[inline_bridge]` — parsed and stored in `IrFn.attrs` (bridge codegen reads it; semantics: suppress zero-copy upgrade)
- [x] Warn (not error) when forced target contradicts analysis — `emit_target_warnings()` in CLI prints `warning:` to stderr

### 2.5.5 Benchmarks & validation
- [x] `docs/benchmarks/hybrid.html` — side-by-side: fully JS vs. WASM (simulated) vs. auto-split Selvr across 4 workloads (matmul, fib, DOM rendering, bridge overhead)
- [x] Measure bridge call overhead — JSON path and zero-copy `Float64Array` measured live in the browser; goal < 50 µs zero-copy shown in benchmark
- [x] Show GPU-compute workloads (via WebGPU) as a future WASM extension target — live support detection + WGSL codegen preview added to `docs/benchmarks/hybrid.html`

---

## Phase 3 — Ecosystem & adoption

Goal: make Selvr usable for real projects. Great tooling is what makes or breaks a new language.

### 3.1 Standard library
- [x] Core data structures (Array, Map, Set, Option, Result) — `std/core/option.self`, `std/core/result.self`, `std/collections/array.self`
- [x] String and text handling (including Unicode) — `std/collections/string.self` (full Unicode via V8 delegation)
- [x] Iterators and functional helpers — `std/core/iter.self` (sum, dot, fill, range, reverse, sort, dedup, zip, moving_avg)
- [x] DOM and Web API wrappers (idiomatic Selvr APIs, auto-targeting aware) — `std/dom/dom.self`
- [x] Networking (fetch wrapper, WebSocket) — `std/net/fetch.self`, `std/net/websocket.self`
- [x] Date/time handling — `std/time/datetime.self`
- [x] Testing framework built in — `std/test/test.self`; `selvr test` CLI command; `#[test]` attribute runner

### 3.2 Package manager
- [x] Design the manifest format (similar to `Cargo.toml`) — `selvr.toml.example`; parsed by `crates/selvr-pkg/src/manifest.rs`
- [x] Build the CLI: `selvr add`, `selvr install`, `selvr publish` — `crates/selvr-cli/src/main.rs`; also `selvr remove`, `selvr search`, `selvr test`, `selvr fmt`, `selvr init`
- [x] Set up a package registry (can start as a GitHub-backed index) — `docs/REGISTRY.md`; registry client in `crates/selvr-pkg/src/registry.rs`; HTTP upload wired in Phase 3
- [x] Support semantic versioning and lockfiles — `semver` crate integration; `selvr.lock` JSON format; `crates/selvr-pkg/src/lockfile.rs` + `resolver.rs`
- [x] Support workspaces/monorepos — `[workspace]` manifest section documented in `docs/REGISTRY.md`

### 3.3 Developer tools
- [x] Language Server Protocol (LSP) implementation (`crates/selvr-lsp` — completions, hover, diagnostics, go-to-definition, document symbols, formatting)
- [x] VS Code extension (syntax highlighting, completions, errors, target annotations — `editors/vscode/`)
- [x] Formatter (`selvr fmt` — real AST pretty-printer via `crates/selvr-fmt`)
- [x] Linter (`selvr lint` — SL001–SL007 rules via `crates/selvr-lint`)
- [x] Debugger protocol support (DAP) — unified WASM/JS debugging (`crates/selvr-dap`, `selvr dap`)
- [x] Browser devtools integration — `window.__selvr` namespace + `devtools.runtime.js` injected in debug builds (`std/devtools/`)

### 3.4 Documentation & community
- [ ] Build the official website (`selvr-lang.org` or similar)
- [ ] Write "The Selvr Book" — a free online tutorial
- [x] Write the API reference — `docs/api/index.html` (GitHub Pages under `/docs`)
- [ ] Set up a Discord or forum for early adopters
- [ ] Create a contribution guide and code of conduct
- [ ] Run a blog with design notes and progress updates

### 3.5 Launch
- [ ] 1.0 release candidate
- [ ] Security audit of the compiler and VM
- [ ] Show HN / announcement post
- [ ] Talks at a JS or web conference
- [ ] 1.0 stable release 🎉

---

## Stretch goals

- [ ] Server-side runtime (Node.js / Deno-style binary)
- [ ] Native mobile target (iOS / Android bindings)
- [ ] Compile to WebGPU shaders for GPU workloads (natural extension of WASM targeting)
- [ ] Hot module reloading during development
- [ ] Formal verification of the type system
- [ ] Auto-targeting extended to Web Workers (offload WASM to a background thread automatically)

---

## Design principles (reference throughout)

Keep coming back to these whenever a decision gets hard:

1. **Performance is non-negotiable.** If a feature makes hot paths slower, it doesn't ship.
2. **Expressiveness over brevity.** Terse is good, but clarity always wins.
3. **Zero-cost abstractions.** High-level features must compile to code as fast as hand-written.
4. **Great errors.** A confusing error message is a bug. Targeting decisions must be explainable.
5. **Boring where it matters.** Innovate on performance and syntax; be conservative on package management, versioning, and tooling conventions.
6. **No escape hatches.** There is no `any`, no `unsafe` block for type rules, no way to opt out of soundness.
7. **The web is the target.** DOM, events, fetch, and the browser event loop are first-class.
8. **The compiler is smarter than you about *where* things run.** Targeting decisions belong to the compiler, not the developer. Overrides exist but are the exception, not the rule.
