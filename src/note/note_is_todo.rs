//! Impl trait [`NoteIsTodo`]

use super::{DefaultProperties, Note};

/// Trait for check note is marked todo
pub trait NoteIsTodo: Note {
    /// Note is marked todo?
    ///
    /// # Example
    /// ```
    /// use obsidian_parser::prelude::*;
    ///
    /// let raw_text = "---\ntags:\n- todo\n---\nSameData";
    /// let note = NoteInMemory::from_string(raw_text).unwrap();
    ///
    /// assert!(note.is_todo().unwrap());
    /// ```
    fn is_todo(&self) -> Result<bool, Self::Error>;
}

impl<N> NoteIsTodo for N
where
    N: Note<Properties = DefaultProperties>,
    N::Error: From<serde_yml::Error>,
{
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), ret, fields(path = format!("{:?}", self.path()))))]
    fn is_todo(&self) -> Result<bool, N::Error> {
        if self.content()?.contains("todo") {
            #[cfg(feature = "tracing")]
            tracing::trace!("Found todo in content");

            return Ok(true);
        }

        let properties = self.properties()?.unwrap_or_default();
        if let Some(value) = properties.get("tags") {
            let tags: Vec<String> = serde_yml::from_value(value.clone())?;

            if tags.contains(&"todo".to_string()) {
                #[cfg(feature = "tracing")]
                tracing::trace!("Found todo in properties");

                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::note::{NoteFromFile, NoteFromReader, NoteFromString};
    use std::io::{Cursor, Write};
    use tempfile::NamedTempFile;

    const TEST_DATA_HAVE: &str = "---\ntags:\n- todo\n---\nSameData";
    const TEST_DATA_NOT_HAVE: &str = "---\ntags:\n- not_todo\n---\nSameData";

    fn is_todo<N>(note: &N) -> Result<(), N::Error>
    where
        N: Note<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error>,
    {
        assert!(note.is_todo()?);
        Ok(())
    }

    fn is_not_todo<N>(note: &N) -> Result<(), N::Error>
    where
        N: Note<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error>,
    {
        assert!(!note.is_todo()?);
        Ok(())
    }

    pub(crate) fn from_string_is_todo<N>() -> Result<(), N::Error>
    where
        N: NoteFromString<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error>,
    {
        let note = N::from_string(TEST_DATA_HAVE)?;
        is_todo(&note)
    }

    pub(crate) fn from_string_is_not_todo<N>() -> Result<(), N::Error>
    where
        N: NoteFromString<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error>,
    {
        let note = N::from_string(TEST_DATA_NOT_HAVE)?;
        is_not_todo(&note)
    }

    pub(crate) fn from_reader_is_todo<N>() -> Result<(), N::Error>
    where
        N: NoteFromReader<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error> + From<std::io::Error>,
    {
        let note = N::from_reader(&mut Cursor::new(TEST_DATA_HAVE))?;
        is_todo(&note)
    }

    pub(crate) fn from_reader_is_not_todo<N>() -> Result<(), N::Error>
    where
        N: NoteFromReader<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error> + From<std::io::Error>,
    {
        let note = N::from_reader(&mut Cursor::new(TEST_DATA_NOT_HAVE))?;
        is_not_todo(&note)
    }

    pub(crate) fn from_file_is_todo<N>() -> Result<(), N::Error>
    where
        N: NoteFromFile<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error> + From<std::io::Error>,
    {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(TEST_DATA_HAVE.as_bytes()).unwrap();

        let note = N::from_file(file.path())?;
        is_todo(&note)
    }

    pub(crate) fn from_file_is_not_todo<N>() -> Result<(), N::Error>
    where
        N: NoteFromFile<Properties = DefaultProperties>,
        N::Error: From<serde_yml::Error> + From<std::io::Error>,
    {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(TEST_DATA_NOT_HAVE.as_bytes()).unwrap();

        let note = N::from_file(file.path())?;
        is_not_todo(&note)
    }

    macro_rules! impl_all_tests_is_todo {
        ($impl_note:path) => {
            #[allow(unused_imports)]
            use $crate::note::note_is_todo::tests::*;

            impl_test_for_note!(impl_from_string_is_todo, from_string_is_todo, $impl_note);
            impl_test_for_note!(
                impl_from_string_is_not_todo,
                from_string_is_not_todo,
                $impl_note
            );

            impl_test_for_note!(impl_from_reader_is_todo, from_reader_is_todo, $impl_note);
            impl_test_for_note!(
                impl_from_reader_is_not_todo,
                from_reader_is_not_todo,
                $impl_note
            );

            impl_test_for_note!(impl_from_file_is_todo, from_file_is_todo, $impl_note);
            impl_test_for_note!(
                impl_from_file_is_not_todo,
                from_file_is_not_todo,
                $impl_note
            );
        };
    }

    pub(crate) use impl_all_tests_is_todo;
}
