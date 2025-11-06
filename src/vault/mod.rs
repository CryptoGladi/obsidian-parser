//! Obsidian vault parsing and analysis
//!
//! Provides functionality for working with entire Obsidian vaults (collections of notes)
//!
//! # Performance Recommendations
//! **Prefer [`ObFileOnDisk`]) over [`ObFileInMemory`](crate::prelude::ObFileInMemory) for large vaults** - it uses significantly less memory
//! by reading files on-demand rather than loading everything into memory upfront.
//!
//! # Examples
//! ## Basic vault analysis
//! ```no_run
//! use obsidian_parser::prelude::*;
//!
//! // Open a vault using default properties (HashMap)
//! let vault = Vault::open_default("/path/to/vault").unwrap();
//!
//! // Check for duplicate note names
//! if vault.check_unique_note_name() {
//!     println!("All note names are unique");
//! } else {
//!     println!("Duplicate note names found!");
//! }
//!
//! // Access parsed files
//! for file in &vault.files {
//!     println!("Note: {:?}", file.path());
//! }
//! ```
//!
//! ## Using custom properties
//! ```no_run
//! use obsidian_parser::prelude::*;
//! use serde::Deserialize;
//!
//! #[derive(Clone, Deserialize)]
//! struct NoteProperties {
//!     created: String,
//!     tags: Vec<String>,
//!     priority: u8,
//! }
//!
//! let vault: Vault<NoteProperties> = Vault::open("/path/to/vault").unwrap();
//!
//! // Access custom properties
//! for file in &vault.files {
//!     let properties = file.properties().unwrap().unwrap();
//!
//!     println!(
//!         "Note created at {} with tags: {:?}",
//!         properties.created,
//!         properties.tags
//!     );
//! }
//! ```
//!
//! ## Building knowledge graphs (requires petgraph feature)
//! ```no_run
//! #[cfg(feature = "petgraph")]
//! {
//!     use obsidian_parser::prelude::*;
//!     use petgraph::dot::{Dot, Config};
//!
//!     let vault = Vault::open_default("/path/to/vault").unwrap();
//!     
//!     // Build directed graph
//!     let graph = vault.get_digraph();
//!     println!("Graph visualization:\n{:?}",
//!         Dot::with_config(&graph, &[Config::EdgeNoLabel])
//!     );
//!     
//!     // Analyze connectivity
//!     let components = petgraph::algo::connected_components(&graph);
//!     println!("Found {} connected components in knowledge base", components);
//! }
//! ```
//!
//! ## Use custom [`ObFile`] (example for [`ObFileInMemory`](crate::prelude::ObFileInMemory))
//! ```no_run
//! use obsidian_parser::prelude::*;
//! use serde::Deserialize;
//!
//! #[derive(Clone, Deserialize)]
//! struct NoteProperties {
//!     created: String,
//!     tags: Vec<String>,
//!     priority: u8,
//! }
//!
//! let vault: Vault<NoteProperties, ObFileInMemory<NoteProperties>> = Vault::open("/path/to/vault").unwrap();
//! ```

pub mod vault_open;
#[cfg(feature = "petgraph")]
#[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
pub mod vault_petgraph;

#[cfg(test)]
mod vault_test;

pub(crate) mod vault_get_files;

use crate::obfile::DefaultProperties;
use crate::obfile::ObFile;
use crate::{error::Error, prelude::ObFileOnDisk};
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::{marker::PhantomData, path::PathBuf};

/// Represents an entire Obsidian vault
///
/// Contains all parsed notes and metadata about the vault. Uses [`ObFileOnDisk`] by default
/// which is optimized for memory efficiency in large vaults.
///
/// # Type Parameters
/// - `T`: Type for frontmatter properties
/// - `F`: File representation type
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Vault<F = ObFileOnDisk<DefaultProperties>>
where
    F: ObFile + Send,
{
    /// All files in the vault
    pub files: Vec<F>,

    /// Path to vault root directory
    pub path: PathBuf,
}

impl<F> Vault<F>
where
    F: ObFile + Send,
{
    /// Returns duplicated note name
    ///
    /// # Performance
    /// Operates in O(n) time for large vaults
    ///
    /// # Other
    /// See [`check_unique_note_name`](Vault::check_unique_note_name)
    #[must_use]
    pub fn get_duplicates_notes(&self) -> Vec<String> {
        #[cfg(feature = "logging")]
        log::debug!(
            "Get duplicates notes in {} ({} files)",
            self.path.display(),
            self.files.len()
        );

        let mut seens_notes = HashSet::new();
        let mut duplicated_notes = Vec::new();

        #[allow(
            clippy::missing_panics_doc,
            clippy::unwrap_used,
            reason = "In any case, we will have a path to the files"
        )]
        for name_note in self.files.iter().map(|x| x.note_name().unwrap()) {
            if !seens_notes.insert(name_note.clone()) {
                #[cfg(feature = "logging")]
                log::trace!("Found duplicate: {name_note}");

                duplicated_notes.push(name_note);
            }
        }

        #[cfg(feature = "logging")]
        if !duplicated_notes.is_empty() {
            log::warn!("Found {} duplicate filenames", duplicated_notes.len());
        }

        duplicated_notes
    }

    /// Checks if all note filenames in the vault are unique
    ///
    /// # Returns
    /// `true` if all filenames are unique, `false` otherwise
    ///
    /// # Performance
    /// Operates in O(n) time for large vaults
    ///
    /// # Other
    /// See [`get_duplicates_notes`](Vault::get_duplicates_notes)
    #[must_use]
    pub fn check_unique_note_name(&self) -> bool {
        self.get_duplicates_notes().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_utils::init_test_logger, vault::vault_test::create_test_vault};

    #[test]
    fn check_unique_note_name() {
        init_test_logger();
        let (vault_path, _) = create_test_vault().unwrap();

        let vault = Vault::open_default(vault_path.path()).unwrap();
        assert!(!vault.check_unique_note_name());
    }
}
