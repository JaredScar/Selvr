#!/usr/bin/env bash
# benchmarks/compile.sh
#
# Compare Selvr compile times against TypeScript (tsc) and esbuild
# for progressively larger synthetic source files.
#
# Prerequisites:
#   cargo build --release -p selvr-cli
#   npm install -g typescript esbuild
#
# Usage:
#   chmod +x benchmarks/compile.sh
#   ./benchmarks/compile.sh
#
# Output: a table printed to stdout.

set -euo pipefail

SELVR_BIN="./target/release/SELVR"
SCRATCH_DIR=$(mktemp -d)
trap 'rm -rf "$SCRATCH_DIR"' EXIT

# ── Sizes to test ─────────────────────────────────────────────────────────────
SIZES=(100 500 1000 5000 10000)

# ── Generate a synthetic Selvr source file of N functions ─────────────────────
gen_SELVR() {
  local n="$1"
  local out="$SCRATCH_DIR/bench_${n}.self"
  {
    for i in $(seq 1 "$n"); do
      echo "fn func${i}(x: i32, y: i32): i32 { return x + y * ${i}; }"
    done
    echo "fn main(): void { console.log(func1(1, 2)); }"
  } > "$out"
  echo "$out"
}

# ── Generate an equivalent TypeScript file ────────────────────────────────────
gen_ts() {
  local n="$1"
  local out="$SCRATCH_DIR/bench_${n}.ts"
  {
    for i in $(seq 1 "$n"); do
      echo "function func${i}(x: number, y: number): number { return x + y * ${i}; }"
    done
    echo "console.log(func1(1, 2));"
  } > "$out"
  echo "$out"
}

# ── Generate an equivalent JS file (for esbuild) ─────────────────────────────
gen_js() {
  local n="$1"
  local out="$SCRATCH_DIR/bench_${n}.js"
  {
    for i in $(seq 1 "$n"); do
      echo "function func${i}(x, y) { return x + y * ${i}; }"
    done
    echo "console.log(func1(1, 2));"
  } > "$out"
  echo "$out"
}

# ── Time a command, return milliseconds ───────────────────────────────────────
time_ms() {
  local start end
  start=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
  "$@" >/dev/null 2>&1 || true
  end=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
  echo $(( (end - start) / 1000000 ))
}

# ── Run ───────────────────────────────────────────────────────────────────────

printf "%-10s  %-12s  %-12s  %-12s\n" "Functions" "Selvr (ms)" "tsc (ms)" "esbuild (ms)"
printf "%-10s  %-12s  %-12s  %-12s\n" "─────────" "──────────" "────────" "────────────"

for N in "${SIZES[@]}"; do
  vx_file=$(gen_SELVR "$N")
  ts_file=$(gen_ts "$N")
  js_file=$(gen_js "$N")

  # Selvr: parse + typecheck + emit bytecode (no JS codegen)
  SELVR_ms="N/A"
  if [[ -x "$SELVR_BIN" ]]; then
    SELVR_ms=$(time_ms "$SELVR_BIN" compile --emit bc -o /dev/null "$vx_file")
  fi

  # TypeScript: full type-check (no emit)
  tsc_ms="N/A"
  if command -v tsc &>/dev/null; then
    tsc_ms=$(time_ms tsc --noEmit --strict --target ES2022 "$ts_file")
  fi

  # esbuild: bundle-only (no type-check)
  esbuild_ms="N/A"
  if command -v esbuild &>/dev/null; then
    esbuild_ms=$(time_ms esbuild --bundle --outfile=/dev/null "$js_file")
  fi

  printf "%-10s  %-12s  %-12s  %-12s\n" "$N" "${SELVR_ms}" "${tsc_ms}" "${esbuild_ms}"
done

echo ""
echo "Note: Selvr times include parse + type-check + bytecode emit."
echo "      tsc times are type-check only (no JS emit)."
echo "      esbuild times are bundle-only (no type-check)."
