//! Obsidian vault parsing and analysis
//!
//! Provides functionality for working with entire Obsidian vaults (collections of notes)
//!
//! # Performance Recommendations
//! **Prefer [`ObFileOnDisk`] over [`ObFileInMemory`] for large vaults** - it uses significantly less memory
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
//! let vault: VaultOnDisk<NoteProperties> = Vault::open("/path/to/vault").unwrap();
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
//! ## Use custom [`ObFile`] (example for [`ObFileInMemory`])
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
//! let vault: VaultInMemory<NoteProperties> = Vault::open("/path/to/vault").unwrap();
//! ```

pub mod error;
pub mod vault_duplicates;
pub mod vault_open;
//#[cfg(feature = "petgraph")]
//#[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
//pub mod vault_petgraph;

//#[cfg(test)]
//mod vault_test;

//pub(crate) mod vault_get_files;

use crate::obfile::DefaultProperties;
use crate::obfile::ObFile;
use crate::prelude::ObFileInMemory;
use crate::prelude::ObFileOnDisk;
use std::path::{Path, PathBuf};

/// Vault, but used [`ObFileOnDisk`]
pub type VaultOnDisk<T = DefaultProperties> = Vault<ObFileOnDisk<T>>;

/// Vault, but used [`ObFileInMemory`]
pub type VaultInMemory<T = DefaultProperties> = Vault<ObFileInMemory<T>>;

/// Represents an entire Obsidian vault
///
/// Contains all parsed notes and metadata about the vault. Uses [`ObFileOnDisk`] by default
/// which is optimized for memory efficiency in large vaults.
///
/// # Type Parameters
/// - `T`: Type for frontmatter properties
/// - `F`: File representation type
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Vault<F>
where
    F: ObFile,
{
    /// All notes in the vault
    notes: Vec<F>,

    /// Path to vault root directory
    path: PathBuf,
}

impl<F> Vault<F>
where
    F: ObFile,
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

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::vault_test::create_test_vault;

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn check_unique_note_name() {
        let (vault_path, _) = create_test_vault().unwrap();

        let vault = Vault::open_default(vault_path.path()).unwrap();
        assert!(!vault.check_unique_note_name());
    }
}
*/
