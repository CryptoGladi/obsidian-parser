//! Module for open impl [`Vault`]

use super::Vault;
use crate::{
    obfile::{ObFile, ObFileRead},
    prelude::ObFileOnDisk,
};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use thiserror::Error;
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
pub struct FilesBuilder<'a> {
    options: &'a VaultOptions,
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

impl<'a> FilesBuilder<'a> {
    #[must_use]
    pub const fn new(options: &'a VaultOptions) -> Self {
        Self {
            options,
            include_hidden: false,
        }
    }

    #[must_use]
    pub const fn include_hidden(mut self, include_hidden: bool) -> Self {
        self.include_hidden = include_hidden;
        self
    }

    fn ignored_hidden_files(include_hidden: bool, entry: &DirEntry) -> bool {
        if !include_hidden && is_hidden(entry.path()) {
            return false;
        }

        true
    }

    #[allow(clippy::should_implement_trait)]
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
            .map(DirEntry::into_path)
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

        let paths: Vec<_> = WalkDir::new(&self.options.path)
            .into_iter()
            .filter_entry(move |entry| {
                entry.depth() == 0 || Self::ignored_hidden_files(include_hidden, entry)
            })
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(DirEntry::into_path)
            .filter(|path| is_md_file(path))
            .collect();

        paths.into_par_iter().map(|path| F::from_file(path))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Dir not found")]
    NotFoundDir(PathBuf),
}

impl<F> Vault<F>
where
    F: ObFile,
{
    fn impl_build_vault(notes: Vec<F>, options: VaultOptions) -> Result<Self, Error> {
        if !options.path.is_dir() {
            return Err(Error::NotFoundDir(options.path));
        }

        #[cfg(feature = "logging")]
        log::debug!(
            "Building vault for {:?} with {} files",
            options,
            notes.len()
        );

        Ok(Self {
            notes,
            path: options.path,
        })
    }

    pub fn build_vault(
        iter: impl Iterator<Item = F>,
        options: &VaultOptions,
    ) -> Result<Self, Error> {
        let notes: Vec<_> = iter.collect();

        Self::impl_build_vault(notes, options.clone())
    }

    #[cfg(feature = "rayon")]
    pub fn par_build_vault(
        iter: impl rayon::iter::ParallelIterator<Item = F>,
        options: &VaultOptions,
    ) -> Result<Self, Error>
    where
        F: Send,
    {
        let notes: Vec<_> = iter.collect();

        Self::impl_build_vault(notes, options.clone())
    }
}

