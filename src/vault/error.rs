use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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
}
