//! Errors for [`Vault`]
//!
//! [`Vault`]: crate::prelude::Vault

use std::path::PathBuf;
use thiserror::Error;

/// Errors for [`Vault`]
///
/// [`Vault`]: crate::prelude::Vault
#[derive(Debug, Error)]
pub enum Error {
    /// Expected a directory path
    #[error("Path: `{0}` is not a directory")]
    IsNotDir(PathBuf),
}
