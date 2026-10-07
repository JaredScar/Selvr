#!/usr/bin/env bash
# Build the release CLI and run the runtime harness.
# From anywhere: ./docs/benchmarks/run.sh

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
exec cargo run -p selvr-cli --release -- bench
