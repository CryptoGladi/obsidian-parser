//! Found duplication in vault

use super::Vault;
use crate::note::Note;

fn get_duplicates<'a, N>(sorted_notes: &[&'a N]) -> Vec<&'a N>
where
    N: Note,
{
    let mut duplicated = Vec::new();
    let mut add_two = true;
    for i in 1..sorted_notes.len() {
        let note1 = sorted_notes[i - 1];
        let note2 = sorted_notes[i];

        if let (Some(name1), Some(name2)) = (note1.note_name(), note2.note_name()) {
            if name1 == name2 {
                if add_two {
                    add_two = false;
                    duplicated.push(note1);
                }

                duplicated.push(note2);
            } else {
                add_two = true;
            }
        }
    }

    duplicated
}

impl<F> Vault<F>
where
    F: Note,
{
    /// Returns duplicated note name
    ///
    /// # Performance
    /// Operates in O(n log n) time for large vaults
    ///
    /// # Other
    /// See [`have_unique_note_by_name`](Vault::have_duplicates_notes_by_name)
    #[must_use]
    pub fn get_duplicates_notes_by_name(&self) -> Vec<&F> {
        #[cfg(feature = "logging")]
        log::debug!(
            "Get duplicates notes by name in {} ({} notes)",
            self.path().display(),
            self.count_notes()
        );

        let sorted_notes = {
            let mut notes: Vec<_> = self.notes().iter().collect();
            notes.sort_unstable_by_key(|note| note.note_name());

            notes
        };

        let duplicated_notes = get_duplicates(&sorted_notes);

        #[cfg(feature = "logging")]
        log::debug!("Found {} duplicated notes", duplicated_notes.len());

        duplicated_notes
    }

    /// Parallel returns duplicated note name
    ///
    /// # Performance
    /// Operates in O(n log n) time for large vaults
    ///
    /// # Other
    /// See [`par_have_unique_note_by_name`](Vault::par_have_duplicates_notes_by_name)
    #[cfg(feature = "rayon")]
    #[must_use]
    pub fn par_get_duplicates_notes_by_name<'a>(&'a self) -> Vec<&'a F>
    where
        &'a F: Send,
    {
        use rayon::prelude::*;

        #[cfg(feature = "logging")]
        log::debug!(
            "Par get duplicates notes by name in {} ({} notes)",
            self.path().display(),
            self.count_notes()
        );

        let sorted_notes = {
            let mut notes: Vec<_> = self.notes().iter().collect();
            notes.par_sort_unstable_by_key(|note| note.note_name());

            notes
        };

        let duplicated_notes = get_duplicates(&sorted_notes);

        #[cfg(feature = "logging")]
        log::debug!("Found {} duplicated notes", duplicated_notes.len());

        duplicated_notes
    }

    /// Checks if all note name in the vault are unique
    ///
    /// # Returns
    /// `true` if all note name are unique, `false` otherwise
    ///
    /// # Performance
    /// Operates in O(n) time for large vaults
    ///
    /// # Other
    /// See [`get_duplicates_notes_by_name`](Vault::get_duplicates_notes_by_name)
    #[must_use]
    pub fn have_duplicates_notes_by_name(&self) -> bool {
        !self.get_duplicates_notes_by_name().is_empty()
    }

    /// Parallel checks if all note name in the vault are unique
    ///
    /// # Returns
    /// `true` if all note name are unique, `false` otherwise
    ///
    /// # Performance
    /// Operates in O(n) time for large vaults
    ///
    /// # Other
    /// See [`par_get_duplicates_notes_by_name`](Vault::par_get_duplicates_notes_by_name)
    #[must_use]
    #[cfg(feature = "rayon")]
    pub fn par_have_duplicates_notes_by_name<'a>(&'a self) -> bool
    where
        &'a F: Send,
    {
        !self.par_get_duplicates_notes_by_name().is_empty()
    }

    /// Get duplicates by content
    #[cfg(feature = "digest")]
    #[cfg_attr(docsrs, doc(cfg(feature = "digest")))]
    pub fn get_duplicates_notes_by_content<D>(&self) -> Result<Vec<&F>, F::Error>
    where
        D: digest::Digest,
    {
        #[cfg(feature = "logging")]
        log::debug!(
            "Get duplicates notes by content in {} ({} notes)",
            self.path().display(),
            self.count_notes()
        );

        let mut hashed = Vec::with_capacity(self.count_notes());
        for i in 0..self.count_notes() {
            let content = self.notes()[i].content()?;
            let hash = D::digest(content.as_bytes());

            hashed.push(hash);
        }

        let sorted_notes = {
            let mut notes: Vec<_> = self.notes().iter().zip(hashed).collect();
            notes.sort_unstable_by_key(|(_, hash)| hash.clone());

            notes
        };

        let mut duplicated_notes = Vec::new();
        let mut add_two = true;
        for i in 1..sorted_notes.len() {
            let (note1, hash1) = &sorted_notes[i - 1];
            let (note2, hash2) = &sorted_notes[i];

            if hash1 == hash2 {
                if add_two {
                    add_two = false;
                    duplicated_notes.push(*note1);
                }

                duplicated_notes.push(*note2);
            } else {
                add_two = true;
            }
        }

        Ok(duplicated_notes)
    }

    /// Check have duplicates notes by content
    #[cfg(feature = "digest")]
    #[cfg_attr(docsrs, doc(cfg(feature = "digest")))]
    pub fn have_duplicates_notes_by_content<D>(&self) -> Result<bool, F::Error>
    where
        D: digest::Digest,
    {
        Ok(!self.get_duplicates_notes_by_content::<D>()?.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        note::{Note, NoteFromFile},
        prelude::{IteratorVaultBuilder, NoteInMemory, VaultBuilder, VaultOptions},
        vault::Vault,
    };
    use serde::de::DeserializeOwned;
    use std::{fs::File, io::Write};
    use tempfile::TempDir;

    fn create_vault_with_diplicates_files<F>() -> (Vault<F>, TempDir)
    where
        F: NoteFromFile,
        F::Error: From<std::io::Error>,
        F::Properties: DeserializeOwned,
    {
        let temp_dir = TempDir::new().unwrap();

        let mut file1 = File::create(&temp_dir.path().join("file.md")).unwrap();
        file1.write_all(b"same text").unwrap();

        let path_to_duplicate_file = temp_dir.path().join("folder");
        std::fs::create_dir(&path_to_duplicate_file).unwrap();
        let mut file2 = File::create(path_to_duplicate_file.join("file.md")).unwrap();
        file2.write_all(b"same text").unwrap();

        let options = VaultOptions::new(&temp_dir);
        let vault = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(Result::unwrap)
            .build_vault(&options)
            .unwrap();

        (vault, temp_dir)
    }

    fn create_vault_without_diplicates_files<F>() -> (Vault<F>, TempDir)
    where
        F: NoteFromFile,
        F::Error: From<std::io::Error>,
        F::Properties: DeserializeOwned,
    {
        let temp_dir = TempDir::new().unwrap();

        File::create(&temp_dir.path().join("file.md")).unwrap();

        let options = VaultOptions::new(&temp_dir);
        let vault = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(Result::unwrap)
            .build_vault(&options)
            .unwrap();

        (vault, temp_dir)
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn with_duplicates_notes_by_name() {
        let (vault, _path) = create_vault_with_diplicates_files::<NoteInMemory>();

        let duplicated_notes: Vec<_> = vault
            .get_duplicates_notes_by_name()
            .into_iter()
            .map(|note| note.note_name().unwrap())
            .collect();

        assert_eq!(duplicated_notes, ["file".to_string(), "file".to_string()]);
        assert!(vault.have_duplicates_notes_by_name());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn without_duplicates_notes_by_name() {
        let (vault, _path) = create_vault_without_diplicates_files::<NoteInMemory>();

        let duplicated_notes: Vec<_> = vault
            .get_duplicates_notes_by_name()
            .into_iter()
            .map(|note| note.note_name().unwrap())
            .collect();

        assert_eq!(duplicated_notes.is_empty(), true);
        assert!(!vault.have_duplicates_notes_by_name());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "rayon")]
    fn par_with_duplicates_notes_by_name() {
        let (vault, _path) = create_vault_with_diplicates_files::<NoteInMemory>();

        let duplicated_notes: Vec<_> = vault
            .par_get_duplicates_notes_by_name()
            .into_iter()
            .map(|note| note.note_name().unwrap())
            .collect();

        assert_eq!(duplicated_notes, ["file".to_string(), "file".to_string()]);
        assert!(vault.par_have_duplicates_notes_by_name());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "rayon")]
    fn par_without_duplicates_notes_by_name() {
        let (vault, _path) = create_vault_without_diplicates_files::<NoteInMemory>();

        let duplicated_notes: Vec<_> = vault
            .par_get_duplicates_notes_by_name()
            .into_iter()
            .map(|note| note.note_name().unwrap())
            .collect();

        assert_eq!(duplicated_notes.is_empty(), true);
        assert!(!vault.par_have_duplicates_notes_by_name());
    }
    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "digest")]
    fn with_duplicates_notes_by_content() {
        let (vault, _path) = create_vault_with_diplicates_files::<NoteInMemory>();

        let duplicated_notes: Vec<_> = vault
            .get_duplicates_notes_by_content::<sha2::Sha256>()
            .unwrap()
            .into_iter()
            .map(|note| note.note_name().unwrap())
            .collect();

        assert_eq!(duplicated_notes, ["file".to_string(), "file".to_string()]);
        assert!(
            vault
                .have_duplicates_notes_by_content::<sha2::Sha256>()
                .unwrap()
        );
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "digest")]
    fn without_duplicates_notes_by_content() {
        let (vault, _path) = create_vault_without_diplicates_files::<NoteInMemory>();

        let duplicated_notes: Vec<_> = vault
            .get_duplicates_notes_by_content::<sha2::Sha256>()
            .unwrap()
            .into_iter()
            .map(|note| note.note_name().unwrap())
            .collect();

        assert_eq!(duplicated_notes.is_empty(), true);
        assert!(
            !vault
                .have_duplicates_notes_by_content::<sha2::Sha256>()
                .unwrap()
        );
    }
}
