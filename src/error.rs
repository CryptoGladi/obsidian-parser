//! Error handling for Obsidian vault parsing operations

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for Obsidian parsing operations
#[derive(Debug, Error)]
pub enum Error {
    /// I/O operation failed (file reading, directory traversal, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid frontmatter format detected
    ///
    /// Occurs when:
    /// - Frontmatter delimiters are incomplete (`---` missing)
    /// - Content between delimiters is empty
    ///
    /// # Example
    /// Parsing a file with malformed frontmatter:
    /// ```text
    /// ---
    /// incomplete yaml
    /// // Missing closing ---
    /// ```
    #[error("Invalid frontmatter format")]
    InvalidFormat,

    /// YAML parsing error in frontmatter properties
    ///
    /// # Example
    /// Parsing invalid YAML syntax:
    /// ```text
    /// ---
    /// key: @invalid_value
    /// ---
    /// ```
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yml::Error),

    /// Expected a directory path
    ///
    /// # Example
    /// ```no_run
    /// use obsidian_parser::prelude::*;
    ///
    /// // Will fail if passed a file path
    /// Vault::open_default("notes.md").unwrap();
    /// ```
    #[error("Path: `{0}` is not a directory")]
    IsNotDir(PathBuf),

    /// Expected a file path
    ///
    /// # Example
    /// ```no_run
    /// use obsidian_parser::prelude::*;
    ///
    /// // Will fail if passed a directory path
    /// ObFileOnDisk::from_file_default("/home/test");
    /// ```
    #[error("Path: `{0}` is not a directory")]
    IsNotFile(PathBuf),
}
