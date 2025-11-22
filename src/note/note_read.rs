//! Impl traits for reading notes

use super::Note;
use serde::de::DeserializeOwned;
use std::{io::Read, path::Path};

/// Trait for parses an Obsidian note from a string
pub trait NoteFromString: Note
where
    Self::Properties: DeserializeOwned,
{
    /// Parses an Obsidian note from a string
    ///
    /// # Arguments
    /// - `raw_text`: Raw markdown content with optional YAML frontmatter
    fn from_string(raw_text: impl AsRef<str>) -> Result<Self, Self::Error>;
}

/// Trait for parses an Obsidian note from a reader
pub trait NoteFromReader: Note
where
    Self::Properties: DeserializeOwned,
    Self::Error: From<std::io::Error>,
{
    /// Parses an Obsidian note from a reader
    fn from_reader(read: &mut impl Read) -> Result<Self, Self::Error>;
}

impl<N> NoteFromReader for N
where
    N: NoteFromString,
    N::Properties: DeserializeOwned,
    N::Error: From<std::io::Error>,
{
    fn from_reader(read: &mut impl Read) -> Result<Self, Self::Error> {
        #[cfg(feature = "logging")]
        log::trace!("Parse obsidian file from reader");

        let mut data = Vec::new();
        read.read_to_end(&mut data)?;

        // SAFETY: Notes files in Obsidian (`*.md`) ensure that the file is encoded in UTF-8
        let text = unsafe { String::from_utf8_unchecked(data) };

        Self::from_string(&text)
    }
}

