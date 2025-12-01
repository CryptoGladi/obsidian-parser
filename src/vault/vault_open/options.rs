//! Options for [`VaultBuilder`] and [`Vault`]
//!
//! [`VaultBuilder`]: crate::vault::vault_open::VaultBuilder
//! [`Vault`]: crate::vault::Vault

use std::path::{Path, PathBuf};

/// Options for [`VaultBuilder`] and [`Vault`]
///
/// [`VaultBuilder`]: crate::vault::vault_open::VaultBuilder
/// [`Vault`]: crate::vault::Vault
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VaultOptions {
    /// Path to vault
    path: PathBuf,
}

impl VaultOptions {
    /// Create new [`VaultOptions`]
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Get path to vault
    #[inline]
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get into path to vault
    #[inline]
    #[must_use]
    pub fn into_path(self) -> PathBuf {
        self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    fn new() {
        let path = PathBuf::from("path/to/vault");
        let options = VaultOptions::new(&path);

        assert_eq!(options.path, path);
        assert_eq!(options.path(), path);
    }
}
