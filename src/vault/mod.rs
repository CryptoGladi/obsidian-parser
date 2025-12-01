//! Obsidian vault parsing and analysis
//!
//! Provides functionality for working with entire Obsidian vaults (collections of notes)
//!
//! # Performance Recommendations
//! **Prefer [`NoteOnDisk`] over [`NoteInMemory`] for large vaults** - it uses significantly less memory
//! by reading files on-demand rather than loading everything into memory upfront.

pub mod error;
pub mod vault_duplicates;
pub mod vault_open;

#[cfg(feature = "petgraph")]
#[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
pub mod vault_petgraph;

#[cfg(test)]
mod vault_test;

use crate::note::DefaultProperties;
use crate::note::Note;
use crate::prelude::{NoteInMemory, NoteOnDisk, NoteOnceCell, NoteOnceLock};
use std::path::{Path, PathBuf};

/// Vault, but used [`NoteOnDisk`]
pub type VaultOnDisk<T = DefaultProperties> = Vault<NoteOnDisk<T>>;

/// Vault, but used [`NoteOnceCell`]
pub type VaultOnceCell<T = DefaultProperties> = Vault<NoteOnceCell<T>>;

/// Vault, but used [`NoteOnceLock`]
pub type VaultOnceLock<T = DefaultProperties> = Vault<NoteOnceLock<T>>;

/// Vault, but used [`NoteInMemory`]
pub type VaultInMemory<T = DefaultProperties> = Vault<NoteInMemory<T>>;

/// Represents an entire Obsidian vault
///
/// Contains all parsed notes and metadata about the vault. Uses [`NoteOnDisk`] by default
/// which is optimized for memory efficiency in large vaults.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Vault<N = NoteInMemory>
where
    N: Note,
{
    /// All notes in the vault
    notes: Vec<N>,

    /// Path to vault root directory
    path: PathBuf,
}

impl<N> Vault<N>
where
    N: Note,
{
    /// Get notes
    #[must_use]
    #[inline]
    pub const fn notes(&self) -> &Vec<N> {
        &self.notes
    }

    /// Get mutables notes
    #[must_use]
    #[inline]
    pub const fn mut_notes(&mut self) -> &mut Vec<N> {
        &mut self.notes
    }

    /// Get count in notes from vault
    #[must_use]
    #[inline]
    pub const fn count_notes(&self) -> usize {
        self.notes().len()
    }

    /// Get path to vault
    #[must_use]
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        prelude::{IteratorVaultBuilder, VaultBuilder, VaultOptions},
        vault::vault_test::create_files_for_vault,
    };

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    fn notes() {
        let (path, files) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let vault: VaultInMemory = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options);

        assert_eq!(vault.notes().len(), files.len());
    }

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    fn count_notes() {
        let (path, files) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let vault: VaultInMemory = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options);

        assert_eq!(vault.count_notes(), files.len());
    }

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    fn path() {
        let (path, _) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let vault: VaultInMemory = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options);

        assert_eq!(vault.path(), path.path());
    }
}
