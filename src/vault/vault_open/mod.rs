//! Module for open impl [`Vault`]

pub mod options;

use super::Vault;
use crate::note::{Note, note_on_disk::NoteOnDisk};
pub use options::VaultOptions;
use serde::de::DeserializeOwned;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

type FilterEntry = dyn FnMut(&DirEntry) -> bool;

/// Builder for [`Vault`]
pub struct VaultBuilder<'a> {
    options: &'a VaultOptions,
    include_hidden: bool,
    follow_links: bool,
    follow_root_links: bool,
    max_depth: Option<usize>,
    min_depth: Option<usize>,
    filter_entry: Option<Box<FilterEntry>>,
}

impl Debug for VaultBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultBuilder")
            .field("options", self.options)
            .finish()
    }
}

impl PartialEq for VaultBuilder<'_> {
    fn eq(&self, other: &Self) -> bool {
        (
            self.options,
            self.include_hidden,
            self.follow_links,
            self.follow_root_links,
            self.max_depth,
            self.min_depth,
            self.filter_entry.is_some(),
        ) == (
            other.options,
            other.include_hidden,
            other.follow_links,
            other.follow_root_links,
            other.max_depth,
            other.min_depth,
            other.filter_entry.is_some(),
        )
    }
}

impl Eq for VaultBuilder<'_> {}

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

macro_rules! impl_setter {
    ($name:ident, $t:ty) => {
        #[must_use]
        #[allow(missing_docs)]
        pub const fn $name(mut self, $name: $t) -> Self {
            self.$name = $name;
            self
        }
    };
}

impl<'a> VaultBuilder<'a> {
    /// Create default [`VaultBuilder`]
    #[must_use]
    pub const fn new(options: &'a VaultOptions) -> Self {
        Self {
            options,
            include_hidden: false,
            follow_links: false,
            follow_root_links: true,
            max_depth: None,
            min_depth: None,
            filter_entry: None,
        }
    }

    impl_setter!(include_hidden, bool);
    impl_setter!(follow_links, bool);
    impl_setter!(follow_root_links, bool);

    /// Set max depth
    #[must_use]
    pub const fn max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    /// Set min depth
    #[must_use]
    pub const fn min_depth(mut self, min_depth: usize) -> Self {
        self.min_depth = Some(min_depth);
        self
    }

    /// Set custom filter entry
    #[must_use]
    pub fn filter_entry<F>(mut self, f: F) -> Self
    where
        F: FnMut(&DirEntry) -> bool + 'static,
    {
        self.filter_entry = Some(Box::new(f));
        self
    }

    fn ignored_hidden_files(include_hidden: bool, entry: &DirEntry) -> bool {
        if !include_hidden && is_hidden(entry.path()) {
            return false;
        }

        true
    }

    fn get_files_from_walkdir(self) -> impl Iterator<Item = PathBuf> {
        let include_hidden = self.include_hidden;
        let mut custom_filter_entry = self.filter_entry.unwrap_or_else(|| Box::new(|_| true));

        WalkDir::new(self.options.path())
            .follow_links(self.follow_links)
            .follow_root_links(self.follow_root_links)
            .max_depth(self.max_depth.unwrap_or(usize::MAX))
            .min_depth(self.min_depth.unwrap_or(1))
            .into_iter()
            .filter_entry(move |entry| {
                Self::ignored_hidden_files(include_hidden, entry) && custom_filter_entry(entry)
            })
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(DirEntry::into_path)
            .filter(|path| is_md_file(path))
    }

    /// Into [`VaultBuilder`] to iterator
    #[allow(clippy::should_implement_trait)]
    #[cfg(not(target_family = "wasm"))]
    pub fn into_iter<F>(self) -> impl Iterator<Item = Result<F, F::Error>>
    where
        F: crate::note::note_read::NoteFromFile,
        F::Properties: DeserializeOwned,
        F::Error: From<std::io::Error>,
    {
        let files = self.get_files_from_walkdir();

        files.map(|path| F::from_file(path))
    }

