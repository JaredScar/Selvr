# Should-Fail Test Suite

These `.self` files are programs that the Selvr type checker **must reject**.
Each file contains:

- A `// ERROR:` comment describing what is wrong.
- An `// EXPECT:` comment naming the `TypeError` variant that must be produced.

## Running the suite

```bash
cargo test -p selvr-typechecker --test should_fail
```

Or with the CLI:

```bash
for f in tests/should_fail/*.self; do
    output=$(SELVR check "$f" 2>&1)
    expect=$(grep EXPECT "$f" | head -1 | sed 's/.*EXPECT: //')
    if echo "$output" | grep -q "$expect"; then
        echo "PASS $f"
    else
        echo "FAIL $f  (expected $expect)"
        echo "  got: $output"
    fi
done
```

## Test inventory

| File | Error expected |
|------|---------------|
| `01_type_mismatch.self` | `TypeMismatch` |
| `02_use_after_move.self` | `UseAfterMove` |
| `03_immutable_assign.self` | `ImmutableAssign` |
| `04_unresolved_name.self` | `UnresolvedName` |
| `05_arg_count_mismatch.self` | `ArgCountMismatch` |
| `06_missing_return.self` | `MissingReturn` |
| `07_await_outside_async.self` | `AwaitOutsideAsync` |
| `08_non_exhaustive_match.self` | `NonExhaustiveMatch` |
| `09_no_such_field.self` | `NoSuchField` |
| `10_not_callable.self` | `NotCallable` |
| `11_infinite_type.self` | `InfiniteType` |
| `12_double_move.self` | `UseAfterMove` |
| `13_return_type_mismatch.self` | `TypeMismatch` |
