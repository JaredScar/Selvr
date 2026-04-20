# Selvr

> **Compiled speed. Expressive syntax. Browser-native.**

Selvr is a statically-typed, compiled language designed from the ground up to replace JavaScript in the browser. It ships pre-compiled bytecode — the browser skips parsing and JIT warm-up entirely. It has a sound type system, deterministic memory management (no GC), and first-class DOM, fetch, and event APIs.

```SELVR
async fn main() {
    let button = dom::query("#greet").unwrap();
    button.on("click", |_| {
        let name = dom::query("#name").unwrap().value();
        dom::query("#output").unwrap().set_text(f"Hello, {name}!");
    });
}
```

---

## Why Selvr?

| Pain point | Selvr's answer |
|---|---|
| GC pauses drop animation frames | Ownership model — memory freed deterministically, zero GC |
| JS parsed and JIT-compiled at runtime | Ships pre-compiled bytecode — cold start is microseconds |
| `typeof null === "object"` | Clean-slate semantics, no legacy baggage |
| No native pattern matching | First-class `match` blocks |
| Metaprogramming at runtime | Compile-time macros, zero runtime cost |
| TypeScript types disappear at runtime | Sound types enforced end-to-end through the VM |

---

## Quick start

### Prerequisites

- [Rust](https://rustup.rs/) ≥ 1.78
- [Node.js](https://nodejs.org/) ≥ 20 (for Phase 1 JS backend)

### Build the toolchain

```bash
cargo build --release
```

The `selvr` binary is placed at `target/release/selvr` (on Windows, `target\release\selvr.exe`).

### Compile a Selvr file

```bash
selvr build hello.self          # produces hello.js + hello.js.map
selvr run hello.self            # compile + run with Node.js
selvr check hello.self          # type-check only
selvr dump hello.self           # print the AST
```

### Hello, world

Create `hello.self`:

```SELVR
fn main() {
    print("Hello, world!");
}
```

```bash
selvr run hello.self
# Hello, world!
```

---

## Repository layout

```
SELVR/
├── SPEC.md                 ← Language specification (start here)
├── Cargo.toml              ← Workspace manifest
│
├── crates/
│   ├── selvr-lexer/        ← Tokenizer
│   ├── selvr-parser/       ← AST parser
│   ├── selvr-typechecker/  ← Type inference + borrow checker
│   ├── selvr-codegen/      ← JS emitter (Phase 1) / bytecode emitter (Phase 2)
│   └── selvr-cli/          ← `selvr` CLI
│
├── runtime/                ← WebAssembly VM (Phase 2, Rust → WASM)
├── stdlib/                 ← Standard library source
├── examples/               ← Example Selvr programs
└── website/                ← Official website source
```

---

## Roadmap

### Phase 1 — Language design & transpiler

- [x] Language specification (`SPEC.md`)
- [x] Lexer, parser, and AST
- [x] Type inference and borrow checker (`selvr-typechecker`)
- [x] JS code generator (`selvr-codegen`)
- [x] CLI (`selvr build`, `selvr run`, `selvr check`, `selvr test`, `selvr fmt`, …)
- [x] Standard library (`std/`)
- [x] Source maps (`.self` ↔ generated JS)
- [x] Browser playground (`docs/playground/`)

### Phase 2 — Native bytecode VM

- [x] Bytecode format (`docs/BYTECODE.md`, `selvr-bytecode`)
- [x] AST → IR → bytecode compiler (`selvr-ir`, `selvr-bytecode`)
- [x] WebAssembly-hosted VM (`selvr-vm`)
- [x] DOM bindings and browser runtime
- [x] Benchmarks vs. V8 / JS (`docs/benchmarks/`)
- [ ] Baseline JIT for hot VM paths (stretch)

### Phase 2.5 — Hybrid WASM / JS targeting

- [x] Automatic per-function JS vs WASM routing, bridge codegen, and `selvr build --emit hybrid` (`selvr-target`, `selvr-bridge`)

### Phase 3 — Ecosystem

- [x] Package manager (`selvr add`, `selvr publish`, lockfiles; `selvr-pkg`)
- [x] LSP server (`selvr-lsp`) and VS Code extension (`editors/vscode/`)
- [x] Formatter (`selvr fmt`) and linter (`selvr lint`)
- [x] Debugger adapter (`selvr dap`) and devtools integration (`std/devtools/`)
- [ ] Official public website, book, and API reference
- [ ] Community channels and 1.0 release

---

## Contributing

Selvr is in early development. Contributions of all kinds are welcome.

1. Read `SPEC.md` to understand the language design.
2. Open an issue before starting large work — many things are still in flux.
3. Run `cargo test` before submitting a pull request.
4. Follow the coding conventions in each crate's module-level comment.

---

## License

Licensed under either of:

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT License (`LICENSE-MIT`)

at your option.
