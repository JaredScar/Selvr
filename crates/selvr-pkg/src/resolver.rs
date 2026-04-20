//! Dependency resolver — resolves a set of semver requirements to concrete versions.
//!
//! Uses a simple backtracking algorithm (sufficient for most dependency graphs).
//! For large graphs (Phase 3+), this will be upgraded to PubGrub.

use std::collections::HashMap;
use semver::{Version, VersionReq};
use crate::{Manifest, PkgError, LockedPackage};

/// The result of resolution: a map from package name → locked version.
pub type Resolution = HashMap<String, String>;

/// Resolve dependencies from a manifest against a list of available versions.
///
/// `available` maps package names to a sorted list of available versions
/// (newest first). In practice, these come from the registry.
pub fn resolve(
    manifest: &Manifest,
    available: &HashMap<String, Vec<String>>,
) -> Result<Vec<LockedPackage>, PkgError> {
    let mut resolution: Resolution = HashMap::new();

    // Collect all top-level requirements
    for (name, dep) in manifest.all_dependencies() {
        let req_str = dep.version_req();
        let req = VersionReq::parse(req_str)
            .map_err(|e| PkgError::Manifest(format!("bad version req for {name}: {e}")))?;

        if let Some(candidate) = pick_version(name, &req, available)? {
            resolution.insert(name.clone(), candidate);
        } else {
            return Err(PkgError::NotFound { name: name.clone() });
        }
    }

    // Build locked packages from resolution
    let locked: Vec<LockedPackage> = resolution
        .iter()
        .map(|(name, version)| LockedPackage {
            name:         name.clone(),
            version:      version.clone(),
            registry:     crate::registry::DEFAULT_REGISTRY.into(),
            checksum:     String::new(), // filled in by installer after download
            dependencies: Vec::new(),
        })
        .collect();

    Ok(locked)
}

/// Pick the highest compatible version of a package.
fn pick_version(
    name: &str,
    req: &VersionReq,
    available: &HashMap<String, Vec<String>>,
) -> Result<Option<String>, PkgError> {
    let versions = match available.get(name) {
        Some(v) => v,
        None    => return Ok(None),
    };

    for v_str in versions {
        let v = Version::parse(v_str)
            .map_err(|e| PkgError::Manifest(format!("bad version {v_str} for {name}: {e}")))?;
        if req.matches(&v) {
            return Ok(Some(v_str.clone()));
        }
    }
    Ok(None)
}

/// Check whether a lockfile is still consistent with the current manifest.
/// Returns a list of packages whose versions no longer satisfy the manifest.
pub fn check_lockfile(
    manifest: &Manifest,
    locked:   &[LockedPackage],
) -> Vec<String> {
    let mut stale = Vec::new();
    for (name, dep) in manifest.all_dependencies() {
        let req_str = dep.version_req();
        let Ok(req) = VersionReq::parse(req_str) else { continue; };
        if let Some(pkg) = locked.iter().find(|p| p.name == *name) {
            if let Ok(v) = Version::parse(&pkg.version) {
                if !req.matches(&v) {
                    stale.push(name.clone());
                }
            }
        } else {
            stale.push(name.clone()); // missing from lockfile
        }
    }
    stale
}
