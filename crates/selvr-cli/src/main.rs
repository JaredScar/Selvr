use std::{fs, path::PathBuf, process};
use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use selvr_lexer::Lexer;
use selvr_parser::Parser;
use selvr_codegen::JsEmitter;
use selvr_ir::lower_module;
use selvr_target::{infer_targets, propagate_targets};
use selvr_bridge::BridgeEmitter;
use selvr_pkg::{Manifest, Lockfile, LockedPackage};
use selvr_fmt::Formatter;
use selvr_lint::{Linter, LintConfig, Severity};

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(ClapParser)]
#[command(
    name    = "selvr",
    version = env!("CARGO_PKG_VERSION"),
    about   = "The Selvr language compiler and toolchain — one language, two runtimes.",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Output format for `selvr build`.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum EmitMode {
    /// Emit a single `.js` file (JS transpiler — Phase 1).
    Js,
    /// Emit `app.wasm` + `app.js` + `app.loader.js` (hybrid compiler — Phase 2.5).
    Hybrid,
}

#[derive(Subcommand)]
enum Command {
    /// Compile a .self file.
    Build {
        /// Input .self file.
        input: PathBuf,
        /// Output file base name (default: same name as input, no extension).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// What to emit: `js` (transpiler) or `hybrid` (WASM+JS split).
        #[arg(long, value_enum, default_value = "js")]
        emit: EmitMode,
        /// Emit source maps alongside output (JS mode only).
        #[arg(long, default_value_t = true)]
        source_map: bool,
    },
    /// Run a .self file by compiling it to JS and executing with Node.js.
    Run {
        /// Input .self file.
        input: PathBuf,
    },
    /// Check a .self file for errors without producing output.
    Check {
        /// Input .self file.
        input: PathBuf,
    },
    /// Pretty-print the AST of a .self file (for debugging).
    Dump {
        /// Input .self file.
        input: PathBuf,
        /// `ast` | `ir` — what to dump.
        #[arg(long, default_value = "ast")]
        format: String,
    },
    /// Print the compile-time WASM/JS targeting decision for every function.
    ///
    /// Example:
    ///   selvr explain app.self
    ///
    /// Output:
    ///   [wasm]  blur          score: 120  reason: >= 10 numeric ops inside loop body; ...
    ///   [js]    onClick       score: -1000  reason: calls a DOM API
    Explain {
        /// Input .self file.
        input: PathBuf,
        /// Emit machine-readable JSON instead of the human-readable table.
        #[arg(long)]
        json: bool,
    },

    // ── Package manager ───────────────────────────────────────────────────────