    /// Into [`VaultBuilder`] to parallel iterator
    #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
    #[cfg(feature = "rayon")]
    #[cfg(not(target_family = "wasm"))]
    #[must_use]
    pub fn into_par_iter<F>(self) -> impl rayon::iter::ParallelIterator<Item = Result<F, F::Error>>
    where
        F: crate::prelude::NoteFromFile + Send,
        F::Properties: DeserializeOwned,
        F::Error: From<std::io::Error> + Send,
    {
        use rayon::prelude::*;

        let files: Vec<_> = self.get_files_from_walkdir().collect();
        files.into_par_iter().map(|path| F::from_file(path))
    }
}

impl<N> Vault<N>
where
    N: Note,
{
    fn impl_build_vault(notes: Vec<N>, options: VaultOptions) -> Self {
        #[cfg(feature = "logging")]
        log::debug!(
            "Building vault for {:?} with {} files",
            options,
            notes.len()
        );

        Self {
            notes,
            path: options.into_path(),
        }
    }

    /// Build vault from iterator
    pub fn build_vault(iter: impl Iterator<Item = N>, options: &VaultOptions) -> Self {
        let notes: Vec<_> = iter.collect();

        Self::impl_build_vault(notes, options.clone())
    }

    /// Build vault from parallel iterator
    #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
    #[cfg(feature = "rayon")]
    pub fn par_build_vault(
        iter: impl rayon::iter::ParallelIterator<Item = N>,
        options: &VaultOptions,
    ) -> Self
    where
        N: Send,
    {
        let notes: Vec<_> = iter.collect();

        Self::impl_build_vault(notes, options.clone())
    }
}

/// Trait for build [`Vault`] from iterator
pub trait IteratorVaultBuilder<N = NoteOnDisk>: Iterator<Item = N>
where
    Self: Sized,
    N: Note,
{
    /// Build [`Vault`] from iterator
    fn build_vault(self, options: &VaultOptions) -> Vault<N> {
        Vault::build_vault(self, options)
    }
}

impl<N, I> IteratorVaultBuilder<N> for I
where
    N: Note,
    I: Iterator<Item = N>,
{
}

/// Trait for build [`Vault`] from parallel iterator
#[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
#[cfg(feature = "rayon")]
pub trait ParallelIteratorVaultBuilder<N = NoteOnDisk>:
    rayon::iter::ParallelIterator<Item = N>
where
    N: Note + Send,
{
    /// Build [`Vault`] from parallel iterator
    #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
    fn build_vault(self, options: &VaultOptions) -> Vault<N> {
        Vault::par_build_vault(self, options)
    }
}

#[cfg(feature = "rayon")]
impl<F, I> ParallelIteratorVaultBuilder<F> for I
where
    F: Note + Send,
    I: rayon::iter::ParallelIterator<Item = F>,
{
}

#[cfg(test)]
#[cfg(not(target_family = "wasm"))]
mod tests {
    use super::*;
    use crate::note::note_in_memory;
    use crate::prelude::NoteFromFile;
    use crate::prelude::NoteInMemory;
    use crate::vault::VaultInMemory;
    use crate::vault::vault_test::create_files_for_vault;
    use std::fs::File;
    use std::io::Write;

