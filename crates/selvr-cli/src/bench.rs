//! Runtime harness for the three numeric workloads in `docs/benchmarks/selvr/`.
//!
//! Each workload is compiled by spawning this same `selvr` binary (`build --emit js`
//! and `build --emit bc`). The `.vlxc` file is executed by `selvr-vm`. The `.js`
//! file and a hand-written plain-JS twin of the same algorithm are executed by
//! Node. A mismatch is a failure, not a timing result.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use selvr_bytecode::decode;
use selvr_vm::mem::{HeapObj, Value};
use selvr_vm::Vm;

const BENCH_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/benchmarks");

/// Calls of `fib(40)` inside each timed sample. 40 stays inside i32, so the
/// VM's wrapping arithmetic and JavaScript's f64 agree.
const FIB_N: i32 = 40;
const FIB_INNER: u32 = 4_000;
/// One sieve of this limit per sample. π(10_000) = 1_229.
const SIEVE_LIMIT: i32 = 10_000;
const SIEVE_INNER: u32 = 1;
/// One n×n matmul per sample.
const MATMUL_N: i32 = 32;
const MATMUL_INNER: u32 = 1;

const SAMPLES: u32 = 5;

struct JsTimes {
    selvr: f64,
    plain: f64,
    selvr_ns: f64,
    plain_ns: f64,
}

struct Row {
    name: &'static str,
    detail: String,
    result: String,
    vm_ns: f64,
    js_ns: f64,
    plain_ns: f64,
}

pub fn cmd_bench() -> Result<()> {
    let root = PathBuf::from(BENCH_ROOT);
    let script = root.join("time-js.mjs");
    if !script.is_file() {
        anyhow::bail!("missing Node runner at {}", script.display());
    }

    let node_ver = node_version()?;
    let tmp = std::env::temp_dir().join(format!("selvr-bench-{}", std::process::id()));
    fs::create_dir_all(&tmp).with_context(|| format!("cannot create {}", tmp.display()))?;

    println!("Selvr runtime benchmark");
    println!("  compiler : this binary, `selvr build`");
    println!("  selvr-vm : native bytecode interpreter (not hosted inside WASM)");
    if cfg!(debug_assertions) {
        println!("  warning  : this binary is unoptimized; rebuild with --release before comparing");
    }
    println!("  node     : {node_ver}");
    println!("  timing   : median of {SAMPLES} samples, inputs built outside the timer");
    println!();

    let mut rows = Vec::new();
    rows.push(bench_fib(&tmp, &script)?);
    rows.push(bench_sieve(&tmp, &script)?);
    rows.push(bench_matmul(&tmp, &script)?);

    let _ = fs::remove_dir_all(&tmp);

    println!();
    println!(
        "{:<10} {:<22} {:>12} {:>12} {:>12} {:>12} {:>10}",
        "workload", "problem", "result", "selvr-vm", "selvr-js", "plain-js", "vm/plain"
    );
    println!("{}", "-".repeat(94));
    for row in &rows {
        println!(
            "{:<10} {:<22} {:>12} {:>12} {:>12} {:>12} {:>10}",
            row.name,
            row.detail,
            row.result,
            fmt_ms(row.vm_ns),
            fmt_ms(row.js_ns),
            fmt_ms(row.plain_ns),
            fmt_ratio(row.vm_ns, row.plain_ns),
        );
    }
    println!();
    println!("vm/plain is selvr-vm time divided by plain-JS time. 1.00 means the same time.");
    println!("selvr-js is the JavaScript from `selvr build`. plain-js is a hand-written twin of the .self file.");
    println!("On these loops a bytecode interpreter is expected to lose to V8, which JIT-compiles the JS.");
    Ok(())
}