    /// Add a dependency to selvr.toml (and update selvr.lock).
    ///
    /// Example:
    ///   selvr add selvr-std            # latest version
    ///   selvr add selvr-std@1.2.0      # pinned version
    ///   selvr add ../local-lib --path  # local path dependency
    Add {
        /// Package name, optionally with version: `name[@version]`
        package: String,
        /// Add as a dev-dependency (only used during `selvr test`).
        #[arg(long)]
        dev: bool,
        /// Add a local path dependency instead of a registry package.
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Install all dependencies listed in selvr.toml (resolves & updates selvr.lock).
    ///
    /// Example:
    ///   selvr install
    Install {
        /// Only install packages already in selvr.lock; never update versions.
        #[arg(long)]
        frozen: bool,
    },

    /// Remove a dependency from selvr.toml and selvr.lock.
    ///
    /// Example:
    ///   selvr remove selvr-std
    Remove {
        /// Package name to remove.
        package: String,
    },

    /// Publish the current package to the Selvr registry.
    ///
    /// Requires `SELVR_TOKEN` environment variable (or `--token` flag).
    ///
    /// Example:
    ///   selvr publish
    ///   selvr publish --dry-run
    Publish {
        /// Check everything without actually uploading.
        #[arg(long)]
        dry_run: bool,
        /// Registry authentication token (default: $SELVR_TOKEN env var).
        #[arg(long)]
        token: Option<String>,
        /// Registry base URL (default: https://pkg.selvr-lang.org).
        #[arg(long)]
        registry: Option<String>,
    },

    /// Search the registry for packages.
    ///
    /// Example:
    ///   selvr search json
    Search {
        /// Search query.
        query: String,
        /// Maximum number of results to show.
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// Run all #[test] functions in a .self file.
    ///
    /// Example:
    ///   selvr test src/main.self
    Test {
        /// Input .self file (or directory — runs all *.self tests inside).
        input: PathBuf,
        /// Only run tests whose name contains this string.
        #[arg(long)]
        filter: Option<String>,
    },

    /// Format a .self file in-place.
    ///
    /// Example:
    ///   selvr fmt src/main.self
    Fmt {
        /// File(s) to format.
        inputs: Vec<PathBuf>,
        /// Check formatting without modifying files; exit non-zero if unformatted.
        #[arg(long)]
        check: bool,
    },

    /// Initialise a new Selvr project in the current directory.
    ///
    /// Creates `selvr.toml`, `src/main.self`, and `selvr.lock`.
    Init {
        /// Project name (default: current directory name).
        name: Option<String>,
    },

    // ── Developer tools ───────────────────────────────────────────────────────

    /// Lint a .self file and report diagnostics.
    ///
    /// Rules:
    ///   SL001 unused_variable   SL002 dead_code       SL003 missing_return
    ///   SL004 shadow_variable   SL005 force_target     SL006 wasm_dom_call
    ///   SL007 js_heavy_compute
    ///
    /// Example:
    ///   selvr lint src/main.self
    ///   selvr lint src/main.self --json
    Lint {
        /// Input file(s) to lint. Defaults to src/ if omitted.
        inputs: Vec<PathBuf>,
        /// Emit diagnostics as newline-delimited JSON.
        #[arg(long)]
        json: bool,
        /// Exit non-zero even for warnings (treat warnings as errors).
        #[arg(long)]
        strict: bool,
    },

    /// Start the Language Server Protocol (LSP) server over stdin/stdout.
    ///
    /// Normally invoked automatically by the VS Code extension (or any
    /// LSP-capable editor configured to use `selvr lsp` as its language server).
    ///
    /// Example:
    ///   selvr lsp
    Lsp {},

    /// Start the Debug Adapter Protocol (DAP) server over stdin/stdout.
    ///
    /// Normally invoked automatically by VS Code when the user presses F5
    /// on a `.self` file. Supports WASM/JS unified debugging with source maps.
    ///
    /// Example:
    ///   selvr dap
    Dap {},
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Build   { input, output, emit, source_map } =>
            cmd_build(input, output, emit, source_map),
        Command::Run     { input }           => cmd_run(input),
        Command::Check   { input }           => cmd_check(input),
        Command::Dump    { input, format }   => cmd_dump(input, format),
        Command::Explain { input, json }     => cmd_explain(input, json),
        // Package manager
        Command::Add     { package, dev, path }           => cmd_add(package, dev, path),
        Command::Install { frozen }                       => cmd_install(frozen),
        Command::Remove  { package }                      => cmd_remove(package),
        Command::Publish { dry_run, token, registry }     => cmd_publish(dry_run, token, registry),
        Command::Search  { query, limit }                 => cmd_search(query, limit),
        Command::Test    { input, filter }                => cmd_test(input, filter),
        Command::Fmt     { inputs, check }                => cmd_fmt(inputs, check),
        Command::Init    { name }                         => cmd_init(name),
        Command::Lint    { inputs, json, strict }         => cmd_lint(inputs, json, strict),
        Command::Lsp     {}                               => cmd_lsp(),
        Command::Dap     {}                               => cmd_dap(),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

// ── Shared: parse .self source ────────────────────────────────────────────────

fn parse_source(input: &PathBuf) -> anyhow::Result<selvr_parser::ast::Module> {
    use anyhow::Context;
    let src = fs::read_to_string(input)
        .with_context(|| format!("cannot read `{}`", input.display()))?;
    let (tokens, lex_errors) = Lexer::new(&src, 0).tokenize();
    for e in &lex_errors { eprintln!("lex error: {e}"); }
    let (module, parse_errors) = Parser::new(tokens, 0).parse();
    for e in &parse_errors { eprintln!("parse error: {e}"); }
    if !lex_errors.is_empty() || !parse_errors.is_empty() {
        anyhow::bail!("compilation failed with errors");
    }
    Ok(module)
}

// ── Commands ──────────────────────────────────────────────────────────────────

fn cmd_build(
    input:           PathBuf,
    output:          Option<PathBuf>,
    emit:            EmitMode,
    emit_source_map: bool,
) -> anyhow::Result<()> {
    use anyhow::Context;

    let module = parse_source(&input)?;
    let src_name = input.display().to_string();

    match emit {
        // ── JS transpiler (Phase 1) ───────────────────────────────────────────
        EmitMode::Js => {
            let out_path = output.unwrap_or_else(|| input.with_extension("js"));
            let out_name = out_path.display().to_string();

            let emitter = JsEmitter::new(&out_name, &src_name);
            let (js, sm) = emitter.emit_module(&module)?;

            fs::write(&out_path, &js)
                .with_context(|| format!("cannot write `{out_name}`"))?;
            println!("compiled `{src_name}` → `{out_name}`");

            if emit_source_map {
                let sm_path = out_path.with_extension("js.map");
                fs::write(&sm_path, &sm)
                    .with_context(|| format!("cannot write source map `{}`", sm_path.display()))?;
            }
        }

        // ── Hybrid WASM+JS (Phase 2.5) ────────────────────────────────────────
        EmitMode::Hybrid => {
            let base = output.unwrap_or_else(|| input.with_extension(""));
            let base_str = base.display().to_string();
            let base_str = base_str.trim_end_matches('.');

            // Lower AST → IR.
            let mut ir = lower_module(&module);

            // Targeting pass: infer + propagate.
            let mut map = infer_targets(&mut ir);
            let changed = propagate_targets(&mut ir, &mut map);
            if !changed.is_empty() {
                eprintln!(
                    "info: propagation re-targeted {} function(s): {}",
                    changed.len(),
                    changed.join(", ")
                );
            }

            // Emit the bridge artefacts.
            let bridge = BridgeEmitter::new(&ir, &map).emit();

            // JS-targeted functions: transpile via selvr-codegen.
            // (Full wiring: future — Phase 2.5 full integration)
            // For now, emit the JS bridge wrappers + loader.
            let bridge_js_path  = format!("{base_str}.bridge.js");
            let loader_js_path  = format!("{base_str}.loader.js");
            let report_json_path = format!("{base_str}.split-report.json");

            fs::write(&bridge_js_path, &bridge.js)
                .with_context(|| format!("cannot write `{bridge_js_path}`"))?;
            fs::write(&loader_js_path, &bridge.loader)
                .with_context(|| format!("cannot write `{loader_js_path}`"))?;
            fs::write(&report_json_path, &bridge.report)
                .with_context(|| format!("cannot write `{report_json_path}`"))?;

            // Summary report.
            let (w, j, _) = map.summary();
            println!(
                "compiled `{src_name}` [hybrid]\n  → {bridge_js_path}  ({w} WASM exports wrapped)\n  → {loader_js_path}  (unified loader)\n  → {report_json_path}  ({j} JS + {w} WASM functions)"
            );
            println!("\n{}", map.explain());

            // Warn about #[wasm] / #[js] conflicts.
            emit_target_warnings(&map);
        }
    }

    Ok(())
}

fn cmd_run(input: PathBuf) -> anyhow::Result<()> {
    use anyhow::Context;
    let tmp = std::env::temp_dir().join("selvr_out.js");
    cmd_build(input, Some(tmp.clone()), EmitMode::Js, false)?;
    let status = std::process::Command::new("node")
        .arg(&tmp)
        .status()
        .with_context(|| "could not start node — is Node.js installed?")?;
    if !status.success() {
        anyhow::bail!("program exited with status {}", status);
    }
    Ok(())
}

fn cmd_check(input: PathBuf) -> anyhow::Result<()> {
    use anyhow::Context;
    let src = fs::read_to_string(&input)
        .with_context(|| format!("cannot read `{}`", input.display()))?;
    let (tokens, lex_errors) = Lexer::new(&src, 0).tokenize();
    let (_, parse_errors) = Parser::new(tokens, 0).parse();
    let total = lex_errors.len() + parse_errors.len();
    for e in &lex_errors   { eprintln!("lex error: {e}"); }
    for e in &parse_errors { eprintln!("parse error: {e}"); }
    if total > 0 {
        anyhow::bail!("{total} error(s)");
    }
    println!("ok");
    Ok(())
}

fn cmd_dump(input: PathBuf, format: String) -> anyhow::Result<()> {
    let module = parse_source(&input)?;
    match format.as_str() {
        "ir" => {
            let mut ir = lower_module(&module);
            let mut map = infer_targets(&mut ir);
            propagate_targets(&mut ir, &mut map);
            println!("{ir:#?}");
        }
        _ => println!("{module:#?}"),
    }
    Ok(())
}

/// `selvr explain app.self` — print a full targeting report.
fn cmd_explain(input: PathBuf, json: bool) -> anyhow::Result<()> {
    let module = parse_source(&input)?;
    let mut ir = lower_module(&module);
    let mut map = infer_targets(&mut ir);
    let changed = propagate_targets(&mut ir, &mut map);

    if json {
        println!("{}", map.to_json());
    } else {
        println!("{}", map.explain());
        if !changed.is_empty() {
            println!(
                "note: call-graph propagation changed {} function target(s): {}",
                changed.len(), changed.join(", ")
            );
        }
    }
    Ok(())
}

// ── Warnings for contradictory forced targets ─────────────────────────────────

fn emit_target_warnings(map: &selvr_target::TargetMap) {
    for rec in map.fns.values() {
        if !rec.forced { continue; }
        // A forced-WASM function that ends up calling JS (score implies DOM).
        if rec.target == selvr_target::Target::Wasm && rec.score < 0 {
            eprintln!(
                "warning: `{}` is annotated `#[wasm]` but analysis score ({}) suggests it \
                 belongs in JS — remove #[wasm] or ensure it has no DOM calls",
                rec.name, rec.score
            );
        }
        // A forced-JS function that looks compute-heavy.
        if rec.target == selvr_target::Target::Js && rec.score > WASM_THRESHOLD_WARN {
            eprintln!(
                "warning: `{}` is annotated `#[js]` but analysis score ({}) suggests it \
                 would benefit from WASM — consider removing #[js]",
                rec.name, rec.score
            );
        }
    }
}

const WASM_THRESHOLD_WARN: i32 = 50;

// ── Package manager commands ──────────────────────────────────────────────────

/// `selvr add <package[@version]> [--dev] [--path <dir>]`
fn cmd_add(package: String, dev: bool, path: Option<PathBuf>) -> anyhow::Result<()> {
    use anyhow::Context;
    let manifest_path = find_manifest()?;
    let mut manifest  = Manifest::from_file(&manifest_path)
        .context("failed to parse selvr.toml")?;

    // Split "name@version" or just "name"
    let (name, version_req) = if let Some(pos) = package.find('@') {
        (&package[..pos], package[pos+1..].to_string())
    } else {
        (package.as_str(), "*".to_string())
    };

    let dep = if let Some(p) = path {
        selvr_pkg::manifest::Dependency::Table(selvr_pkg::manifest::DependencyTable {
            version:  version_req,
            path:     Some(p.display().to_string()),
            optional: false,
            features: Vec::new(),
        })
    } else {
        selvr_pkg::manifest::Dependency::Version(version_req.clone())
    };

    let deps = if dev { &mut manifest.dev_dependencies } else { &mut manifest.dependencies };
    deps.insert(name.to_string(), dep);

    let toml = manifest.to_toml().context("failed to serialise selvr.toml")?;
    fs::write(&manifest_path, toml).context("failed to write selvr.toml")?;
    println!("added `{name}` to {}", if dev { "dev-dependencies" } else { "dependencies" });
    println!("run `selvr install` to download and lock");
    Ok(())
}

/// `selvr install [--frozen]`
fn cmd_install(frozen: bool) -> anyhow::Result<()> {
    use anyhow::Context;
    let manifest_path = find_manifest()?;
    let manifest      = Manifest::from_file(&manifest_path)
        .context("failed to parse selvr.toml")?;

    let lock_path = manifest_path.with_file_name("selvr.lock");

    if frozen {
        // Read existing lockfile; error if missing.
        if !lock_path.exists() {
            anyhow::bail!("selvr.lock not found; run `selvr install` without --frozen first");
        }
        let lock = Lockfile::from_file(&lock_path).context("failed to parse selvr.lock")?;
        let stale = selvr_pkg::resolver::check_lockfile(&manifest, &lock.packages);
        if !stale.is_empty() {
            anyhow::bail!("lockfile is out of date for: {}. Remove --frozen to update.", stale.join(", "));
        }
        println!("lockfile is up to date ({} packages locked)", lock.packages.len());
        return Ok(());
    }

    println!("resolving dependencies for `{}`…", manifest.package.name);

    // Simulate resolution (registry client to be wired in Phase 3).
    // For now, resolve local-path deps only and print what would be fetched.
    let mut lock = if lock_path.exists() {
        Lockfile::from_file(&lock_path).unwrap_or_default()
    } else {
        Lockfile::new()
    };

    let mut installed = 0usize;
    for (name, dep) in manifest.all_dependencies() {
        if let Some(local) = dep.local_path() {
            let pkg = LockedPackage {
                name:         name.clone(),
                version:      dep.version_req().to_string(),
                registry:     format!("path:{local}"),
                checksum:     "local".into(),
                dependencies: Vec::new(),
            };
            lock.upsert(pkg);
            println!("  locked `{name}` (local path: {local})");
            installed += 1;
        } else {
            println!("  would fetch `{name}@{}` from registry (registry client: Phase 3)", dep.version_req());
            let pkg = LockedPackage {
                name:         name.clone(),
                version:      dep.version_req().trim_start_matches('^').trim_start_matches('~').to_string(),
                registry:     selvr_pkg::registry::DEFAULT_REGISTRY.into(),
                checksum:     "pending".into(),
                dependencies: Vec::new(),
            };
            lock.upsert(pkg);
            installed += 1;
        }
    }

    lock.sort();
    lock.to_file(&lock_path).context("failed to write selvr.lock")?;
    println!("\n{installed} package(s) locked → selvr.lock");
    Ok(())
}

/// `selvr remove <package>`
fn cmd_remove(package: String) -> anyhow::Result<()> {
    use anyhow::Context;
    let manifest_path = find_manifest()?;
    let mut manifest  = Manifest::from_file(&manifest_path)
        .context("failed to parse selvr.toml")?;

    let removed_main = manifest.dependencies.remove(&package).is_some();
    let removed_dev  = manifest.dev_dependencies.remove(&package).is_some();

    if !removed_main && !removed_dev {
        anyhow::bail!("`{package}` is not a dependency of this package");
    }

    let toml = manifest.to_toml().context("failed to serialise selvr.toml")?;
    fs::write(&manifest_path, toml).context("failed to write selvr.toml")?;

    // Also remove from lockfile.
    let lock_path = manifest_path.with_file_name("selvr.lock");
    if lock_path.exists() {
        let mut lock = Lockfile::from_file(&lock_path).unwrap_or_default();
        lock.remove(&package);
        lock.sort();
        lock.to_file(&lock_path).context("failed to write selvr.lock")?;
    }

    println!("removed `{package}` from selvr.toml");
    Ok(())
}

/// `selvr publish [--dry-run] [--token <tok>] [--registry <url>]`
fn cmd_publish(dry_run: bool, token: Option<String>, registry: Option<String>) -> anyhow::Result<()> {
    let manifest_path = find_manifest()?;
    let manifest      = Manifest::from_file(&manifest_path)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let reg = registry.as_deref()
        .unwrap_or(selvr_pkg::registry::DEFAULT_REGISTRY);
    let _tok = token
        .or_else(|| std::env::var("SELVR_TOKEN").ok())
        .unwrap_or_default();

    println!("publishing `{}` v{} to {reg}", manifest.package.name, manifest.package.version);

    if dry_run {
        println!("[dry-run] would upload package — no changes made");
        println!("  name:    {}", manifest.package.name);
        println!("  version: {}", manifest.package.version);
        println!("  license: {}", manifest.package.license.as_deref().unwrap_or("(none)"));
        println!("  entry:   {}", manifest.build.entry);
        return Ok(());
    }

    // Full publish (HTTP upload) is wired in Phase 3.
    println!("note: registry HTTP upload is implemented in Phase 3.");
    println!("      run `selvr publish --dry-run` to preview the package metadata.");
    Ok(())
}

/// `selvr search <query> [--limit N]`
fn cmd_search(query: String, limit: usize) -> anyhow::Result<()> {
    let reg = selvr_pkg::registry::Registry::default();
    let url = reg.search_url(&query, limit);
    println!("searching registry for `{query}`…");
    println!("  (registry client is implemented in Phase 3)");
    println!("  search URL: {url}");
    println!();
    println!("  Example results:");
    println!("  selvr-std    1.0.0  The official Selvr standard library");
    println!("  selvr-test   1.0.0  Built-in test runner");
    println!("  selvr-dom    0.2.0  DOM utility wrappers");
    Ok(())
}

/// `selvr test <input> [--filter <name>]`
fn cmd_test(input: PathBuf, filter: Option<String>) -> anyhow::Result<()> {
    use anyhow::Context;

    // Collect .self files to test
    let files: Vec<PathBuf> = if input.is_dir() {
        let mut v = Vec::new();
        for entry in std::fs::read_dir(&input).context("cannot read directory")? {
            let e = entry?;
            if e.path().extension().map_or(false, |x| x == "self") {
                v.push(e.path());
            }
        }
        v
    } else {
        vec![input]
    };

    let mut failed = 0usize;

    for file in &files {
        let module = parse_source(file)?;
        // Collect #[test] functions from the AST
        let test_fns: Vec<&str> = module.items.iter()
            .filter_map(|item| {
                if let selvr_parser::ast::Item::FnDef(f) = item {
                    let is_test = f.attrs.iter().any(|a| a.name.as_str() == "test");
                    if is_test { Some(f.name.as_str()) } else { None }
                } else { None }
            })
            .collect();

        if test_fns.is_empty() {
            println!("{}: no #[test] functions found", file.display());
            continue;
        }

        println!("running {} test(s) in {}:", test_fns.len(), file.display());

        // Compile the file to JS and run it with Node.js, injecting a test harness.
        let tmp_dir = std::env::temp_dir();
        let tmp_js  = tmp_dir.join("selvr_test.js");

        let src_name  = file.display().to_string();
        let out_name  = tmp_js.display().to_string();
        let emitter   = JsEmitter::new(&out_name, &src_name);
        let (mut js, _) = emitter.emit_module(&module)?;

        // Inject test runner harness
        js.push_str("\n// --- selvr test harness ---\n");
        js.push_str("let __passed = 0, __failed = 0;\n");
        for name in &test_fns {
            if let Some(ref f) = filter {
                if !name.contains(f.as_str()) { continue; }
            }
            js.push_str(&format!(
                "try {{ {name}(); __passed++; process.stdout.write('  ✓ {name}\\n'); }}\
                 catch(e) {{ __failed++; process.stderr.write('  ✗ {name}: ' + e.message + '\\n'); }}\n"
            ));
        }
        js.push_str("console.log(`\\n${{__passed}}/${{__passed+__failed}} tests passed.`);\n");
        js.push_str("if (__failed > 0) process.exit(1);\n");

        fs::write(&tmp_js, &js)?;

        let status = std::process::Command::new("node")
            .arg(&tmp_js)
            .status()
            .context("could not start node — is Node.js installed?")?;

        if !status.success() {
            failed += 1;
        }
    }

    if failed > 0 {
        anyhow::bail!("test run failed");
    }
    Ok(())
}

/// `selvr fmt [files...] [--check]`
fn cmd_fmt(inputs: Vec<PathBuf>, check: bool) -> anyhow::Result<()> {
    use anyhow::Context;

    let files = if inputs.is_empty() {
        let mut v = Vec::new();
        if let Ok(entries) = std::fs::read_dir("src") {
            for e in entries.flatten() {
                if e.path().extension().map_or(false, |x| x == "self") {
                    v.push(e.path());
                }
            }
        }
        v
    } else {
        inputs
    };

    if files.is_empty() {
        println!("no .self files to format");
        return Ok(());
    }

    let fmt = Formatter::new();
    let mut any_changed = false;

    for file in &files {
        let src = fs::read_to_string(file)
            .with_context(|| format!("cannot read `{}`", file.display()))?;
        match fmt.format_src(&src) {
            Ok(formatted) => {
                if formatted != src {
                    any_changed = true;
                    if check {
                        println!("{}: would reformat", file.display());
                    } else {
                        fs::write(file, &formatted)
                            .with_context(|| format!("cannot write `{}`", file.display()))?;
                        println!("formatted {}", file.display());
                    }
                } else if check {
                    println!("{}: ok", file.display());
                }
            }
            Err(e) => {
                eprintln!("{}: format error — {e}", file.display());
                any_changed = true;
            }
        }
    }

    if check && any_changed {
        anyhow::bail!("some files need formatting (run `selvr fmt` to fix)");
    }
    Ok(())
}

// ── Developer tool commands ───────────────────────────────────────────────────

/// `selvr lint [files...] [--json] [--strict]`
fn cmd_lint(inputs: Vec<PathBuf>, json: bool, strict: bool) -> anyhow::Result<()> {
    use anyhow::Context;

    let files = if inputs.is_empty() {
        let mut v = Vec::new();
        if let Ok(entries) = std::fs::read_dir("src") {
            for e in entries.flatten() {
                if e.path().extension().map_or(false, |x| x == "self") {
                    v.push(e.path());
                }
            }
        }
        v
    } else {
        inputs
    };

    if files.is_empty() {
        println!("no .self files to lint");
        return Ok(());
    }

    let linter = Linter::new(LintConfig::default());
    let mut total_errors   = 0usize;
    let mut total_warnings = 0usize;
    let mut all_diags = Vec::new();

    for file in &files {
        let src = fs::read_to_string(file)
            .with_context(|| format!("cannot read `{}`", file.display()))?;
        let diags = linter.check_src(&src);

        for d in &diags {
            match d.severity {
                Severity::Error   => total_errors   += 1,
                Severity::Warning => total_warnings += 1,
                _ => {}
            }
        }

        if json {
            all_diags.extend(diags);
        } else {
            for d in &diags {
                let prefix = file.display();
                eprintln!("{prefix}:{}:{}: {} [{}] — {}", d.span.line, d.span.col, d.severity, d.code, d.message);
                if let Some(fix) = &d.fix {
                    eprintln!("  help: {fix}");
                }
            }
        }
    }

    if json {
        println!("{}", selvr_lint::Linter::to_ndjson(&all_diags));
    } else {
        let warns = if total_warnings > 0 { format!("{total_warnings} warning(s)") } else { String::new() };
        let errs  = if total_errors   > 0 { format!("{total_errors} error(s)")   } else { String::new() };
        let summary = [errs, warns].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", ");
        if summary.is_empty() {
            println!("lint: {} file(s) — no issues found ✓", files.len());
        } else {
            eprintln!("lint: {summary}");
        }
    }

    if total_errors > 0 || (strict && total_warnings > 0) {
        anyhow::bail!("lint failed");
    }
    Ok(())
}

/// `selvr lsp` — spawn the LSP server binary (delegates to `selvr-lsp`).
fn cmd_lsp() -> anyhow::Result<()> {
    // Prefer `selvr-lsp` on PATH; fall back to running this binary with `lsp`.
    let status = std::process::Command::new("selvr-lsp")
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => anyhow::bail!("selvr-lsp exited with {s}"),
        Err(_) => {
            // Binary not found — print installation instructions.
            eprintln!("selvr-lsp: binary not found on PATH.");
            eprintln!("Build it with:  cargo build -p selvr-lsp --release");
            eprintln!("Then copy the binary to a directory on your PATH.");
            eprintln!();
            eprintln!("VS Code users: install the Selvr extension from editors/vscode/.");
            eprintln!("The extension automatically discovers the language server.");
            anyhow::bail!("selvr-lsp not found");
        }
    }
}

/// `selvr dap` — spawn the DAP server binary (delegates to `selvr-dap`).
fn cmd_dap() -> anyhow::Result<()> {
    let status = std::process::Command::new("selvr-dap")
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => anyhow::bail!("selvr-dap exited with {s}"),
        Err(_) => {
            eprintln!("selvr-dap: binary not found on PATH.");
            eprintln!("Build it with:  cargo build -p selvr-dap --release");
            eprintln!("Then copy the binary to a directory on your PATH.");
            anyhow::bail!("selvr-dap not found");
        }
    }
}

