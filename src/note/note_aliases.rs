//! Impl trait [`NoteAliases`]

use super::{DefaultProperties, Note};

const ALIASES_FIELD_NAME: &str = "aliases";

/// Getting aliases from note
///
/// # Example
///
/// ```
/// use obsidian_parser::prelude::*;
///
/// let raw_text = "---\ntags:\n- todo\n---\nSameData";
/// let note = NoteInMemory::from_string(raw_text).unwrap();
///
/// let aliases = note.aliases().unwrap();
/// assert!(aliases.is_empty());
/// ```
pub trait NoteAliases: Note {
    /// Get aliases from note
    ///
    /// # Example
    ///
    /// ```
    /// use obsidian_parser::prelude::*;
    ///
    /// let raw_text = "---\ntags:\n- todo\n---\nSameData";
    /// let note = NoteInMemory::from_string(raw_text).unwrap();
    ///
    /// let aliases = note.aliases().unwrap();
    /// assert!(aliases.is_empty());
    /// ```
    fn aliases(&self) -> Result<Vec<String>, Self::Error>;

    /// Get count aliases from note
    ///
    /// # Example
    ///
    /// ```
    /// use obsidian_parser::prelude::*;
    ///
    /// let raw_text = "---\naliases:\n- my_alias\n---\nSameData";
    /// let note = NoteInMemory::from_string(raw_text).unwrap();
    ///
    /// let count_aliases = note.count_aliases().unwrap();
    /// assert_eq!(count_aliases, 1);
    /// ```
    #[inline]
    fn count_aliases(&self) -> Result<usize, Self::Error> {
        let aliases = self.aliases()?;
        Ok(aliases.len())
    }

    /// Have aliases in note?
    ///
    /// # Example
    ///
    /// ```
    /// use obsidian_parser::prelude::*;
    ///
    /// let raw_text = "---\naliases:\n- my_alias\n---\nSameData";
    /// let note = NoteInMemory::from_string(raw_text).unwrap();
    ///
    /// let have_aliases = note.have_aliases().unwrap();
    /// assert!(have_aliases);
    /// ```
    #[inline]
    fn have_aliases(&self) -> Result<bool, Self::Error> {
        let aliases = self.count_aliases()?;
        Ok(aliases != 0)
    }
}

impl<N> NoteAliases for N
where
    N: Note<Properties = DefaultProperties>,
    N::Error: From<serde_yml::Error>,
{
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), ret, fields(path = format!("{:?}", self.path()))))]
    fn aliases(&self) -> Result<Vec<String>, Self::Error> {
        let properties = self.properties()?.unwrap_or_default();

        match properties.get(ALIASES_FIELD_NAME) {
            Some(value) => {
                let aliases = serde_yml::from_value(value.clone())?;

                Ok(aliases)
            }
            None => Ok(Vec::default()),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::note::{NoteFromFile, NoteFromReader, NoteFromString};
    use std::io::{Cursor, Write};
    use tempfile::NamedTempFile;

    const TEST_DATA_HAVE_ALIASES: &str = "---\naliases:\n- my_alias\n---\nSameData";
    const TEST_DATA_NOT_HAVE_ALIASES: &str = "---\ntags:\n- todo\n---\nSameData";

    fn have_aliases<N>(note: &N) -> Result<(), N::Error>
    where
        N: Note<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error>,
    {
        let aliases = note.aliases()?;

        assert_eq!(aliases, vec!["my_alias".to_string()]);
        Ok(())
    }

    fn have_not_aliases<N>(note: &N) -> Result<(), N::Error>
    where
        N: Note<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error>,
    {
        let aliases = note.aliases()?;

        assert!(aliases.is_empty());
        Ok(())
    }

    pub(crate) fn from_string_have_aliases<N>() -> Result<(), N::Error>
    where
        N: NoteFromString<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error>,
    {
        let note = N::from_string(TEST_DATA_HAVE_ALIASES)?;
        have_aliases(&note)
    }

    pub(crate) fn from_string_have_not_aliases<N>() -> Result<(), N::Error>
    where
        N: NoteFromString<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error>,
    {
        let note = N::from_string(TEST_DATA_NOT_HAVE_ALIASES)?;
        have_not_aliases(&note)
    }

    pub(crate) fn from_reader_have_aliases<N>() -> Result<(), N::Error>
    where
        N: NoteFromReader<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error> + From<std::io::Error>,
    {
        let note = N::from_reader(&mut Cursor::new(TEST_DATA_HAVE_ALIASES))?;
        have_aliases(&note)
    }

    pub(crate) fn from_reader_have_not_aliases<N>() -> Result<(), N::Error>
    where
        N: NoteFromReader<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error> + From<std::io::Error>,
    {
        let note = N::from_reader(&mut Cursor::new(TEST_DATA_NOT_HAVE_ALIASES))?;
        have_not_aliases(&note)
    }

    pub(crate) fn from_file_have_aliases<N>() -> Result<(), N::Error>
    where
        N: NoteFromFile<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error> + From<std::io::Error>,
    {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(TEST_DATA_HAVE_ALIASES.as_bytes()).unwrap();

        let note = N::from_file(file.path())?;
        have_aliases(&note)
    }

    pub(crate) fn from_file_have_not_aliases<N>() -> Result<(), N::Error>
    where
        N: NoteFromFile<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error> + From<std::io::Error>,
    {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(TEST_DATA_NOT_HAVE_ALIASES.as_bytes())
            .unwrap();

        let note = N::from_file(file.path())?;
        have_not_aliases(&note)
    }

    macro_rules! impl_all_tests_aliases {
        ($impl_note:path) => {
            #[allow(unused_imports)]
            use $crate::note::note_aliases::tests::*;

            impl_test_for_note!(
                impl_from_string_have_aliases,
                from_string_have_aliases,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_string_have_not_aliases,
                from_string_have_not_aliases,
                $impl_note
            );

            impl_test_for_note!(
                impl_from_reader_have_aliases,
                from_reader_have_aliases,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_reader_have_not_aliases,
                from_reader_have_not_aliases,
                $impl_note
            );

            impl_test_for_note!(
                impl_from_file_have_aliases,
                from_file_have_aliases,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_file_have_not_aliases,
                from_file_have_not_aliases,
                $impl_note
            );
        };
    }

    pub(crate) use impl_all_tests_aliases;
}
