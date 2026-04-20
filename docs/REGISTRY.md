# Selvr Package Registry

The Selvr package registry (`pkg.selvr-lang.org`) hosts community and official packages
for the Selvr language. It is inspired by crates.io and follows the same core design
principles: content-addressed packages, immutable versions, and a simple REST API.

---

## URL structure

```
https://pkg.selvr-lang.org/v1/
  packages/                       — package listing
  packages/{name}                 — package info (all versions)
  packages/{name}/{version}       — version metadata
  packages/{name}/{version}/dl    — tarball download
  search?q={query}&limit={n}      — full-text search
  me                              — authenticated user info
```

---

## Package format

Each published package is a gzipped tar archive (`.tar.gz`) containing:

```
selvr.toml          — manifest (required)
src/                — Selvr source files (*.self)
  main.self         — or any other entry point
  ...
README.md           — optional
LICENSE             — optional
```

The tarball **must not** include:
- `selvr.lock` (regenerated on install)
- Build artefacts (`*.js`, `*.wasm`)
- `node_modules/` or similar dependency caches

---

## selvr.lock format

`selvr.lock` is a JSON file. It is committed to version control so that
`selvr install --frozen` gives every developer (and CI) identical package versions.

```json
{
  "version": 1,
  "packages": [
    {
      "name":         "selvr-std",
      "version":      "1.2.3",
      "registry":     "https://pkg.selvr-lang.org",
      "checksum":     "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "dependencies": ["selvr-unicode@1.0.1"]
    },
    {
      "name":         "selvr-unicode",
      "version":      "1.0.1",
      "registry":     "https://pkg.selvr-lang.org",
      "checksum":     "sha256:abc123...",
      "dependencies": []
    }
  ]
}
```

### Fields

| Field          | Type       | Description                                             |
|----------------|------------|---------------------------------------------------------|
| `version`      | `u32`      | Lockfile format version (currently `1`)                 |
| `name`         | `string`   | Package name (matches `selvr.toml [package].name`)      |
| `version`      | `string`   | Exact resolved semver version                           |
| `registry`     | `string`   | Registry base URL (for multi-registry workspaces)       |
| `checksum`     | `string`   | `sha256:` hex of the downloaded tarball                 |
| `dependencies` | `string[]` | Direct deps of this package as `"name@version"` strings |

---

## Semver requirements

Selvr follows [semantic versioning 2.0.0](https://semver.org). Version requirements
in `selvr.toml` use the same syntax as Cargo:

| Syntax      | Meaning                             |
|-------------|-------------------------------------|
| `"1.2.3"`   | Exactly `1.2.3`                     |
| `"^1.2.3"`  | `>=1.2.3 <2.0.0` (caret — default) |
| `"~1.2.3"`  | `>=1.2.3 <1.3.0` (tilde)           |
| `">=1.0"`   | At least `1.0.0`                    |
| `"*"`       | Any version (latest stable)         |

---

## Authentication

To publish packages you need a token from `pkg.selvr-lang.org`.

```bash
# Set token as environment variable (recommended for CI)
export SELVR_TOKEN="selvrpkg_..."

# Or pass inline
selvr publish --token selvrpkg_...
```

Tokens are scoped to your user account and can be revoked at any time from
the registry web UI.

---

## Publish checklist

Before running `selvr publish`:

1. **Version bump** — update `[package].version` in `selvr.toml`
2. **CHANGELOG** — document what changed
3. **Tests** — run `selvr test` and confirm all tests pass
4. **Dry run** — `selvr publish --dry-run` to preview metadata
5. **Publish** — `selvr publish`

Versions are **immutable** once published. To fix a broken release, publish a new
patch version. Yanking a version removes it from `selvr install` resolution but
keeps it downloadable for existing lockfiles.

---

## Workspaces

A workspace is a directory containing multiple Selvr packages, each with its own
`selvr.toml`, sharing a single root `selvr.lock`.

```toml
# workspace/selvr.toml  (root)
[workspace]
members = [
  "packages/core",
  "packages/dom",
  "packages/net",
]
```

Run `selvr install` from the workspace root to install dependencies for all members.
The resolver produces a **unified** lockfile that satisfies all members simultaneously.

---

## Registry API reference

### `GET /v1/packages/{name}`

Returns info about a package (all versions).

```json
{
  "name":        "selvr-std",
  "description": "The official Selvr standard library",
  "versions":    ["1.0.0", "1.1.0", "1.2.3"],
  "latest":      "1.2.3",
  "downloads":   142381
}
```

### `GET /v1/packages/{name}/{version}`

Returns full metadata for a specific version.

```json
{
  "name":         "selvr-std",
  "version":      "1.2.3",
  "description":  "The official Selvr standard library",
  "authors":      ["The Selvr Authors"],
  "license":      "MIT OR Apache-2.0",
  "dependencies": [{ "name": "selvr-unicode", "req": "^1.0.0", "dev": false }],
  "checksum":     "sha256:...",
  "download_url": "https://pkg.selvr-lang.org/v1/packages/selvr-std/1.2.3/dl",
  "yanked":       false
}
```

### `POST /v1/packages/publish`

Publishes a new package version. Requires `Authorization: Bearer <token>` header.
Body: multipart form with `manifest` (JSON-encoded `selvr.toml`) and `tarball` (`.tar.gz` binary).

### `GET /v1/search?q={query}&limit={n}`

Returns up to `limit` (default 10, max 100) matching packages:

```json
[
  { "name": "selvr-std",   "version": "1.2.3", "description": "...", "downloads": 142381 },
  { "name": "selvr-test",  "version": "1.0.0", "description": "...", "downloads":  38200 }
]
```

---

## Official packages

| Package           | Version | Description                              |
|-------------------|---------|------------------------------------------|
| `selvr-std`       | `1.0.0` | Core standard library (Option, Result, Iter, Array, String, …) |
| `selvr-test`      | `1.0.0` | Built-in test runner (`#[test]`, assertions) |
| `selvr-dom`       | `0.1.0` | DOM and Web API wrappers                 |
| `selvr-net`       | `0.1.0` | Fetch and WebSocket client               |
| `selvr-time`      | `0.1.0` | Date and time utilities                  |
| `selvr-wgpu`      | `0.1.0` | WebGPU compute bindings (Phase 3 stretch goal) |
