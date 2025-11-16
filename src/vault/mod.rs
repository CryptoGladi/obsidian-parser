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
use crate::note::note_once_cell::NoteOnceCell;
use crate::prelude::NoteInMemory;
use crate::prelude::NoteOnDisk;
use std::path::{Path, PathBuf};

/// Vault, but used [`NoteOnDisk`]
pub type VaultOnDisk<T = DefaultProperties> = Vault<NoteOnDisk<T>>;

/// Vault, but used [`NoteOnceCell`]
pub type VaultOnceCell<T = DefaultProperties> = Vault<NoteOnceCell<T>>;

/// Vault, but used [`NoteInMemory`]
pub type VaultInMemory<T = DefaultProperties> = Vault<NoteInMemory<T>>;

/// Represents an entire Obsidian vault
///
/// Contains all parsed notes and metadata about the vault. Uses [`NoteOnDisk`] by default
/// which is optimized for memory efficiency in large vaults.
///
/// # Type Parameters
/// - `T`: Type for frontmatter properties
/// - `F`: File representation type
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Vault<F = NoteInMemory>
where
    F: Note,
{
    /// All notes in the vault
    notes: Vec<F>,

    /// Path to vault root directory
    path: PathBuf,
}

impl<F> Vault<F>
where
    F: Note,
{
    #[must_use]
    pub const fn notes(&self) -> &Vec<F> {
        &self.notes
    }

    #[must_use]
    pub const fn count_notes(&self) -> usize {
        self.notes().len()
    }

    #[must_use]
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

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn notes() {
        let (path, files) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let vault: VaultInMemory = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options)
            .unwrap();

        assert_eq!(vault.notes().len(), files.len());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn count_notes() {
        let (path, files) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let vault: VaultInMemory = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options)
            .unwrap();

        assert_eq!(vault.count_notes(), files.len());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn path() {
        let (path, _) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let vault: VaultInMemory = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options)
            .unwrap();

        assert_eq!(vault.path(), path.path());
    }
}