    fn impl_open<F>(path: impl AsRef<Path>) -> Vault<F>
    where
        F: NoteFromFile,
        F::Error: From<std::io::Error>,
        F::Properties: DeserializeOwned,
    {
        let options = VaultOptions::new(path);

        VaultBuilder::new(&options)
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options)
    }

    #[cfg(feature = "rayon")]
    fn impl_par_open<F>(path: impl AsRef<Path>) -> Vault<F>
    where
        F: NoteFromFile + Send,
        F::Error: From<std::io::Error> + Send,
        F::Properties: DeserializeOwned,
    {
        use rayon::prelude::*;

        let options = VaultOptions::new(path);

        VaultBuilder::new(&options)
            .into_par_iter()
            .map(|file| file.unwrap())
            .build_vault(&options)
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn open() {
        let (path, vault_notes) = create_files_for_vault().unwrap();

        let vault: VaultInMemory = impl_open(&path);

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "rayon")]
    fn par_open() {
        let (path, vault_notes) = create_files_for_vault().unwrap();

        let vault: VaultInMemory = impl_par_open(&path);

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn ignore_not_md_files() {
        let (path, vault_notes) = create_files_for_vault().unwrap();
        File::create(path.path().join("extra_file.not_md")).unwrap();

        let vault: VaultInMemory = impl_open(&path);

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "rayon")]
    fn par_ignore_not_md_files() {
        let (path, vault_notes) = create_files_for_vault().unwrap();
        File::create(path.path().join("extra_file.not_md")).unwrap();

        let vault: VaultInMemory = impl_par_open(&path);

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
        let errors = VaultBuilder::new(&options)
            .into_iter::<NoteInMemory>()
            .filter_map(Result::err)
            .collect::<Vec<_>>();

        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors.last(),
            Some(note_in_memory::Error::InvalidFormat(_))
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
        let errors = VaultBuilder::new(&options)
            .into_par_iter::<NoteInMemory>()
            .filter_map(Result::err)
            .collect::<Vec<_>>();

        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors.last(),
            Some(note_in_memory::Error::InvalidFormat(_))
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
        let vault = VaultBuilder::new(&options)
            .into_iter::<NoteInMemory>()
            .filter_map(|file| match file {
                Ok(file) => Some(file),
                Err(error) => {
                    errors.push(error);

                    None
                }
            })
            .build_vault(&options);

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());

        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors.last(),
            Some(note_in_memory::Error::InvalidFormat(_))
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
        let vault = VaultBuilder::new(&options)
            .into_par_iter::<NoteInMemory>()
            .filter_map(|file| match file {
                Ok(file) => Some(file),
                Err(error) => {
                    errors.lock().unwrap().push(error);

                    None
                }
            })
            .build_vault(&options);

        assert_eq!(vault.count_notes(), vault_notes.len());
        assert_eq!(vault.path(), path.path());

        assert_eq!(errors.lock().unwrap().len(), 1);
        assert!(matches!(
            errors.lock().unwrap().last(),
            Some(note_in_memory::Error::InvalidFormat(_))
        ));
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn include_hidden() {
        let (path, files) = create_files_for_vault().unwrap();

        let mut file = File::create_new(path.path().join(".hidden.md")).unwrap();
        file.write_all(b"hidden information").unwrap();

        let options = VaultOptions::new(&path);

        let vault_with_hidden: VaultInMemory = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options);

        let vault_without_hidden: VaultInMemory = VaultBuilder::new(&options)
            .include_hidden(false)
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options);

        assert_eq!(vault_with_hidden.count_notes(), files.len() + 1);
        assert_eq!(vault_without_hidden.count_notes(), files.len());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn max_depth() {
        let (path, _) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let vault: VaultInMemory = VaultBuilder::new(&options)
            .max_depth(1) // Without `data/main.md`
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options);

        assert_eq!(vault.count_notes(), 2);
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn min_depth() {
        let (path, _) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let vault: VaultInMemory = VaultBuilder::new(&options)
            .min_depth(2) // Only `data/main.md`
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options);

        assert_eq!(vault.count_notes(), 1);
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn filter_entry() {
        let (path, _) = create_files_for_vault().unwrap();

        let options = VaultOptions::new(&path);
        let vault: VaultInMemory = VaultBuilder::new(&options)
            .filter_entry(|entry| !entry.file_name().eq_ignore_ascii_case("main.md"))
            .into_iter()
            .map(|file| file.unwrap())
            .build_vault(&options);

        assert_eq!(vault.count_notes(), 1);
    }
}