fn bench_fib(tmp: &Path, script: &Path) -> Result<Row> {
    let src = PathBuf::from(BENCH_ROOT).join("selvr/fib.self");
    let (js, bc) = compile(tmp, &src, "fib")?;

    let ten = run_vm_once(&bc, |vm| {
        i32_result(&vm.call_by_name("fib", vec![Value::I32(10)])?)
    })?;
    expect_close("fib(10) vm", ten, 55.0)?;
    let js_ten = time_js(script, &js, "fib", 10, 1, 1)?;
    expect_close("fib(10) selvr-js", js_ten.selvr, 55.0)?;
    expect_close("fib(10) plain-js", js_ten.plain, 55.0)?;

    let vm = time_samples(&bc, SAMPLES, FIB_INNER, |module| {
        let mut vm = Vm::new(module.clone());
        let start = Instant::now();
        let mut last = 0.0;
        for _ in 0..FIB_INNER {
            last = i32_result(&vm.call_by_name("fib", vec![Value::I32(FIB_N)])?)?;
        }
        Ok((last, start.elapsed()))
    })?;
    let js_times = time_js(script, &js, "fib", FIB_N, FIB_INNER, SAMPLES)?;
    expect_close("fib vm vs selvr-js", vm.timed, js_times.selvr)?;
    expect_close("fib selvr-js vs plain", js_times.selvr, js_times.plain)?;
    Ok(Row {
        name: "fib",
        detail: format!("fib({FIB_N}) × {FIB_INNER}"),
        result: format!("{}", vm.timed as i64),
        vm_ns: vm.per_call_ns,
        js_ns: js_times.selvr_ns,
        plain_ns: js_times.plain_ns,
    })
}

fn bench_sieve(tmp: &Path, script: &Path) -> Result<Row> {
    let src = PathBuf::from(BENCH_ROOT).join("selvr/sieve.self");
    let (js, bc) = compile(tmp, &src, "sieve")?;

    // Absolute checks against known prime counts, then the timed limit.
    let small = run_vm_once(&bc, |vm| {
        i32_result(&vm.call_by_name("sieve", vec![Value::I32(10)])?)
    })?;
    expect_close("sieve(10)", small, 4.0)?;
    let hundred = run_vm_once(&bc, |vm| {
        i32_result(&vm.call_by_name("sieve", vec![Value::I32(100)])?)
    })?;
    expect_close("sieve(100)", hundred, 25.0)?;

    let vm = time_samples(&bc, SAMPLES, SIEVE_INNER, |module| {
        let mut vm = Vm::new(module.clone());
        let start = Instant::now();
        let mut last = 0.0;
        for _ in 0..SIEVE_INNER {
            last = i32_result(&vm.call_by_name("sieve", vec![Value::I32(SIEVE_LIMIT)])?)?;
        }
        Ok((last, start.elapsed()))
    })?;
    expect_close("sieve(10000) vm", vm.timed, 1229.0)?;
    let js10 = time_js(script, &js, "sieve", 10, 1, 1)?;
    expect_close("sieve(10) selvr-js", js10.selvr, 4.0)?;
    expect_close("sieve(10) plain-js", js10.plain, 4.0)?;
    let js100 = time_js(script, &js, "sieve", 100, 1, 1)?;
    expect_close("sieve(100) selvr-js", js100.selvr, 25.0)?;
    expect_close("sieve(100) plain-js", js100.plain, 25.0)?;

    let js_times = time_js(script, &js, "sieve", SIEVE_LIMIT, SIEVE_INNER, SAMPLES)?;
    expect_close("sieve(10000) selvr-js", js_times.selvr, 1229.0)?;
    expect_close("sieve(10000) plain-js", js_times.plain, 1229.0)?;
    expect_close("sieve vm vs selvr-js", vm.timed, js_times.selvr)?;
    Ok(Row {
        name: "sieve",
        detail: format!("sieve({SIEVE_LIMIT}) × {SIEVE_INNER}"),
        result: format!("{}", vm.timed as i64),
        vm_ns: vm.per_call_ns,
        js_ns: js_times.selvr_ns,
        plain_ns: js_times.plain_ns,
    })
}

