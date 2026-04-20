//! `selvr.lock` — deterministic lockfile for reproducible builds.
//!
//! The lockfile is a JSON file (easy to parse, easy to diff in Git).
//! It records the exact resolved version, registry URL, and SHA-256
//! checksum for every transitive dependency.
//!
//! # Format (selvr.lock)
//! ```json
//! {
//!   "version": 1,
//!   "packages": [
//!     {
//!       "name":     "selvr-std",
//!       "version":  "1.2.3",
//!       "registry": "https://pkg.selvr-lang.org",
//!       "checksum": "sha256:abc123...",
//!       "dependencies": ["some-dep@0.4.1"]
//!     }
//!   ]
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::PkgError;

/// The full lockfile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    pub version:  u32,
    pub packages: Vec<LockedPackage>,
}

/// A single locked package entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockedPackage {
    pub name:         String,
    pub version:      String,
    pub registry:     String,
    pub checksum:     String,
    #[serde(default)]
    pub dependencies: Vec<String>,   // "name@version" strings
}

impl Lockfile {
    /// Create a new empty lockfile.
    pub fn new() -> Self {
        Self { version: 1, packages: Vec::new() }
    }

    /// Read from `selvr.lock`.
    pub fn from_file(path: &Path) -> Result<Self, PkgError> {
        let src = std::fs::read_to_string(path).map_err(PkgError::Io)?;
        serde_json::from_str(&src).map_err(PkgError::Json)
    }

    /// Write to `selvr.lock`.
    pub fn to_file(&self, path: &Path) -> Result<(), PkgError> {
        let json = serde_json::to_string_pretty(self).map_err(PkgError::Json)?;
        std::fs::write(path, json).map_err(PkgError::Io)
    }

    /// Look up a locked package by name.
    pub fn get(&self, name: &str) -> Option<&LockedPackage> {
        self.packages.iter().find(|p| p.name == name)
    }

    /// Insert or update a locked package.
    pub fn upsert(&mut self, pkg: LockedPackage) {
        if let Some(existing) = self.packages.iter_mut().find(|p| p.name == pkg.name) {
            *existing = pkg;
        } else {
            self.packages.push(pkg);
        }
    }

    /// Remove a package by name.
    pub fn remove(&mut self, name: &str) {
        self.packages.retain(|p| p.name != name);
    }

    /// Sort packages alphabetically for deterministic diffs.
    pub fn sort(&mut self) {
        self.packages.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

impl Default for Lockfile {
    fn default() -> Self { Self::new() }
}
