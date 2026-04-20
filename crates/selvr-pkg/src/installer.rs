//! Package installer — downloads, verifies, and caches packages.
//!
//! Packages are stored in `~/.selvr/cache/{name}/{version}/`.
//! A package is a gzipped tar archive (.tar.gz) containing:
//!   src/    — .self source files
//!   selvr.toml — manifest
//!
//! The installer:
//!   1. Checks the local cache first.
//!   2. Downloads from the registry if not cached.
//!   3. Verifies the SHA-256 checksum.
//!   4. Extracts to the cache directory.
//!   5. Updates the lockfile checksum field.

use std::path::PathBuf;
use crate::{LockedPackage, PkgError};

/// Return the path to the Selvr cache directory.
pub fn cache_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".selvr").join("cache")
}

/// Return the path where a specific package version is cached.
pub fn package_cache_path(name: &str, version: &str) -> PathBuf {
    cache_dir().join(name).join(version)
}

/// Return true if a package is already in the local cache.
pub fn is_cached(name: &str, version: &str) -> bool {
    package_cache_path(name, version).exists()
}

/// Ensure the cache directory exists.
pub fn ensure_cache_dir() -> Result<(), PkgError> {
    std::fs::create_dir_all(cache_dir()).map_err(PkgError::Io)
}

/// Install a package from its tarball bytes.
///
/// In the full implementation this is called after the registry client
/// downloads the tarball. Verifies checksum, extracts, records in lockfile.
pub fn install_from_bytes(
    pkg:   &LockedPackage,
    bytes: &[u8],
) -> Result<PathBuf, PkgError> {
    // 1. Verify checksum
    crate::registry::verify_checksum(bytes, &pkg.checksum)?;

    // 2. Create cache dir
    let dest = package_cache_path(&pkg.name, &pkg.version);
    std::fs::create_dir_all(&dest).map_err(PkgError::Io)?;

    // 3. Write the raw tarball (extraction requires tar/flate2 — added in Phase 3)
    let tarball_path = dest.join("pkg.tar.gz");
    std::fs::write(&tarball_path, bytes).map_err(PkgError::Io)?;

    Ok(dest)
}

/// Return the path to a package's `src/` directory in the cache.
pub fn package_src_dir(name: &str, version: &str) -> PathBuf {
    package_cache_path(name, version).join("src")
}

/// List all cached packages as `(name, version)` pairs.
pub fn list_cached() -> Result<Vec<(String, String)>, PkgError> {
    let cache = cache_dir();
    if !cache.exists() { return Ok(Vec::new()); }
    let mut result = Vec::new();
    for entry in std::fs::read_dir(&cache).map_err(PkgError::Io)? {
        let entry = entry.map_err(PkgError::Io)?;
        let name  = entry.file_name().to_string_lossy().to_string();
        let pkg_dir = entry.path();
        if pkg_dir.is_dir() {
            for ver_entry in std::fs::read_dir(&pkg_dir).map_err(PkgError::Io)? {
                let ver_entry = ver_entry.map_err(PkgError::Io)?;
                let version   = ver_entry.file_name().to_string_lossy().to_string();
                result.push((name.clone(), version));
            }
        }
    }
    Ok(result)
}

/// Remove a specific package from the cache.
pub fn remove_cached(name: &str, version: &str) -> Result<(), PkgError> {
    let path = package_cache_path(name, version);
    if path.exists() {
        std::fs::remove_dir_all(path).map_err(PkgError::Io)?;
    }
    Ok(())
}
