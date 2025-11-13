//! Module for open impl [`Vault`]

use super::Vault;
use crate::{
    obfile::{ObFile, ObFileRead},
    prelude::ObFileOnDisk,
};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VaultOptions {
    path: PathBuf,
}

impl VaultOptions {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FilesBuilder {
    options: VaultOptions,
    include_hidden: bool,
}

fn is_hidden(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .file_name()
        .is_some_and(|e| e.to_str().is_some_and(|name| name.starts_with('.')))
}

fn is_md_file(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .is_some_and(|p| p.eq_ignore_ascii_case("md"))
}

impl FilesBuilder {
    pub fn new(options: VaultOptions) -> Self {
        Self {
            options,
            include_hidden: false,
        }
    }

    pub fn include_hidden(mut self, include_hidden: bool) -> Self {
        self.include_hidden = include_hidden;
        self
    }

    fn ignored_hidden_files(include_hidden: bool, entry: &DirEntry) -> bool {
        if !include_hidden && is_hidden(entry.path()) {
            return false;
        }

        return true;
    }

    pub fn into_iter<F>(self) -> impl Iterator<Item = Result<F, F::Error>>
    where
        F: ObFileRead,
        F::Properties: DeserializeOwned,
        F::Error: From<std::io::Error>,
    {
        let include_hidden = self.include_hidden;

        let files = WalkDir::new(&self.options.path)
            .into_iter()
            .filter_entry(move |entry| {
                entry.depth() == 0 || Self::ignored_hidden_files(include_hidden, entry)
            })
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .filter(|path| is_md_file(path));

        files.map(|path| F::from_file(path))
    }

    #[cfg(feature = "rayon")]
    pub fn into_par_iter<F>(self) -> impl rayon::iter::ParallelIterator<Item = Result<F, F::Error>>
    where
        F: ObFileRead + Send,
        F::Properties: DeserializeOwned,
        F::Error: From<std::io::Error> + Send,
    {
        use rayon::prelude::*;
        let include_hidden = self.include_hidden;

        let files = WalkDir::new(&self.options.path)
            .into_iter()
            .filter_entry(move |entry| {
                entry.depth() == 0 || Self::ignored_hidden_files(include_hidden, entry)
            })
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .filter(|path| is_md_file(path));

        files.map(|path| F::from_file(path)).par_bridge()
    }
}

pub trait IteratorFilesBuilder<F = ObFileOnDisk>: Iterator<Item = F>
where
    Self: Sized,
    F: ObFile,
{
    fn build_vault(self, options: VaultOptions) -> Vault<F> {
        Vault {
            notes: self.collect::<Vec<F>>(),
            path: options.path,
        }
    }
}

impl<F, I> IteratorFilesBuilder<F> for I
where
    F: ObFile,
    I: Iterator<Item = F>,
{
}

#[cfg(feature = "rayon")]
pub trait ParallelIteratorFilesBuilder<F = ObFileOnDisk>:
    rayon::iter::ParallelIterator<Item = F>
where
    F: ObFile + Send,
{
    fn build_vault(self, options: VaultOptions) -> Vault<F> {
        Vault {
            notes: self.collect::<Vec<F>>(),
            path: options.path,
        }
    }
}

#[cfg(feature = "rayon")]
impl<F, I> ParallelIteratorFilesBuilder<F> for I
where
    F: ObFile + Send,
    I: rayon::iter::ParallelIterator<Item = F>,
{
}

/*
impl<F> Vault<F>
where
    F: ObFileRead,
    F::Properties: DeserializeOwned,
    F::Error: From<std::io::Error>,
{
    pub fn open(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();

        let files = VaultBuilder::new(&path)
            .include_hidden(false)
            .into_iter()
            .filter_map(Result::ok)
            .build_vault();

        Self { files, path }
    }
}

#[allow(clippy::implicit_hasher)]
impl Vault<ObFileOnDisk<DefaultProperties>> {
    /// Opens vault using default properties ([`HashMap`](std::collections::HashMap)) and [`ObFileOnDisk`] storage
    ///
    /// Recommended for most use cases due to its memory efficiency
    ///
    /// # Example
    /// ```no_run
    /// use obsidian_parser::prelude::*;
    ///
    /// // Open a vault using default properties (HashMap)
    /// let vault = Vault::open_default("/path/to/vault").unwrap();
    /// ```
    /// # Errors
    /// Returns `Error` if:
    /// - Path doesn't exist or isn't a directory
    /// - File parse is not correct
    pub fn open_default<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::open(path)
    }

    /// Opens vault using default properties ([`HashMap`](std::collections::HashMap)) and [`ObFileOnDisk`] storage
    ///
    /// Recommended for most use cases due to its memory efficiency
    ///
    /// # Example
    /// ```no_run
    /// use obsidian_parser::prelude::*;
    ///
    /// // Open a vault using default properties (HashMap)
    /// let vault = Vault::open_ignore_default("/path/to/vault").unwrap();
    /// ```
    /// # Errors
    /// Returns `Error` if:
    /// - Path doesn't exist or isn't a directory
    pub fn open_ignore_default<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::open_ignore(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::ObFileInMemory;
    use crate::{test_utils::init_test_logger, vault::vault_test::create_test_vault};
    use serde::{Deserialize, Serialize};
    use std::fs::File;
    use std::io::Write;

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn open() {
        init_test_logger();
        let (vault_path, vault_files) = create_test_vault().unwrap();
        let vault = Vault::open_default(vault_path.path()).unwrap();

        assert_eq!(vault.files.len(), vault_files.len());
        assert_eq!(vault.path, vault_path.path());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[should_panic]
    fn open_not_dir() {
        init_test_logger();
        let (vault_path, _) = create_test_vault().unwrap();
        let path_to_file = vault_path.path().join("main.md");
        assert!(path_to_file.is_file());

        let _ = Vault::open_default(&path_to_file).unwrap();
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn open_with_extra_files() {
        init_test_logger();
        let (vault_path, vault_files) = create_test_vault().unwrap();
        File::create(vault_path.path().join("extra_file.not_md")).unwrap();

        let vault = Vault::open_default(vault_path.path()).unwrap();

        assert_eq!(vault.files.len(), vault_files.len());
        assert_eq!(vault.path, vault_path.path());
    }

    #[derive(Clone, Deserialize, Serialize)]
    pub struct TestProperties {
        #[allow(dead_code)]
        not_correct: String,
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn open_with_error() {
        init_test_logger();

        let (vault_path, _) = create_test_vault().unwrap();
        let mut file = File::create(vault_path.path().join("not_file.md")).unwrap();
        file.write_all(b"---\nnot: \n---\ndata").unwrap(); // Not UTF-8

        let error_open = Vault::<ObFileInMemory<TestProperties>>::open(vault_path.path())
            .err()
            .unwrap();

        assert!(matches!(error_open, Error::Yaml(_)));
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn open_with_error_but_ignored() {
        init_test_logger();

        let (vault_path, _) = create_test_vault().unwrap();
        let mut file = File::create(vault_path.path().join("not_file.md")).unwrap();
        file.write_all(b"---\nnot: \n---\ndata").unwrap(); // Not UTF-8

        let error_open =
            Vault::<ObFileInMemory<TestProperties>>::open_ignore(vault_path.path()).err();

        assert!(matches!(error_open, None));
    }
}
*/