pub trait IteratorFilesBuilder<F = ObFileOnDisk>: Iterator<Item = F>
where
    Self: Sized,
    F: ObFile,
{
    fn build_vault(self, options: &VaultOptions) -> Result<Vault<F>, Error> {
        Vault::build_vault(self, options)
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
    fn build_vault(self, options: &VaultOptions) -> Result<Vault<F>, Error> {
        Vault::par_build_vault(self, options)
    }
}

#[cfg(feature = "rayon")]
impl<F, I> ParallelIteratorFilesBuilder<F> for I
where
    F: ObFile + Send,
    I: rayon::iter::ParallelIterator<Item = F>,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obfile::obfile_in_memory;
    use crate::prelude::ObFileInMemory;
    use crate::vault::VaultInMemory;
    use crate::vault::vault_test::create_files_for_vault;
    use std::fs::File;
    use std::io::Write;

    fn impl_open<F>(path: impl AsRef<Path>) -> Result<Vault<F>, Error>
    where
        F: ObFileRead,
        F::Error: From<std::io::Error>,
        F::Properties: DeserializeOwned,
    {
        let options = VaultOptions::new(path);
        FilesBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options)
    }

    #[cfg(feature = "rayon")]
    fn impl_par_open<F>(path: impl AsRef<Path>) -> Result<Vault<F>, Error>
    where
        F: ObFileRead + Send,
        F::Error: From<std::io::Error> + Send,
        F::Properties: DeserializeOwned,
    {
        use rayon::prelude::*;

        let options = VaultOptions::new(path);
        FilesBuilder::new(&options)
            .include_hidden(true)
            .into_par_iter()
            .map(|file| file.unwrap())
            .build_vault(&options)
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn open() {
        let (path, vault_notes) = create_files_for_vault().unwrap();

        let vault: VaultInMemory = impl_open(&path).unwrap();

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "rayon")]
    fn par_open() {
        let (path, vault_notes) = create_files_for_vault().unwrap();

        let vault: VaultInMemory = impl_par_open(&path).unwrap();

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn open_not_dir() {
        let (path, _) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let files = FilesBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|file| file.unwrap());

        drop(path); // Delete folder

        let result: Result<VaultInMemory, _> = files.build_vault(&options);
        assert!(matches!(result, Err(Error::NotFoundDir(_))));
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "rayon")]
    fn par_open_not_dir() {
        use rayon::prelude::*;

        let (path, _) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let files = FilesBuilder::new(&options)
            .include_hidden(true)
            .into_par_iter()
            .filter_map(Result::ok);

        drop(path); // Delete folder

        let result: Result<VaultInMemory, _> = files.build_vault(&options);
        assert!(matches!(result, Err(Error::NotFoundDir(_))));
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn ignore_not_md_files() {
        let (path, vault_notes) = create_files_for_vault().unwrap();
        File::create(path.path().join("extra_file.not_md")).unwrap();

        let vault: VaultInMemory = impl_open(&path).unwrap();

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "rayon")]
    fn par_ignore_not_md_files() {
        let (path, vault_notes) = create_files_for_vault().unwrap();
        File::create(path.path().join("extra_file.not_md")).unwrap();

        let vault: VaultInMemory = impl_par_open(&path).unwrap();

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn open_with_error() {
        let (path, _) = create_files_for_vault().unwrap();
        let mut file = File::create(path.path().join("not_file.md")).unwrap();
        file.write_all(b"---").unwrap();

        let options = VaultOptions::new(&path);
        let errors = FilesBuilder::new(&options)
            .include_hidden(true)
            .into_iter::<ObFileInMemory>()
            .filter_map(Result::err)
            .collect::<Vec<_>>();

        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors.last(),
            Some(obfile_in_memory::Error::InvalidFormat(_))
        ));
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "rayon")]
    fn par_open_with_error() {
        use rayon::prelude::*;

        let (path, _) = create_files_for_vault().unwrap();
        let mut file = File::create(path.path().join("not_file.md")).unwrap();
        file.write_all(b"---").unwrap();

        let options = VaultOptions::new(&path);
        let errors = FilesBuilder::new(&options)
            .include_hidden(true)
            .into_par_iter::<ObFileInMemory>()
            .filter_map(Result::err)
            .collect::<Vec<_>>();

        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors.last(),
            Some(obfile_in_memory::Error::InvalidFormat(_))
        ));
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn open_with_error_but_ignored() {
        let (path, vault_notes) = create_files_for_vault().unwrap();
        let mut file = File::create(path.path().join("not_file.md")).unwrap();
        file.write_all(b"---").unwrap();

        let options = VaultOptions::new(&path);

        let mut errors = Vec::new();
        let vault = FilesBuilder::new(&options)
            .include_hidden(true)
            .into_iter::<ObFileInMemory>()
            .filter_map(|file| match file {
                Ok(file) => Some(file),
                Err(error) => {
                    errors.push(error);

                    None
                }
            })
            .build_vault(&options)
            .unwrap();

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());

        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors.last(),
            Some(obfile_in_memory::Error::InvalidFormat(_))
        ));
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "rayon")]
    fn par_open_with_error_but_ignored() {
        use rayon::prelude::*;
        use std::sync::{Arc, Mutex};

        let (path, vault_notes) = create_files_for_vault().unwrap();
        let mut file = File::create(path.path().join("not_file.md")).unwrap();
        file.write_all(b"---").unwrap();

        let options = VaultOptions::new(&path);

        let errors = Arc::new(Mutex::new(Vec::new()));
        let vault = FilesBuilder::new(&options)
            .include_hidden(true)
            .into_par_iter::<ObFileInMemory>()
            .filter_map(|file| match file {
                Ok(file) => Some(file),
                Err(error) => {
                    errors.lock().unwrap().push(error);

                    None
                }
            })
            .build_vault(&options)
            .unwrap();

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());

        assert_eq!(errors.lock().unwrap().len(), 1);
        assert!(matches!(
            errors.lock().unwrap().last(),
            Some(obfile_in_memory::Error::InvalidFormat(_))
        ));
    }
}