/// Trait for parses an Obsidian note from a file
#[cfg(not(target_family = "wasm"))]
pub trait NoteFromFile: Note
where
    Self::Properties: DeserializeOwned,
    Self::Error: From<std::io::Error>,
{
    /// Parses an Obsidian note from a file
    ///
    /// # Arguments
    /// - `path`: Filesystem path to markdown file
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Self::Error>;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        note::{DefaultProperties, parser},
        test_utils::is_error,
    };
    use std::{
        borrow::Cow,
        io::{Cursor, Write},
        path::PathBuf,
    };
    use tempfile::NamedTempFile;

    const TEST_DATA: &str = "---\n\
topic: life\n\
created: 2025-03-16\n\
---\n\
Test data\n\
---\n\
Two test data";

    const BROKEN_DATA: &str = "---\n\
    asdfv:--fs\n\
    sfsf\n\
    ---\n\
    TestData";

    const UNICODE_DATA: &str = "---\ndata: 💩\n---\nSuper data 💩💩💩";

    const SPACE_DATA: &str = "  ---\ntest: test-data\n---\n";

    fn test_data<T>(note: T, path: Option<PathBuf>) -> Result<(), T::Error>
    where
        T: Note<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let path = path.map(|p| Cow::Owned(p));
        let properties = note.properties()?.unwrap();

        assert_eq!(properties["topic"], "life");
        assert_eq!(properties["created"], "2025-03-16");
        assert_eq!(note.content()?, "Test data\n---\nTwo test data");
        assert_eq!(note.path(), path);

        Ok(())
    }

    fn without_properties<T>(file: T, text: &str) -> Result<(), T::Error>
    where
        T: Note<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        assert_eq!(file.properties().unwrap(), None);
        assert_eq!(file.content().unwrap(), text);

        Ok(())
    }

    fn invalid_yaml<T>(result: Result<T, T::Error>) -> Result<(), T::Error>
    where
        T: Note<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let error = result.err().unwrap();

        assert!(is_error::<serde_yml::Error>(error));
        Ok(())
    }

    fn invalid_format<T>(result: Result<T, T::Error>) -> Result<(), T::Error>
    where
        T: Note<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let error = result.err().unwrap();

        assert!(is_error::<parser::Error>(error));
        Ok(())
    }

    fn with_unicode<T>(file: T) -> Result<(), T::Error>
    where
        T: Note<Properties = DefaultProperties>,
    {
        let properties = file.properties()?.unwrap();

        assert_eq!(properties["data"], "💩");
        assert_eq!(file.content().unwrap(), "Super data 💩💩💩");

        Ok(())
    }

    fn space_with_properties<T>(file: T, content: &str) -> Result<(), T::Error>
    where
        T: Note<Properties = DefaultProperties>,
    {
        let properties = file.properties()?;

        assert_eq!(file.content().unwrap(), content);
        assert_eq!(properties, None);

        Ok(())
    }

    pub(crate) fn from_reader<T>() -> Result<(), T::Error>
    where
        T: NoteFromReader<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let mut reader = Cursor::new(TEST_DATA);
        let file = T::from_reader(&mut reader)?;

        test_data(file, None)?;
        Ok(())
    }

    pub(crate) fn from_reader_without_properties<T>() -> Result<(), T::Error>
    where
        T: NoteFromReader<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let test_data = "TEST_DATA";
        let file = T::from_reader(&mut Cursor::new(test_data))?;

        without_properties(file, test_data)?;
        Ok(())
    }

    pub(crate) fn from_reader_invalid_yaml<T>() -> Result<(), T::Error>
    where
        T: NoteFromReader<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let result = T::from_reader(&mut Cursor::new(BROKEN_DATA));

        invalid_yaml(result)?;
        Ok(())
    }

    pub(crate) fn from_reader_invalid_format<T>() -> Result<(), T::Error>
    where
        T: NoteFromReader<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let broken_data = "---\n";
        let result = T::from_reader(&mut Cursor::new(broken_data));

        invalid_format(result)?;
        Ok(())
    }

    pub(crate) fn from_reader_with_unicode<T>() -> Result<(), T::Error>
    where
        T: NoteFromReader<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let file = T::from_reader(&mut Cursor::new(UNICODE_DATA))?;

        with_unicode(file)?;
        Ok(())
    }

    pub(crate) fn from_reader_space_with_properties<T>() -> Result<(), T::Error>
    where
        T: NoteFromReader<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let file = T::from_reader(&mut Cursor::new(SPACE_DATA))?;

        space_with_properties(file, SPACE_DATA)?;
        Ok(())
    }

    pub(crate) fn from_string<T>() -> Result<(), T::Error>
    where
        T: NoteFromString<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let file = T::from_string(TEST_DATA)?;

        test_data(file, None)?;
        Ok(())
    }

    pub(crate) fn from_string_without_properties<T>() -> Result<(), T::Error>
    where
        T: NoteFromString<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let test_data = "TEST_DATA";
        let file = T::from_string(test_data)?;

        without_properties(file, test_data)?;
        Ok(())
    }

    pub(crate) fn from_string_with_invalid_yaml<T>() -> Result<(), T::Error>
    where
        T: NoteFromString<Properties = DefaultProperties>,
        T::Error: From<std::io::Error> + From<serde_yml::Error> + 'static,
    {
        let result = T::from_string(BROKEN_DATA);

        invalid_yaml(result)?;
        Ok(())
    }

    pub(crate) fn from_string_invalid_format<T>() -> Result<(), T::Error>
    where
        T: NoteFromString<Properties = DefaultProperties>,
        T::Error: From<std::io::Error> + From<parser::Error>,
    {
        let broken_data = "---\n";

        let result = T::from_string(broken_data);
        invalid_format(result)?;

        Ok(())
    }

    pub(crate) fn from_string_with_unicode<T>() -> Result<(), T::Error>
    where
        T: NoteFromString<Properties = DefaultProperties>,
    {
        let file = T::from_string(UNICODE_DATA)?;

        with_unicode(file)?;
        Ok(())
    }

    pub(crate) fn from_string_space_with_properties<T>() -> Result<(), T::Error>
    where
        T: NoteFromString<Properties = DefaultProperties>,
    {
        let file = T::from_string(SPACE_DATA)?;

        space_with_properties(file, SPACE_DATA)?;
        Ok(())
    }

    pub(crate) fn from_file<T>() -> Result<(), T::Error>
    where
        T: NoteFromFile<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(TEST_DATA.as_bytes()).unwrap();

        let file = T::from_file(temp_file.path()).unwrap();

        test_data(file, Some(temp_file.path().to_path_buf()))?;
        Ok(())
    }

    pub(crate) fn from_file_note_name<T>() -> Result<(), T::Error>
    where
        T: NoteFromFile<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"TEST_DATA").unwrap();

        let name_temp_file = temp_file
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let file = T::from_file(temp_file.path())?;

        assert_eq!(file.note_name(), Some(name_temp_file));
        Ok(())
    }

    pub(crate) fn from_file_without_properties<T>() -> Result<(), T::Error>
    where
        T: NoteFromFile<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let test_data = "TEST_DATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let file = T::from_file(test_file.path())?;

        without_properties(file, test_data)?;
        Ok(())
    }

    pub(crate) fn from_file_with_invalid_yaml<T>() -> Result<(), T::Error>
    where
        T: NoteFromFile<Properties = DefaultProperties>,
        T::Error: From<std::io::Error> + From<serde_yml::Error>,
    {
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(BROKEN_DATA.as_bytes()).unwrap();

        let result = T::from_file(test_file.path());

        invalid_yaml(result)?;
        Ok(())
    }

    pub(crate) fn from_file_invalid_format<T>() -> Result<(), T::Error>
    where
        T: NoteFromFile<Properties = DefaultProperties>,
        T::Error: From<std::io::Error> + From<parser::Error>,
    {
        let broken_data = "---\n";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(broken_data.as_bytes()).unwrap();

        let result = T::from_file(test_file.path());

        invalid_format(result)?;
        Ok(())
    }

    pub(crate) fn from_file_with_unicode<T>() -> Result<(), T::Error>
    where
        T: NoteFromFile<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(UNICODE_DATA.as_bytes()).unwrap();

        let file = T::from_file(test_file.path())?;

        with_unicode(file)?;
        Ok(())
    }

    pub(crate) fn from_file_space_with_properties<T>() -> Result<(), T::Error>
    where
        T: NoteFromFile<Properties = DefaultProperties>,
        T::Error: From<std::io::Error>,
    {
        let data = "  ---\ntest: test-data\n---\n";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(data.as_bytes()).unwrap();

        let file = T::from_file(test_file.path())?;

        space_with_properties(file, data)?;
        Ok(())
    }

    macro_rules! impl_all_tests_from_reader {
        ($impl_note:path) => {
            #[allow(unused_imports)]
            use $crate::note::note_read::tests::*;

            impl_test_for_note!(impl_from_reader, from_reader, $impl_note);

            impl_test_for_note!(
                impl_from_reader_without_properties,
                from_reader_without_properties,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_reader_with_invalid_yaml,
                from_reader_invalid_yaml,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_reader_invalid_format,
                from_reader_invalid_format,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_reader_with_unicode,
                from_reader_with_unicode,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_reader_space_with_properties,
                from_reader_space_with_properties,
                $impl_note
            );
        };
    }

    macro_rules! impl_all_tests_from_string {
        ($impl_note:path) => {
            #[allow(unused_imports)]
            use $crate::note::note_read::tests::*;

            impl_test_for_note!(impl_from_string, from_string, $impl_note);

            impl_test_for_note!(
                impl_from_string_without_properties,
                from_string_without_properties,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_string_with_invalid_yaml,
                from_string_with_invalid_yaml,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_string_invalid_format,
                from_string_invalid_format,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_string_with_unicode,
                from_string_with_unicode,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_string_space_with_properties,
                from_string_space_with_properties,
                $impl_note
            );
        };
    }

    macro_rules! impl_all_tests_from_file {
        ($impl_note:path) => {
            #[allow(unused_imports)]
            use $crate::note::impl_tests::*;

            impl_test_for_note!(impl_from_file, from_file, $impl_note);
            impl_test_for_note!(impl_from_file_note_name, from_file_note_name, $impl_note);

            impl_test_for_note!(
                impl_from_file_without_properties,
                from_file_without_properties,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_file_with_invalid_yaml,
                from_file_with_invalid_yaml,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_file_invalid_format,
                from_file_invalid_format,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_file_with_unicode,
                from_file_with_unicode,
                $impl_note
            );
            impl_test_for_note!(
                impl_from_file_space_with_properties,
                from_file_space_with_properties,
                $impl_note
            );
        };
    }

    pub(crate) use impl_all_tests_from_file;
    pub(crate) use impl_all_tests_from_reader;
    pub(crate) use impl_all_tests_from_string;
}
