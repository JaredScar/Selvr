//! Registry client — communicates with the Selvr package registry.
//!
//! The registry API follows a simple REST design:
//!
//! ```
//! GET  /v1/packages/{name}                → PackageInfo (all versions)
//! GET  /v1/packages/{name}/{version}      → VersionInfo (metadata + download URL)
//! GET  /v1/packages/{name}/{version}/dl   → binary tarball (.tar.gz)
//! POST /v1/packages/publish               → publish a new version (auth required)
//! GET  /v1/search?q={query}&limit={n}     → SearchResult[]
//! ```

use serde::{Deserialize, Serialize};
use crate::PkgError;

pub const DEFAULT_REGISTRY: &str = "https://pkg.selvr-lang.org";

/// Summary information about a package from the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name:        String,
    pub description: Option<String>,
    pub versions:    Vec<String>,
    pub latest:      String,
    pub downloads:   u64,
}

/// Full metadata for a specific package version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub name:         String,
    pub version:      String,
    pub description:  Option<String>,
    pub authors:      Vec<String>,
    pub license:      Option<String>,
    pub dependencies: Vec<RegistryDep>,
    pub checksum:     String,         // sha256 hex of the tarball
    pub download_url: String,
    pub yanked:       bool,
}

/// A dependency as represented in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryDep {
    pub name:    String,
    pub req:     String,   // semver requirement, e.g. "^1.0.0"
    pub dev:     bool,
}

/// A single search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub name:        String,
    pub version:     String,
    pub description: Option<String>,
    pub downloads:   u64,
}

/// Registry client.  Uses `reqwest` in the CLI binary; this module provides
/// a trait-based interface so it can be mocked in tests.
pub struct Registry {
    pub base_url: String,
    pub token:    Option<String>,
}

impl Registry {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into(), token: None }
    }

    pub fn default() -> Self {
        Self::new(DEFAULT_REGISTRY)
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Build the URL for a package info request.
    pub fn package_url(&self, name: &str) -> String {
        format!("{}/v1/packages/{}", self.base_url, name)
    }

    /// Build the URL for a specific version.
    pub fn version_url(&self, name: &str, version: &str) -> String {
        format!("{}/v1/packages/{}/{}", self.base_url, name, version)
    }

    /// Build the download URL for a tarball.
    pub fn download_url(&self, name: &str, version: &str) -> String {
        format!("{}/v1/packages/{}/{}/dl", self.base_url, name, version)
    }

    /// Build the search URL.
    pub fn search_url(&self, query: &str, limit: usize) -> String {
        format!("{}/v1/search?q={}&limit={}", self.base_url, query, limit)
    }
}

/// Verify a downloaded tarball matches its declared SHA-256 checksum.
pub fn verify_checksum(bytes: &[u8], expected: &str) -> Result<(), PkgError> {
    use sha2::{Sha256, Digest};
    let hash   = Sha256::digest(bytes);
    let actual = format!("sha256:{}", hex::encode(hash));
    if actual != expected {
        return Err(PkgError::ChecksumMismatch {
            name:     "package".into(),
            expected: expected.into(),
            actual,
        });
    }
    Ok(())
}
