#!/usr/bin/env bash
# Time `selvr build` on generated source files, and optionally tsc and esbuild.
#
# The .self files are synthetic: N one-line functions, not an application and
# not React. A missing tool is reported as "not installed". A failing command
# aborts the script. Durations are a single wall-clock sample, so they are noisy.
#
# From the repository root, after `cargo build --release -p selvr-cli`:
#   ./docs/benchmarks/compile.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if [[ -x ./target/release/selvr ]]; then
  SELVR="./target/release/selvr"
elif [[ -x ./target/release/selvr.exe ]]; then
  SELVR="./target/release/selvr.exe"
else
  echo "error: target/release/selvr not found. Run: cargo build --release -p selvr-cli" >&2
  exit 1
fi

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

SIZES=(100 500 1000 5000)

now_ns() {
  local t
  t="$(date +%s%N 2>/dev/null || true)"
  if [[ "$t" =~ ^[0-9]{16,}$ ]]; then
    printf '%s\n' "$t"
  elif command -v python >/dev/null 2>&1; then
    python -c 'import time; print(int(time.time()*1e9))'
  elif command -v python3 >/dev/null 2>&1; then
    python3 -c 'import time; print(int(time.time()*1e9))'
  else
    echo "error: need \`date +%s%N\` or python to measure time" >&2
    exit 1
  fi
}

# Print milliseconds. Exit the script if the command fails.
time_ms() {
  local start end
  start="$(now_ns)"
  if ! "$@" >/dev/null; then
    echo "error: command failed: $*" >&2
    exit 1
  fi
  end="$(now_ns)"
  echo $(( (end - start) / 1000000 ))
}

gen_self() {
  local n="$1"
  local out="$SCRATCH/bench_${n}.self"
  : > "$out"
  local i
  for i in $(seq 1 "$n"); do
    printf 'fn func%s(x: i32, y: i32): i32 { return x + y * %s; }\n' "$i" "$i" >> "$out"
  done
  printf 'fn main(): void { }\n' >> "$out"
  printf '%s\n' "$out"
}

gen_ts() {
  local n="$1"
  local out="$SCRATCH/bench_${n}.ts"
  : > "$out"
  local i
  for i in $(seq 1 "$n"); do
    printf 'function func%s(x: number, y: number): number { return x + y * %s; }\n' "$i" "$i" >> "$out"
  done
  printf '%s\n' "$out"
}

gen_js() {
  local n="$1"
  local out="$SCRATCH/bench_${n}.js"
  : > "$out"
  local i
  for i in $(seq 1 "$n"); do
    printf 'function func%s(x, y) { return x + y * %s; }\n' "$i" "$i" >> "$out"
  done
  printf '%s\n' "$out"
}

has_tsc=0
has_esbuild=0
command -v tsc >/dev/null 2>&1 && has_tsc=1
command -v esbuild >/dev/null 2>&1 && has_esbuild=1

printf '%-10s  %-14s  %-14s  %-14s  %-14s\n' "Functions" "selvr js (ms)" "selvr bc (ms)" "tsc (ms)" "esbuild (ms)"
printf '%-10s  %-14s  %-14s  %-14s  %-14s\n' "---------" "-------------" "-------------" "--------" "------------"

for N in "${SIZES[@]}"; do
  self_file="$(gen_self "$N")"
  ts_file="$(gen_ts "$N")"
  js_file="$(gen_js "$N")"

  selvr_js="$(time_ms "$SELVR" build "$self_file" -o "$SCRATCH/out.js" --emit js)"
  selvr_bc="$(time_ms "$SELVR" build "$self_file" -o "$SCRATCH/out.vlxc" --emit bc)"

  if [[ "$has_tsc" -eq 1 ]]; then
    tsc_ms="$(time_ms tsc --noEmit --strict --target ES2022 "$ts_file")"
  else
    tsc_ms="not installed"
  fi

  if [[ "$has_esbuild" -eq 1 ]]; then
    esbuild_ms="$(time_ms esbuild --bundle --outfile="$SCRATCH/bundle.js" "$js_file")"
  else
    esbuild_ms="not installed"
  fi

  printf '%-10s  %-14s  %-14s  %-14s  %-14s\n' "$N" "$selvr_js" "$selvr_bc" "$tsc_ms" "$esbuild_ms"
done

echo ""
echo "selvr js: \`selvr build --emit js\` (parse and emit JavaScript)."
echo "selvr bc: \`selvr build --emit bc\` (parse, lower to IR, emit .vlxc)."
echo "tsc:      type-check only, when tsc is on PATH."
echo "esbuild:  bundle only, when esbuild is on PATH."
echo "Sources are generated N-function files in a temp directory, not a real program."
