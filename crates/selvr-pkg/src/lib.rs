//! selvr-pkg — package manager core
//!
//! Provides:
//! - [`Manifest`]  — parsed `selvr.toml`
//! - [`Lockfile`]  — parsed/generated `selvr.lock`
//! - [`Registry`]  — HTTP client for the package registry
//! - [`Resolver`]  — semver dependency resolver
//! - [`Installer`] — downloads and caches packages

pub mod manifest;
pub mod lockfile;
pub mod registry;
pub mod resolver;
pub mod installer;
pub mod error;

pub use manifest::{Manifest, Dependency, PackageMeta};
pub use lockfile::{Lockfile, LockedPackage};
pub use error::PkgError;
