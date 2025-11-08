//! Represents an Obsidian note file with frontmatter properties and content

pub mod obfile_default;
pub mod obfile_in_memory;
pub mod obfile_on_disk;
pub mod obfile_read;
pub mod obfile_write;

use crate::error::Error;
use std::{borrow::Cow, collections::HashMap, fs::OpenOptions, path::Path};

pub use obfile_default::ObFileDefault;
pub use obfile_read::ObFileRead;
pub use obfile_write::ObFileWrite;

pub(crate) type DefaultProperties = HashMap<String, serde_yml::Value>;

/// Represents an Obsidian note file with frontmatter properties and content
///
/// This trait provides a standardized interface for working with Obsidian markdown files,
/// handling frontmatter parsing, content extraction, and file operations.
///
/// # Example
/// ```no_run
/// use obsidian_parser::prelude::*;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Clone)]
/// struct NoteProperties {
///     topic: String,
///     created: String,
/// }
///
/// let note: ObFileInMemory<NoteProperties> = ObFileRead::from_file("note.md").unwrap();
/// let properties = note.properties().unwrap().unwrap();
/// println!("Note topic: {}", properties.topic);
/// ```
///
/// # Other
/// * To open and read [`ObFile`] to a file, use the [`ObFileRead`] trait.
/// * To write and modify [`ObFile`] to a file, use the [`ObFileWrite`] trait.
pub trait ObFile: Sized {
    /// Frontmatter properties type
    type Properties: Clone;

    /// Returns the parsed properties of frontmatter
    ///
    /// Returns [`None`] if the note has no properties
    ///
    /// # Errors
    /// Usually errors are related to [`Error::Io`]
    fn properties(&self) -> Result<Option<Cow<'_, Self::Properties>>, Error>;

    /// Returns the main content body of the note (excluding frontmatter)
    ///
    /// # Implementation Notes
    /// - Strips YAML frontmatter if present
    /// - Preserves original formatting and whitespace
    ///
    /// # Errors
    /// Usually errors are related to [`Error::Io`]
    fn content(&self) -> Result<Cow<'_, str>, Error>;

    /// Returns the source file path if available
    ///
    /// Returns [`None`] for in-memory notes without physical storage
    fn path(&self) -> Option<Cow<'_, Path>>;

    /// Get note name
    fn note_name(&self) -> Option<String> {
        self.path().as_ref().map(|path| {
            path.file_stem()
                .expect("Path is not file")
                .to_string_lossy()
                .to_string()
        })
    }
}

/// Parses Obsidian-style links in note content
///
/// Handles all link formats:
/// - `[[Note]]`
/// - `[[Note|Alias]]`
/// - `[[Note^block]]`
/// - `[[Note#heading]]`
/// - `[[Note#heading|Alias]]`
///
/// # Example
/// ```
/// # use obsidian_parser::obfile::parse_links;
/// let content = "[[Physics]] and [[Math|Mathematics]]";
/// let links: Vec<_> = parse_links(content).collect();
/// assert_eq!(links, vec!["Physics", "Math"]);
/// ```
pub fn parse_links(text: &str) -> impl Iterator<Item = &str> {
    text.match_indices("[[").filter_map(move |(start_pos, _)| {
        let end_pos = text[start_pos + 2..].find("]]")?;
        let inner = &text[start_pos + 2..start_pos + 2 + end_pos];

        let note_name = inner
            .split('#')
            .next()?
            .split('^')
            .next()?
            .split('|')
            .next()?
            .trim();

        Some(note_name)
    })
}

#[derive(Debug, PartialEq)]
enum ResultParse<'a> {
    WithProperties {
        content: &'a str,
        properties: &'a str,
    },
    WithoutProperties,
}

fn parse_obfile(raw_text: &str) -> Result<ResultParse<'_>, Error> {
    let have_start_properties = raw_text
        .lines()
        .next()
        .is_some_and(|line| line.trim_end() == "---");

    if have_start_properties {
        let closed = raw_text["---".len()..]
            .find("---")
            .ok_or(Error::InvalidFormat)?;

        return Ok(ResultParse::WithProperties {
            content: raw_text[(closed + 2 * "...".len())..].trim(),
            properties: raw_text["...".len()..(closed + "...".len())].trim(),
        });
    }

    Ok(ResultParse::WithoutProperties)
}