fn bench_matmul(tmp: &Path, script: &Path) -> Result<Row> {
    let src = PathBuf::from(BENCH_ROOT).join("selvr/matmul.self");
    let (js, bc) = compile(tmp, &src, "matmul")?;

    let tiny = run_vm_once(&bc, |vm| matmul_sum(vm, 2))?;
    expect_close("matmul(2) vm", tiny, 2.75)?;
    let js_tiny = time_js(script, &js, "matmul", 2, 1, 1)?;
    expect_close("matmul(2) selvr-js", js_tiny.selvr, 2.75)?;
    expect_close("matmul(2) plain-js", js_tiny.plain, 2.75)?;

    let vm = time_samples(&bc, SAMPLES, MATMUL_INNER, |module| {
        let mut vm = Vm::new(module.clone());
        let (av, bv) = matmul_inputs(&mut vm, MATMUL_N);
        let start = Instant::now();
        let mut last = 0.0;
        for _ in 0..MATMUL_INNER {
            let out = vm.call_by_name("matmul", vec![av.clone(), bv.clone(), Value::I32(MATMUL_N)])?;
            last = sum_array(&vm, &out)?;
        }
        Ok((last, start.elapsed()))
    })?;
    let js_times = time_js(script, &js, "matmul", MATMUL_N, MATMUL_INNER, SAMPLES)?;
    expect_close("matmul selvr-js vs plain", js_times.selvr, js_times.plain)?;
    expect_close("matmul vm vs selvr-js", vm.timed, js_times.selvr)?;
    Ok(Row {
        name: "matmul",
        detail: format!("{MATMUL_N}×{MATMUL_N} × {MATMUL_INNER}"),
        result: format!("{:.4}", vm.timed),
        vm_ns: vm.per_call_ns,
        js_ns: js_times.selvr_ns,
        plain_ns: js_times.plain_ns,
    })
}

struct VmRun {
    /// Result of the last timed call.
    timed: f64,
    /// Median nanoseconds per call.
    per_call_ns: f64,
}

/// Each sample builds its own VM. `one` returns the call result and the
/// duration of the timed section only (input setup stays outside that clock).
fn time_samples<F>(bc_path: &Path, samples: u32, inner: u32, mut one: F) -> Result<VmRun>
where
    F: FnMut(&selvr_bytecode::BytecodeModule) -> Result<(f64, Duration)>,
{
    let bytes = fs::read(bc_path).with_context(|| format!("cannot read {}", bc_path.display()))?;
    let module = decode(&bytes).map_err(|e| anyhow::anyhow!("bytecode decode failed: {e}"))?;

    let mut per_call = Vec::with_capacity(samples as usize);
    let mut timed = 0.0;
    let inner_n = inner.max(1) as f64;
    for _ in 0..samples {
        let (result, elapsed) = one(&module)?;
        timed = result;
        per_call.push(duration_ns(elapsed) / inner_n);
    }
    per_call.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = per_call[per_call.len() / 2];
    Ok(VmRun { timed, per_call_ns: mid })
}

fn duration_ns(d: Duration) -> f64 {
    d.as_secs_f64() * 1e9
}

fn run_vm_once<F>(bc_path: &Path, mut body: F) -> Result<f64>
where
    F: FnMut(&mut Vm) -> Result<f64>,
{
    let bytes = fs::read(bc_path)?;
    let module = decode(&bytes).map_err(|e| anyhow::anyhow!("bytecode decode failed: {e}"))?;
    let mut vm = Vm::new(module);
    body(&mut vm)
}

fn i32_result(v: &Value) -> Result<f64> {
    match v {
        Value::I32(n) => Ok(*n as f64),
        other => anyhow::bail!("expected i32, found {}", other.type_name()),
    }
}

fn matmul_inputs(vm: &mut Vm, n: i32) -> (Value, Value) {
    let (a, b) = matrices(n);
    let av = vm.heap.alloc_array(a.into_iter().map(Value::F64).collect());
    let bv = vm.heap.alloc_array(b.into_iter().map(Value::F64).collect());
    (av, bv)
}

