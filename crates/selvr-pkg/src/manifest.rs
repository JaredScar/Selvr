//! Parsing for `selvr.toml` — the Selvr package manifest.
//!
//! # Format
//! ```toml
//! [package]
//! name    = "my-app"
//! version = "0.1.0"
//! authors = ["Alice <alice@example.com>"]
//! license = "MIT"
//! description = "A Selvr application"
//! homepage    = "https://example.com"
//! repository  = "https://github.com/example/my-app"
//!
//! [dependencies]
//! selvr-std   = "1.0.0"
//! some-lib    = "^0.3.1"
//! other-lib   = { version = "2.0", path = "../other-lib" }
//!
//! [dev-dependencies]
//! selvr-test  = "1.0.0"
//!
//! [targets]
//! # Override automatic WASM/JS targeting for specific functions.
//! # Equivalent to #[wasm] / #[js] attributes in source.
//! force_wasm = ["heavy_compute", "matmul"]
//! force_js   = ["render_frame", "on_click"]
//!
//! [build]
//! emit = "hybrid"           # "js" | "hybrid"
//! opt  = "release"          # "debug" | "release"
//! entry = "src/main.self"
//! ```

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use crate::PkgError;

/// The full parsed `selvr.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub package:          PackageMeta,
    #[serde(default)]
    pub dependencies:     HashMap<String, Dependency>,
    #[serde(rename = "dev-dependencies", default)]
    pub dev_dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub targets:          TargetOverrides,
    #[serde(default)]
    pub build:            BuildConfig,
}

/// `[package]` section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    pub name:        String,
    pub version:     String,
    #[serde(default)]
    pub authors:     Vec<String>,
    #[serde(default)]
    pub license:     Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub homepage:    Option<String>,
    #[serde(default)]
    pub repository:  Option<String>,
    #[serde(default)]
    pub keywords:    Vec<String>,
}

/// A single dependency — either a bare version string or a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// `some-lib = "^1.0.0"`
    Version(String),
    /// `some-lib = { version = "^1.0.0", path = "../some-lib" }`
    Table(DependencyTable),
}

impl Dependency {
    pub fn version_req(&self) -> &str {
        match self {
            Dependency::Version(v)        => v,
            Dependency::Table(t)          => &t.version,
        }
    }
    pub fn local_path(&self) -> Option<&str> {
        match self {
            Dependency::Table(t) => t.path.as_deref(),
            _                    => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTable {
    pub version:  String,
    #[serde(default)]
    pub path:     Option<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub features: Vec<String>,
}

/// `[targets]` — compile-time targeting overrides (supplement to #[wasm]/#[js]).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetOverrides {
    #[serde(default)]
    pub force_wasm: Vec<String>,
    #[serde(default)]
    pub force_js:   Vec<String>,
}

/// `[build]` — build configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_emit")]
    pub emit:  String,
    #[serde(default = "default_opt")]
    pub opt:   String,
    #[serde(default = "default_entry")]
    pub entry: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self { emit: default_emit(), opt: default_opt(), entry: default_entry() }
    }
}

fn default_emit()  -> String { "js".into() }
fn default_opt()   -> String { "debug".into() }
fn default_entry() -> String { "src/main.self".into() }

impl Manifest {
    /// Parse a `selvr.toml` file.
    pub fn from_file(path: &Path) -> Result<Self, PkgError> {
        let src = std::fs::read_to_string(path)
            .map_err(PkgError::Io)?;
        Self::from_str(&src)
    }

    /// Parse from a TOML string.
    pub fn from_str(src: &str) -> Result<Self, PkgError> {
        toml::from_str(src)
            .map_err(|e| PkgError::Manifest(e.to_string()))
    }

    /// Serialize the manifest back to TOML.
    pub fn to_toml(&self) -> Result<String, PkgError> {
        toml::to_string_pretty(self)
            .map_err(|e| PkgError::Manifest(e.to_string()))
    }

    /// Find `selvr.toml` by walking up from `start`.
    pub fn find_root(start: &Path) -> Option<std::path::PathBuf> {
        let mut dir = start.to_path_buf();
        loop {
            let candidate = dir.join("selvr.toml");
            if candidate.exists() { return Some(candidate); }
            if !dir.pop() { return None; }
        }
    }

    /// All direct dependencies (regular + dev).
    pub fn all_dependencies(&self) -> impl Iterator<Item = (&String, &Dependency)> {
        self.dependencies.iter().chain(self.dev_dependencies.iter())
    }
}