#[cfg(test)]
mod tests {
    use super::{ResultParse, parse_obfile};
    use crate::test_utils::init_test_logger;

    #[test]
    fn parse_obfile_without_properties() {
        init_test_logger();
        let test_data = "test_data";
        let result = parse_obfile(test_data).unwrap();

        assert_eq!(result, ResultParse::WithoutProperties);
    }

    #[test]
    fn parse_obfile_with_properties() {
        init_test_logger();
        let test_data = "---\nproperties data\n---\ntest data";
        let result = parse_obfile(test_data).unwrap();

        assert_eq!(
            result,
            ResultParse::WithProperties {
                content: "test data",
                properties: "properties data"
            }
        );
    }

    #[test]
    fn parse_obfile_without_properties_but_with_closed() {
        init_test_logger();
        let test_data1 = "test_data---";
        let test_data2 = "test_data\n---\n";

        let result1 = parse_obfile(test_data1).unwrap();
        let result2 = parse_obfile(test_data2).unwrap();

        assert_eq!(result1, ResultParse::WithoutProperties);
        assert_eq!(result2, ResultParse::WithoutProperties);
    }

    #[test]
    #[should_panic]
    fn parse_obfile_with_properties_but_without_closed() {
        init_test_logger();
        let test_data = "---\nproperties data\ntest data";
        let _ = parse_obfile(test_data).unwrap();
    }

    #[test]
    fn parse_obfile_with_() {
        init_test_logger();
        let test_data = "---properties data";

        let result = parse_obfile(test_data).unwrap();
        assert_eq!(result, ResultParse::WithoutProperties);
    }

    #[test]
    fn parse_obfile_without_properties_but_with_spaces() {
        init_test_logger();
        let test_data = "   ---\ndata";

        let result = parse_obfile(test_data).unwrap();
        assert_eq!(result, ResultParse::WithoutProperties);
    }

    #[test]
    fn parse_obfile_with_properties_but_check_trim_end() {
        init_test_logger();
        let test_data = "---\r\nproperties data\r\n---\r   \ntest data";
        let result = parse_obfile(test_data).unwrap();

        assert_eq!(
            result,
            ResultParse::WithProperties {
                content: "test data",
                properties: "properties data"
            }
        );
    }

    #[test]
    fn test_parse_links() {
        init_test_logger();
        let test_data =
            "[[Note]] [[Note|Alias]] [[Note^block]] [[Note#Heading|Alias]] [[Note^block|Alias]]";

        let ds: Vec<_> = super::parse_links(test_data).collect();

        assert!(ds.iter().all(|x| *x == "Note"))
    }
}

#[cfg(test)]
pub(crate) mod impl_tests {
    use super::*;
    use crate::test_utils::init_test_logger;
    use std::io::Write;
    use tempfile::NamedTempFile;

    pub(crate) static TEST_DATA: &str = "---\n\
topic: life\n\
created: 2025-03-16\n\
---\n\
Test data\n\
---\n\
Two test data";

    pub(crate) fn from_string<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let file = T::from_string(TEST_DATA, None::<&str>)?;
        let properties = file.properties().unwrap().unwrap();

