use super::Vault;
use crate::obfile::ObFile;
use std::collections::HashSet;

impl<F> Vault<F>
where
    F: ObFile,
{
    /// Returns duplicated note name
    ///
    /// # Performance
    /// Operates in O(n) time for large vaults
    ///
    /// # Other
    /// See [`check_unique_note_name`](Vault::check_unique_note_name)
    pub fn get_duplicates_notes_by_name(&self) -> Vec<String> {
        #[cfg(feature = "logging")]
        log::debug!(
            "Get duplicates notes by name in {} ({} notes)",
            self.path().display(),
            self.count_notes()
        );

        let mut seens_notes = HashSet::new();
        let mut duplicated_notes = Vec::new();

        #[allow(
            clippy::missing_panics_doc,
            clippy::unwrap_used,
            reason = "In any case, we will have a path to the files"
        )]
        for note in self.notes() {
            let note_name = note.note_name().unwrap();

            if !seens_notes.insert(note_name.clone()) {
                #[cfg(feature = "logging")]
                log::trace!("Found duplicate");

                duplicated_notes.push(note_name);
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
    pub fn have_duplicates_notes(&self) -> bool {
        !self.get_duplicates_notes_by_name().is_empty()
    }

    #[cfg(feature = "digest")]
    #[cfg_attr(docsrs, doc(cfg(feature = "digest")))]
    pub fn get_duplicates_notes_by_content<'a, D>(&'a self) -> Result<Vec<&'a F>, F::Error>
    where
        D: digest::Digest,
    {
        #[cfg(feature = "logging")]
        log::debug!(
            "Get duplicates notes by content in {} ({} notes)",
            self.path().display(),
            self.count_notes()
        );

        let mut seens_notes = HashSet::new();
        let mut duplicated_notes = Vec::new();

        #[allow(
            clippy::missing_panics_doc,
            clippy::unwrap_used,
            reason = "In any case, we will have a path to the files"
        )]
        for note in self.notes() {
            let hashed_content = D::digest(note.content()?.as_bytes());

            if !seens_notes.insert(hashed_content) {
                #[cfg(feature = "logging")]
                log::trace!("Found duplicate");

                duplicated_notes.push(note);
            }
        }

        #[cfg(feature = "logging")]
        if !duplicated_notes.is_empty() {
            log::warn!("Found {} duplicate filenames", duplicated_notes.len());
        }

        Ok(duplicated_notes)
    }

    #[cfg(feature = "digest")]
    #[cfg(feature = "rayon")]
    #[cfg_attr(docsrs, doc(cfg(feature = "digest")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
    pub fn par_get_duplicates_notes_by_content<D>(&self) -> Result<Vec<F>, F::Error>
    where
        F: Clone + Send,
        D: digest::Digest,
    {
        use rayon::prelude::*;

        #[cfg(feature = "logging")]
        log::debug!(
            "Get duplicates notes by content in {} ({} notes)",
            self.path().display(),
            self.count_notes()
        );

        #[cfg(feature = "logging")]
        log::debug!("Hashing notes...");

        let notes: Vec<_> = self
            .notes()
            .clone()
            .into_par_iter()
            .map(|note| {
                let hash = D::digest(note.content().unwrap().as_bytes());

                (note, hash)
            })
            .collect();

        #[cfg(feature = "logging")]
        log::debug!("Done hashing notes!");

        let mut seens_notes = HashSet::new();
        
        let mut duplicated_notes = Vec::new();

        #[allow(
            clippy::missing_panics_doc,
            clippy::unwrap_used,
            reason = "In any case, we will have a path to the files"
        )]
        for (note, hashed_content) in notes {
            if !seens_notes.insert(hashed_content) {
                #[cfg(feature = "logging")]
                log::trace!("Found duplicate");

                duplicated_notes.push((note));
            }
        }

        #[cfg(feature = "logging")]
        if !duplicated_notes.is_empty() {
            log::warn!("Found {} duplicate filenames", duplicated_notes.len());
        }

        Ok(duplicated_notes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        obfile::ObFileRead,
        prelude::{FilesBuilder, IteratorFilesBuilder, ObFileInMemory, VaultOptions},
        vault::Vault,
    };
    use serde::de::DeserializeOwned;
    use std::fs::File;
    use tempfile::TempDir;

    fn create_vault_with_diplicates_files<F>() -> (Vault<F>, TempDir)
    where
        F: ObFileRead,
        F::Error: From<std::io::Error>,
        F::Properties: DeserializeOwned,
    {
        let temp_dir = TempDir::new().unwrap();

        let file1 = File::create(&temp_dir.path().join("file.md")).unwrap();
        file1.

        let path_to_duplicate_file = temp_dir.path().join("folder");
        std::fs::create_dir(&path_to_duplicate_file).unwrap();
        File::create(path_to_duplicate_file.join("file.md")).unwrap();

        let options = VaultOptions::new(&temp_dir);
        let vault = FilesBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(Result::unwrap)
            .build_vault(&options)
            .unwrap();

        (vault, temp_dir)
    }

    fn create_vault_without_diplicates_files<F>() -> (Vault<F>, TempDir)
    where
        F: ObFileRead,
        F::Error: From<std::io::Error>,
        F::Properties: DeserializeOwned,
    {
        let temp_dir = TempDir::new().unwrap();

        File::create(&temp_dir.path().join("file.md")).unwrap();

        let options = VaultOptions::new(&temp_dir);
        let vault = FilesBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(Result::unwrap)
            .build_vault(&options)
            .unwrap();

        (vault, temp_dir)
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn get_duplicates_notes_by_name() {
        let (vault, _path) = create_vault_without_diplicates_files::<ObFileInMemory>();

        assert!(vault.get_duplicates_notes_by_name().is_empty());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn with_duplicates_notes_by_name() {
        let (vault, _path) = create_vault_with_diplicates_files::<ObFileInMemory>();

        let duplicated_notes: Vec<_> = vault.get_duplicates_notes_by_name();
        assert_eq!(duplicated_notes, ["file".to_string()]);
        assert!(vault.have_duplicates_notes());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn check_unique_note_name() {
        let (vault, _path) = create_vault_without_diplicates_files::<ObFileInMemory>();

        assert!(!vault.have_duplicates_notes());
    }
}
