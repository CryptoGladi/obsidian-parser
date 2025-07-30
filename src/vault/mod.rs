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
//! // Check for duplicate note names (important for graph operations)
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
//! #[derive(Clone, Default, Deserialize)]
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
//!     let properties = file.properties().unwrap();
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
//!     let graph = vault.get_digraph().unwrap();
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
//! #[derive(Clone, Default, Deserialize)]
//! struct NoteProperties {
//!     created: String,
//!     tags: Vec<String>,
//!     priority: u8,
//! }
//!
//! let vault: Vault<NoteProperties, ObFileInMemory<NoteProperties>> = Vault::open("/path/to/vault").unwrap();
//! ```

#[cfg(feature = "petgraph")]
pub mod vault_petgraph;

#[cfg(test)]
mod vault_test;

use crate::obfile::ObFile;
use crate::{error::Error, prelude::ObFileOnDisk};
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::{
    collections::HashMap,
    marker::PhantomData,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| s.starts_with('.'))
}

/// Represents an entire Obsidian vault
///
/// Contains all parsed notes and metadata about the vault. Uses [`ObFileOnDisk`] by default
/// which is optimized for memory efficiency in large vaults.
///
/// # Type Parameters
/// - `T`: Type for frontmatter properties
/// - `F`: File representation type
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Vault<T, F = ObFileOnDisk<T>>
where
    T: DeserializeOwned + Clone,
    F: ObFile<T> + Send,
{
    /// All files in the vault
    pub files: Vec<F>,

    /// Path to vault root directory
    pub path: PathBuf,

    /// Phantom data
    pub phantom: PhantomData<T>,
}

fn check_vault(path: impl AsRef<Path>) -> Result<(), Error> {
    let path_buf = path.as_ref().to_path_buf();

    if !path_buf.is_dir() {
        #[cfg(feature = "logging")]
        log::error!("Path is not directory: {}", path_buf.display());

        return Err(Error::IsNotDir(path_buf));
    }

    Ok(())
}

fn get_files_for_parse<T: FromIterator<DirEntry>>(path: impl AsRef<Path>) -> T {
    WalkDir::new(path)
        .min_depth(1)
        .into_iter()
        .filter_entry(|x| !is_hidden(x))
        .filter_map(Result::ok)
        .filter(|x| {
            x.path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        })
        .collect()
}

impl<T, F> Vault<T, F>
where
    T: DeserializeOwned + Clone,
    F: ObFile<T> + Send,
{
    #[cfg(feature = "rayon")]
    fn parse_files<L>(files: &[DirEntry], f: L) -> Vec<F>
    where
        L: Fn(&DirEntry) -> Option<F> + Sync + Send,
    {
        use rayon::prelude::*;

        files.into_par_iter().filter_map(f).collect()
    }

    #[cfg(not(feature = "rayon"))]
    fn parse_files<L>(files: &[DirEntry], f: L) -> Vec<F>
    where
        L: Fn(&DirEntry) -> Option<F>,
    {
        files.into_iter().filter_map(f).collect()
    }

    /// Opens and parses an Obsidian vault
    ///
    /// Recursively scans the directory for Markdown files (.md) and parses them.
    /// Uses [`ObFileOnDisk`] by default which is more memory efficient than [`ObFileInMemory`](crate::prelude::ObFileInMemory).
    ///
    /// # Arguments
    /// * `path` - Path to the vault directory
    ///
    /// # Errors
    /// Returns `Error` if:
    /// - Path doesn't exist or isn't a directory
    ///
    /// Files that fail parsing are skipped
    ///
    /// # Memory Considerations
    /// For vaults with 1000+ notes, prefer [`ObFileOnDisk`] (default) over [`ObFileInMemory`](crate::prelude::ObFileInMemory) as it:
    /// 1. Uses 90%+ less memory upfront
    /// 2. Only loads file content when accessed
    /// 3. Scales better for large knowledge bases
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::debug!("Opening vault at: {}", path_buf.display());

        check_vault(&path)?;
        let files_for_parse: Vec<_> = get_files_for_parse(&path);

        #[cfg(feature = "logging")]
        log::debug!("Found {} markdown files to parse", files_for_parse.len());

        #[allow(unused_variables)]
        let files = Self::parse_files(&files_for_parse, |file| match F::from_file(file.path()) {
            Ok(file) => Some(file),
            Err(e) => {
                #[cfg(feature = "logging")]
                log::warn!("Failed to parse {}: {}", file.path().display(), e);

                None
            }
        });

        #[cfg(feature = "logging")]
        log::info!("Parsed {} files", files.len());

        Ok(Self {
            files,
            path: path_buf,
            phantom: PhantomData,
        })
    }

    /// Returns duplicated note name
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
    /// **Critical for graph operations** where notes are identified by name.
    /// Always run this before calling [`get_digraph`](Vault::get_digraph) or [`get_ungraph`](Vault::get_ungraph).
    ///
    /// # Returns
    /// `true` if all filenames are unique, `false` otherwise
    ///
    /// # Performance
    /// Operates in O(n) time - safe for large vaults
    ///
    /// # Other
    /// See [`get_duplicates_notes`](Vault::get_duplicates_notes)
    #[must_use]
    pub fn check_unique_note_name(&self) -> bool {
        self.get_duplicates_notes().is_empty()
    }
}

#[allow(clippy::implicit_hasher)]
impl Vault<HashMap<String, serde_yml::Value>, ObFileOnDisk> {
    /// Opens vault using default properties ([`HashMap`]) and [`ObFileOnDisk`] storage
    ///
    /// Recommended for most use cases due to its memory efficiency
    ///
    /// # Errors
    /// Returns `Error` if:
    /// - Path doesn't exist or isn't a directory
    pub fn open_default<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::open(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_utils::init_test_logger, vault::vault_test::create_test_vault};
    use std::fs::File;

    #[test]
    fn open() {
        init_test_logger();
        let (vault_path, vault_files) = create_test_vault().unwrap();
        let vault = Vault::open_default(vault_path.path()).unwrap();

        assert_eq!(vault.files.len(), vault_files.len());
        assert_eq!(vault.path, vault_path.path());
    }

    #[test]
    #[should_panic]
    fn open_not_dir() {
        init_test_logger();
        let (vault_path, _) = create_test_vault().unwrap();
        let path_to_file = vault_path.path().join("main.md");
        assert!(path_to_file.is_file());

        let _ = Vault::open_default(&path_to_file).unwrap();
    }

    #[test]
    fn open_with_extra_files() {
        init_test_logger();
        let (vault_path, vault_files) = create_test_vault().unwrap();
        File::create(vault_path.path().join("extra_file.not_md")).unwrap();

        let vault = Vault::open_default(vault_path.path()).unwrap();

        assert_eq!(vault.files.len(), vault_files.len());
        assert_eq!(vault.path, vault_path.path());
    }

    #[test]
    fn check_unique_note_name() {
        init_test_logger();
        let (vault_path, _) = create_test_vault().unwrap();

        let mut vault = Vault::open_default(vault_path.path()).unwrap();
        assert!(vault.check_unique_note_name());

        vault.files.push(vault.files.first().unwrap().clone());
        assert!(!vault.check_unique_note_name());
    }
}