        assert_eq!(properties["topic"], "life");
        assert_eq!(properties["created"], "2025-03-16");
        assert_eq!(file.content().unwrap(), "Test data\n---\nTwo test data");
        Ok(())
    }

    pub(crate) fn from_string_note_name<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let file1 = T::from_string(TEST_DATA, None::<&str>)?;
        let file2 = T::from_string(TEST_DATA, Some("Super node.md"))?;

        assert_eq!(file1.note_name(), None);
        assert_eq!(file2.note_name(), Some("Super node".to_string()));
        Ok(())
    }

    pub(crate) fn from_string_without_properties<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let test_data = "TEST_DATA";
        let file = T::from_string(test_data, None::<&str>)?;

        assert_eq!(file.properties().unwrap(), None);
        assert_eq!(file.content().unwrap(), test_data);
        Ok(())
    }

    pub(crate) fn from_string_with_invalid_yaml<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let broken_data = "---\n\
    asdfv:--fs\n\
    sfsf\n\
    ---\n\
    TestData";

        assert!(matches!(
            T::from_string(broken_data, None::<&str>),
            Err(Error::Yaml(_))
        ));
        Ok(())
    }

    pub(crate) fn from_string_invalid_format<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let broken_data = "---\n";

        assert!(matches!(
            T::from_string(broken_data, None::<&str>),
            Err(Error::InvalidFormat)
        ));
        Ok(())
    }

    pub(crate) fn from_string_with_unicode<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let data = "---\ndata: 💩\n---\nSuper data 💩💩💩";
        let file = T::from_string(data, None::<&str>)?;
        let properties = file.properties().unwrap().unwrap();

        assert_eq!(properties["data"], "💩");
        assert_eq!(file.content().unwrap(), "Super data 💩💩💩");
        Ok(())
    }

    pub(crate) fn from_string_space_with_properties<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let data = "  ---\ntest: test-data\n---\n";
        let file = T::from_string(data, None::<&str>)?;
        let properties = file.properties().unwrap();

        assert_eq!(file.content().unwrap(), data);
        assert_eq!(properties, None);
        Ok(())
    }

    pub(crate) fn from_file<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"TEST_DATA").unwrap();

        let file = T::from_file(temp_file.path()).unwrap();
        assert_eq!(file.content().unwrap(), "TEST_DATA");
        assert_eq!(file.path().unwrap(), temp_file.path());
        assert_eq!(file.properties().unwrap(), None);
        Ok(())
    }

    pub(crate) fn from_file_note_name<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"TEST_DATA").unwrap();

        let name_temp_file = temp_file
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let file = T::from_file(temp_file.path()).unwrap();

        assert_eq!(file.note_name(), Some(name_temp_file));
        Ok(())
    }

    pub(crate) fn from_file_without_properties<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let test_data = "TEST_DATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let file = T::from_file(test_file.path())?;

        assert_eq!(file.properties().unwrap(), None);
        assert_eq!(file.content().unwrap(), test_data);
        Ok(())
    }

    pub(crate) fn from_file_with_invalid_yaml<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let broken_data = "---\n\
    asdfv:--fs\n\
    sfsf\n\
    ---\n\
    TestData";

        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(broken_data.as_bytes()).unwrap();

        assert!(matches!(
            T::from_file(test_file.path()),
            Err(Error::Yaml(_))
        ));
        Ok(())
    }

    pub(crate) fn from_file_invalid_format<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let broken_data = "---\n";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(broken_data.as_bytes()).unwrap();

        assert!(matches!(
            T::from_file(test_file.path()),
            Err(Error::InvalidFormat)
        ));
        Ok(())
    }

    pub(crate) fn from_file_with_unicode<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let data = "---\ndata: 💩\n---\nSuper data 💩💩💩";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(data.as_bytes()).unwrap();

        let file = T::from_file(test_file.path())?;
        let properties = file.properties().unwrap().unwrap();

        assert_eq!(properties["data"], "💩");
        assert_eq!(file.content().unwrap(), "Super data 💩💩💩");
        Ok(())
    }

    pub(crate) fn from_file_space_with_properties<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties>,
    {
        init_test_logger();
        let data = "  ---\ntest: test-data\n---\n";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(data.as_bytes()).unwrap();

        let file = T::from_string(data, None::<&str>)?;

        assert_eq!(file.content().unwrap(), data);
        assert_eq!(file.properties().unwrap(), None);
        Ok(())
    }

    pub(crate) fn flush_properties<T>() -> Result<(), Error>
    where
        T: ObFileWrite<Properties = DefaultProperties> + ObFileRead<Properties = DefaultProperties>,
    {
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(TEST_DATA.as_bytes()).unwrap();

        let file = T::from_file_default(test_file.path())?;
        let open_options = OpenOptions::new().write(true).create(false).clone();
        file.flush_properties(&open_options)?;
        drop(file);

        let file = T::from_file_default(test_file.path())?;

        let properties = file.properties()?.unwrap();
        assert_eq!(properties["topic"], "life");
        assert_eq!(properties["created"], "2025-03-16");
        assert_eq!(file.content().unwrap(), "Test data\n---\nTwo test data");

        Ok(())
    }

    pub(crate) fn flush_content<T>() -> Result<(), Error>
    where
        T: ObFileWrite<Properties = DefaultProperties> + ObFileRead<Properties = DefaultProperties>,
    {
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(TEST_DATA.as_bytes()).unwrap();

        let file = T::from_file_default(test_file.path())?;
        let open_options = OpenOptions::new().write(true).create(false).clone();
        file.flush_content(&open_options)?;
        drop(file);

        let file = T::from_file_default(test_file.path())?;
        let properties = file.properties()?.unwrap();
        assert_eq!(properties["topic"], "life");
        assert_eq!(properties["created"], "2025-03-16");
        assert_eq!(file.content().unwrap(), "Test data\n---\nTwo test data");

        Ok(())
    }

    pub(crate) fn flush<T>() -> Result<(), Error>
    where
        T: ObFileRead<Properties = DefaultProperties> + ObFileWrite<Properties = DefaultProperties>,
    {
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(TEST_DATA.as_bytes()).unwrap();

        let file = T::from_file_default(test_file.path())?;
        let open_options = OpenOptions::new().write(true).create(false).clone();
        file.flush(&open_options)?;
        drop(file);

        let file = T::from_file_default(test_file.path())?;
        let properties = file.properties()?.unwrap();
        assert_eq!(properties["topic"], "life");
        assert_eq!(properties["created"], "2025-03-16");
        assert_eq!(file.content().unwrap(), "Test data\n---\nTwo test data");

        Ok(())
    }

    macro_rules! impl_test_for_obfile {
        ($name_test:ident, $fn_test:ident, $impl_obfile:path) => {
            #[test]
            fn $name_test() {
                $fn_test::<$impl_obfile>().unwrap();
            }
        };
    }

    pub(crate) use impl_test_for_obfile;

    macro_rules! impl_all_tests_from_string {
        ($impl_obfile:path) => {
            #[allow(unused_imports)]
            use $crate::obfile::impl_tests::*;

            impl_test_for_obfile!(impl_from_string, from_string, $impl_obfile);

            impl_test_for_obfile!(
                impl_from_string_note_name,
                from_string_note_name,
                $impl_obfile
            );
            impl_test_for_obfile!(
                impl_from_string_without_properties,
                from_string_without_properties,
                $impl_obfile
            );
            impl_test_for_obfile!(
                impl_from_string_with_invalid_yaml,
                from_string_with_invalid_yaml,
                $impl_obfile
            );
            impl_test_for_obfile!(
                impl_from_string_invalid_format,
                from_string_invalid_format,
                $impl_obfile
            );
            impl_test_for_obfile!(
                impl_from_string_with_unicode,
                from_string_with_unicode,
                $impl_obfile
            );
            impl_test_for_obfile!(
                impl_from_string_space_with_properties,
                from_string_space_with_properties,
                $impl_obfile
            );
        };
    }

    macro_rules! impl_all_tests_from_file {
        ($impl_obfile:path) => {
            #[allow(unused_imports)]
            use $crate::obfile::impl_tests::*;

            impl_test_for_obfile!(impl_from_file, from_file, $impl_obfile);
            impl_test_for_obfile!(impl_from_file_note_name, from_file_note_name, $impl_obfile);

            impl_test_for_obfile!(
                impl_from_file_without_properties,
                from_file_without_properties,
                $impl_obfile
            );
            impl_test_for_obfile!(
                impl_from_file_with_invalid_yaml,
                from_file_with_invalid_yaml,
                $impl_obfile
            );
            impl_test_for_obfile!(
                impl_from_file_invalid_format,
                from_file_invalid_format,
                $impl_obfile
            );
            impl_test_for_obfile!(
                impl_from_file_with_unicode,
                from_file_with_unicode,
                $impl_obfile
            );
            impl_test_for_obfile!(
                impl_from_file_space_with_properties,
                from_file_space_with_properties,
                $impl_obfile
            );
        };
    }

    macro_rules! impl_all_tests_flush {
        ($impl_obfile:path) => {
            #[allow(unused_imports)]
            use $crate::obfile::impl_tests::*;

            impl_test_for_obfile!(impl_flush, flush, $impl_obfile);
            impl_test_for_obfile!(impl_flush_content, flush_content, $impl_obfile);
            impl_test_for_obfile!(impl_flush_properties, flush_properties, $impl_obfile);
        };
    }

    pub(crate) use impl_all_tests_flush;
    pub(crate) use impl_all_tests_from_file;
    pub(crate) use impl_all_tests_from_string;
}
