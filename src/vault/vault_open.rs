//! Module for open impl [`Vault`]

use super::{DefaultProperties, Error, Vault};
use crate::{
    obfile::{ObFile, ObFileRead},
    prelude::ObFileOnDisk,
    vault::vault_get_files::get_files_for_parse,
};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

fn check_vault(path: impl AsRef<Path>) -> Result<(), Error> {
    let path_buf = path.as_ref().to_path_buf();

    if !path_buf.is_dir() {
        #[cfg(feature = "logging")]
        log::error!("Path is not directory: {}", path_buf.display());

        return Err(Error::IsNotDir(path_buf));
    }

    Ok(())
}

#[derive(Debug)]
pub struct VaultBuilder {
    path: PathBuf,
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

impl VaultBuilder {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
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

    pub fn into_iter<F>(self) -> impl Iterator<Item = Result<F, Error>>
    where
        F: ObFileRead,
        F::Properties: DeserializeOwned,
    {
        let include_hidden = self.include_hidden;

        let files = WalkDir::new(self.path)
            .into_iter()
            .filter_entry(move |e| e.depth() == 0 || Self::ignored_hidden_files(include_hidden, e))
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .filter(|path| is_md_file(path));

        files.map(|path| F::from_file(path))
    }
}

pub trait IteratorVaultBulder<F>: Iterator<Item = F>
where
    Self: Sized,
    F: ObFile,
{
    fn build_vault(self) -> Vec<F> {
        Vec::from_iter(self)
    }
}

impl<F, I> IteratorVaultBulder<F> for I
where
    F: ObFile,
    I: Iterator<Item = F>,
{
}

impl<F> Vault<F>
where
    F::Properties: DeserializeOwned,
    F: ObFileRead + Send,
{
    /// Parsing files by rayon with ignore
    #[cfg(feature = "rayon")]
    fn parse_files_with_ignore<L>(files: &[PathBuf], f: L) -> Vec<F>
    where
        L: Fn(&PathBuf) -> Result<F, Error> + Sync + Send,
    {
        use rayon::prelude::*;

        files
            .into_par_iter()
            .filter_map(|file| f(file).ok())
            .collect()
    }

    /// Parsing files withut rayon
    #[cfg(not(feature = "rayon"))]
    fn parse_files_with_ignore<L>(files: &[PathBuf], f: L) -> Vec<F>
    where
        L: Fn(&PathBuf) -> Result<F, Error>,
    {
        files.into_iter().filter_map(|file| f(file).ok()).collect()
    }

    /// Parsing files by rayon
    #[cfg(feature = "rayon")]
    fn parse_files<L>(files: &[PathBuf], f: L) -> Result<Vec<F>, Error>
    where
        L: Fn(&PathBuf) -> Result<F, Error> + Sync + Send,
    {
        use rayon::prelude::*;

        files
            .into_par_iter()
            .map(f)
            .try_fold(
                || Vec::new(),
                |mut acc, result| {
                    acc.push(result?);
                    Ok(acc)
                },
            )
            .try_reduce(
                || Vec::new(),
                |mut a, mut b| {
                    a.append(&mut b);
                    Ok(a)
                },
            )
    }

    /// Parsing files withut rayon
    #[cfg(not(feature = "rayon"))]
    fn parse_files<L>(files: &[PathBuf], f: L) -> Result<Vec<F>, Error>
    where
        L: Fn(&PathBuf) -> Result<F, Error> + Sync + Send,
    {
        files
            .into_iter()
            .map(|file| f(file))
            .try_fold(Vec::new(), |mut acc, result| {
                acc.push(result?);
                Ok::<Vec<F>, Error>(acc)
            })
    }

    /// Opens and parses an Obsidian vault
    ///
    /// Recursively scans the directory for Markdown files (.md) and parses them.
    /// Uses [`ObFileOnDisk`] by default which is more memory efficient than [`ObFileInMemory`](crate::prelude::ObFileInMemory).
    ///
    /// # Arguments
    /// * `path` - Path to the vault directory
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
        #[allow(clippy::manual_ok_err)]
        let files = Self::parse_files(&files_for_parse, |file| F::from_file(file))?;

        #[cfg(feature = "logging")]
        log::info!("Parsed {} files", files.len());

        Ok(Self {
            files,
            path: path_buf,
        })
    }

    /// Opens and parses an Obsidian vault but **ignored errors**
    ///
    /// Recursively scans the directory for Markdown files (.md) and parses them.
    /// Uses [`ObFileOnDisk`] by default which is more memory efficient than [`ObFileInMemory`](crate::prelude::ObFileInMemory).
    ///
    /// # Arguments
    /// * `path` - Path to the vault directory
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
    ///
    /// # Memory Considerations
    /// For vaults with 1000+ notes, prefer [`ObFileOnDisk`] (default) over [`ObFileInMemory`](crate::prelude::ObFileInMemory) as it:
    /// 1. Uses 90%+ less memory upfront
    /// 2. Only loads file content when accessed
    /// 3. Scales better for large knowledge bases
    pub fn open_ignore<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::debug!("Opening ignored vault at: {}", path_buf.display());

        check_vault(&path)?;
        let files_for_parse: Vec<_> = get_files_for_parse(&path);

        #[cfg(feature = "logging")]
        log::debug!("Found {} markdown files to parse", files_for_parse.len());

        #[allow(unused_variables)]
        #[allow(clippy::manual_ok_err)]
        let files = Self::parse_files_with_ignore(&files_for_parse, |file| F::from_file(file));

        #[cfg(feature = "logging")]
        log::info!("Parsed {} files", files.len());

        Ok(Self {
            files,
            path: path_buf,
        })
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

    #[derive(Clone, Deserialize, Serialize)]
    pub struct TestProperties {
        #[allow(dead_code)]
        not_correct: String,
    }

    #[test]
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

    #[test]
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