fn matmul_sum(vm: &mut Vm, n: i32) -> Result<f64> {
    let (av, bv) = matmul_inputs(vm, n);
    let out = vm.call_by_name("matmul", vec![av, bv, Value::I32(n)])?;
    sum_array(vm, &out)
}

fn sum_array(vm: &Vm, v: &Value) -> Result<f64> {
    let Value::Object(idx) = v else {
        anyhow::bail!("expected array, found {}", v.type_name());
    };
    let Some(HeapObj::Array(elems)) = vm.heap.get(*idx) else {
        anyhow::bail!("expected array");
    };
    let mut s = 0.0;
    for el in elems {
        match el {
            Value::F64(f) => s += f,
            other => anyhow::bail!("expected f64 element, found {}", other.type_name()),
        }
    }
    Ok(s)
}

/// Same matrices as `docs/benchmarks/time-js.mjs`.
fn matrices(n: i32) -> (Vec<f64>, Vec<f64>) {
    let len = (n as usize) * (n as usize);
    let mut a = vec![0.0; len];
    let mut b = vec![0.0; len];
    for i in 0..len {
        a[i] = (i % 7) as f64 * 0.5;
        b[i] = (i % 5) as f64 * 0.25;
    }
    (a, b)
}

fn compile(tmp: &Path, src: &Path, stem: &str) -> Result<(PathBuf, PathBuf)> {
    let js = tmp.join(format!("{stem}.js"));
    let bc = tmp.join(format!("{stem}.vlxc"));
    selvr_build(src, &js, "js")?;
    selvr_build(src, &bc, "bc")?;
    Ok((js, bc))
}

fn selvr_build(input: &Path, output: &Path, emit: &str) -> Result<()> {
    let exe = std::env::current_exe().context("cannot locate the selvr binary")?;
    let out = Command::new(&exe)
        .arg("build")
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--emit")
        .arg(emit)
        .output()
        .with_context(|| format!("failed to launch {}", exe.display()))?;
    if !out.status.success() {
        anyhow::bail!(
            "selvr build --emit {emit} {} failed:\n{}",
            input.display(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn time_js(script: &Path, js: &Path, kind: &str, param: i32, inner: u32, samples: u32) -> Result<JsTimes> {
    let out = Command::new("node")
        .arg(script)
        .arg(js)
        .arg(kind)
        .arg(param.to_string())
        .arg(inner.to_string())
        .arg(samples.to_string())
        .output()
        .context("could not start node — is Node.js installed?")?;
    if !out.status.success() {
        anyhow::bail!(
            "node runner failed for {kind}:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let text = String::from_utf8(out.stdout).context("node runner wrote non-utf8")?;
    let v: serde_json::Value = serde_json::from_str(text.trim())
        .with_context(|| format!("node runner wrote: {text}"))?;
    Ok(JsTimes {
        selvr: v["selvr"].as_f64().context("missing selvr result")?,
        plain: v["plain"].as_f64().context("missing plain result")?,
        selvr_ns: v["selvr_ns"].as_f64().context("missing selvr_ns")?,
        plain_ns: v["plain_ns"].as_f64().context("missing plain_ns")?,
    })
}

fn node_version() -> Result<String> {
    let out = Command::new("node")
        .arg("-v")
        .output()
        .context("could not start node — is Node.js installed?")?;
    if !out.status.success() {
        anyhow::bail!("`node -v` failed");
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn expect_close(label: &str, got: f64, want: f64) -> Result<()> {
    let scale = got.abs().max(want.abs()).max(1.0);
    if (got - want).abs() <= 1e-6 * scale {
        Ok(())
    } else {
        anyhow::bail!("{label}: got {got}, expected {want}");
    }
}

fn fmt_ms(ns: f64) -> String {
    if ns >= 100_000.0 {
        format!("{:.3} ms", ns / 1e6)
    } else {
        format!("{:.3} us", ns / 1e3)
    }
}

fn fmt_ratio(vm_ns: f64, plain_ns: f64) -> String {
    if plain_ns <= 0.0 {
        return "n/a".to_string();
    }
    format!("{:.2}", vm_ns / plain_ns)
}
