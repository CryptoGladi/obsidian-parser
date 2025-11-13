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
    #[must_use]
    pub fn get_duplicates_notes(&self) -> Vec<String> {
        #[cfg(feature = "logging")]
        log::debug!(
            "Get duplicates notes in {} ({} files)",
            self.path.display(),
            self.notes.len()
        );

        let mut seens_notes = HashSet::new();
        let mut duplicated_notes = Vec::new();

        #[allow(
            clippy::missing_panics_doc,
            clippy::unwrap_used,
            reason = "In any case, we will have a path to the files"
        )]
        for name_note in self.notes.iter().map(|x| x.note_name().unwrap()) {
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
    /// # Returns
    /// `true` if all filenames are unique, `false` otherwise
    ///
    /// # Performance
    /// Operates in O(n) time for large vaults
    ///
    /// # Other
    /// See [`get_duplicates_notes`](Vault::get_duplicates_notes)
    #[must_use]
    pub fn check_unique_note_name(&self) -> bool {
        self.get_duplicates_notes().is_empty()
    }
}