/// `selvr init [name]`
fn cmd_init(name: Option<String>) -> anyhow::Result<()> {
    let pkg_name = name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "my-selvr-app".into())
    });

    if PathBuf::from("selvr.toml").exists() {
        anyhow::bail!("`selvr.toml` already exists in this directory");
    }

    // Create selvr.toml
    let manifest = format!(r#"[package]
name        = "{pkg_name}"
version     = "0.1.0"
authors     = []
license     = "MIT"
description = "A Selvr project"

[dependencies]
selvr-std = "1.0.0"

[build]
emit  = "js"
opt   = "debug"
entry = "src/main.self"
"#);
    fs::write("selvr.toml", manifest)?;

    // Create src/main.self
    fs::create_dir_all("src")?;
    if !PathBuf::from("src/main.self").exists() {
        fs::write("src/main.self", r#"// src/main.self — entry point

fn main(): void {
    console.log("Hello from Selvr!");
}
"#)?;
    }

    // Create empty lockfile
    Lockfile::new().to_file(&PathBuf::from("selvr.lock"))
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    println!("initialised project `{pkg_name}`");
    println!("  selvr.toml    — manifest");
    println!("  src/main.self — entry point");
    println!("  selvr.lock    — lockfile (empty)");
    println!();
    println!("run `selvr build src/main.self` to compile");
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Walk up the directory tree to find `selvr.toml`.
fn find_manifest() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    Manifest::find_root(&cwd)
        .ok_or_else(|| anyhow::anyhow!(
            "could not find `selvr.toml` — are you inside a Selvr project? \
             Run `selvr init` to create one."
        ))
}
