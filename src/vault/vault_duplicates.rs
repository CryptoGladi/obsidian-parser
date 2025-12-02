//! Found duplication in vault

use std::collections::HashSet;

use super::Vault;
use crate::note::Note;

impl<N> Vault<N>
where
    N: Note,
{
    /// Returns duplicated note name
    ///
    /// # Performance
    /// Operates in O(n log n) time for large vaults
    ///
    /// # Other
    /// See [`have_unique_note_by_name`](Vault::have_duplicates_notes_by_name)
    #[must_use]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(path = %self.path.display(), count_notes = %self.notes.len())))]
    pub fn get_duplicates_notes_by_name(&self) -> Vec<&N> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Get duplicates notes by name...");

        let mut duplicated_notes = Vec::new();
        let mut viewed = HashSet::new();
        for note in self.notes() {
            if let Some(note_name) = note.note_name() {
                let already_have = !viewed.insert(note_name);

                if already_have {
                    duplicated_notes.push(note);
                }
            }
        }

        #[cfg(feature = "tracing")]
        tracing::debug!("Found {} duplicated notes", duplicated_notes.len());

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

    /// Get duplicates by content
    #[cfg(feature = "digest")]
    #[cfg_attr(docsrs, doc(cfg(feature = "digest")))]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(path = %self.path.display(), count_notes = %self.notes.len())))]
    pub fn get_duplicates_notes_by_content<D>(&self) -> Result<Vec<&N>, N::Error>
    where
        D: digest::Digest,
    {
        #[cfg(feature = "tracing")]
        tracing::debug!("Get duplicates notes by content");

        let hashed = {
            let mut hashed = Vec::with_capacity(self.count_notes());
            for i in 0..self.count_notes() {
                let content = self.notes()[i].content()?;
                let hash = D::digest(content.as_bytes());

                hashed.push(hash);
            }

            hashed
        };

        let mut duplicated_notes = Vec::new();
        let mut viewed = HashSet::new();
        for (note, hash_content) in self.notes().iter().zip(hashed) {
            let already_have = !viewed.insert(hash_content);

            if already_have {
                duplicated_notes.push(note);
            }
        }

        #[cfg(feature = "tracing")]
        tracing::debug!("Found {} duplicated notes", duplicated_notes.len());

        Ok(duplicated_notes)
    }

    /// Check have duplicates notes by content
    #[cfg(feature = "digest")]
    #[cfg_attr(docsrs, doc(cfg(feature = "digest")))]
    pub fn have_duplicates_notes_by_content<D>(&self) -> Result<bool, N::Error>
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
            .build_vault(&options);

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
            .build_vault(&options);

        (vault, temp_dir)
    }

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    fn with_duplicates_notes_by_name() {
        let (vault, _path) = create_vault_with_diplicates_files::<NoteInMemory>();

        let duplicated_notes: Vec<_> = vault
            .get_duplicates_notes_by_name()
            .into_iter()
            .map(|note| note.note_name().unwrap())
            .collect();

        assert_eq!(duplicated_notes, ["file".to_string()]);
        assert!(vault.have_duplicates_notes_by_name());
    }

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
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

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    #[cfg(feature = "digest")]
    fn with_duplicates_notes_by_content() {
        let (vault, _path) = create_vault_with_diplicates_files::<NoteInMemory>();

        let duplicated_notes: Vec<_> = vault
            .get_duplicates_notes_by_content::<sha2::Sha256>()
            .unwrap()
            .into_iter()
            .map(|note| note.note_name().unwrap())
            .collect();

        assert_eq!(duplicated_notes, ["file".to_string()]);

        assert!(
            vault
                .have_duplicates_notes_by_content::<sha2::Sha256>()
                .unwrap()
        );
    }

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
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
