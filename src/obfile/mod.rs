//! Represents an Obsidian note file with frontmatter properties and content

pub mod obfile_in_memory;
pub mod obfile_on_disk;

use crate::error::Error;
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

type DefaultProperties = HashMap<String, serde_yml::Value>;

/// Represents an Obsidian note file with frontmatter properties and content
///
/// This trait provides a standardized interface for working with Obsidian markdown files,
/// handling frontmatter parsing, content extraction, and file operations.
///
/// # Type Parameters
/// - `T`: Frontmatter properties type
///
/// # Example
/// ```no_run
/// use obsidian_parser::prelude::*;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Default, Clone)]
/// struct NoteProperties {
///     topic: String,
///     created: String,
/// }
///
/// let note: ObFileInMemory<NoteProperties> = ObFile::from_file("note.md").unwrap();
/// println!("Note topic: {}", note.properties().unwrap().topic);
/// ```
pub trait ObFile<T = DefaultProperties>: Sized
where
    T: DeserializeOwned + Clone,
{
    /// Returns the main content body of the note (excluding frontmatter)
    ///
    /// # Implementation Notes
    /// - Strips YAML frontmatter if present
    /// - Preserves original formatting and whitespace
    fn content(&self) -> String;

    /// Returns the source file path if available
    ///
    /// Returns [`None`] for in-memory notes without physical storage
    fn path(&self) -> Option<PathBuf>;

    /// Returns the parsed properties of frontmatter
    ///
    /// Returns [`None`] if the note has no properties
    fn properties(&self) -> Option<T>;

    /// Get note name
    fn note_name(&self) -> Option<String> {
        if let Some(path) = self.path() {
            if let Some(name) = path.file_stem() {
                return Some(name.to_string_lossy().to_string());
            }
        }

        None
    }

    /// Parses an Obsidian note from a string
    ///
    /// # Arguments
    /// - `raw_text`: Raw markdown content with optional YAML frontmatter
    /// - `path`: Optional source path for reference
    ///
    /// # Errors
    /// - [`Error::InvalidFormat`] for malformed frontmatter
    /// - [`Error::Yaml`] for invalid YAML syntax
    fn from_string<P: AsRef<Path>>(raw_text: &str, path: Option<P>) -> Result<Self, Error>;

    /// Parses an Obsidian note from a file
    ///
    /// # Arguments
    /// - `path`: Filesystem path to markdown file
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    /// - [`Error::FromUtf8`] for non-UTF8 content
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::trace!("Parse obsidian file from file: {}", path_buf.display());

        let data = std::fs::read(path)?;
        let text = String::from_utf8(data)?;

        Self::from_string(&text, Some(path_buf))
    }
}

/// Default implementation using [`HashMap`] for properties
///
/// Automatically implemented for all `ObFile<HashMap<..>>` types.
/// Provides identical interface with explicitly named methods.
pub trait ObFileDefault: ObFile<DefaultProperties> {
    /// Same as [`ObFile::from_string`] with default properties type
    ///
    /// # Errors
    /// - [`Error::InvalidFormat`] for malformed frontmatter
    /// - [`Error::Yaml`] for invalid YAML syntax
    fn from_string_default<P: AsRef<Path>>(text: &str, path: Option<P>) -> Result<Self, Error>;

    /// Same as [`ObFile::from_file`] with default properties type
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    /// - [`Error::FromUtf8`] for non-UTF8 content
    fn from_file_default<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
}

impl<T> ObFileDefault for T
where
    T: ObFile<DefaultProperties>,
{
    fn from_string_default<P: AsRef<Path>>(text: &str, path: Option<P>) -> Result<Self, Error> {
        Self::from_string(text, path)
    }

    fn from_file_default<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_file(path)
    }
}

#[derive(Debug, PartialEq)]
enum ResultParse<'a> {
    WithProperties {
        content: &'a str,
        properties: &'a str,
    },
    WithoutProperties,
}

fn parse_obfile(raw_text: &str) -> Result<ResultParse, Error> {
    let mut lines = raw_text.lines();
    if lines.next().unwrap_or_default().trim_end() == "---" {
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
}

#[cfg(test)]
pub(crate) mod impl_tests {
    use super::*;
    use crate::test_utils::init_test_logger;
    use serde::Deserialize;
    use std::io::Write;
    use tempfile::NamedTempFile;

    pub(crate) static TEST_DATA: &str = "---\n\
topic: life\n\
created: 2025-03-16\n\
---\n\
Test data\n\
---\n\
Two test data";

    #[derive(Debug, Deserialize, Default, PartialEq, Clone)]
    pub(crate) struct TestProperties {
        pub(crate) topic: String,
        pub(crate) created: String,
    }

    pub(crate) fn from_string<T: ObFile>() -> Result<(), Error> {
        init_test_logger();
        let file = T::from_string(TEST_DATA, None::<&str>)?;
        let properties = file.properties().unwrap();

        assert_eq!(properties["topic"], "life");
        assert_eq!(properties["created"], "2025-03-16");
        assert_eq!(file.content(), "Test data\n---\nTwo test data");
        Ok(())
    }

    pub(crate) fn from_string_note_name<T: ObFile>() -> Result<(), Error> {
        init_test_logger();
        let file1 = T::from_string(TEST_DATA, None::<&str>)?;
        let file2 = T::from_string(TEST_DATA, Some("Super node.md"))?;

        assert_eq!(file1.note_name(), None);
        assert_eq!(file2.note_name(), Some("Super node".to_string()));
        Ok(())
    }

    pub(crate) fn from_string_without_properties<T: ObFile>() -> Result<(), Error> {
        init_test_logger();
        let test_data = "TEST_DATA";
        let file = T::from_string(test_data, None::<&str>)?;

        assert_eq!(file.properties(), None);
        assert_eq!(file.content(), test_data);
        Ok(())
    }

    pub(crate) fn from_string_with_invalid_yaml<T: ObFile>() -> Result<(), Error> {
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

    pub(crate) fn from_string_invalid_format<T: ObFile>() -> Result<(), Error> {
        init_test_logger();
        let broken_data = "---\n";

        assert!(matches!(
            T::from_string(broken_data, None::<&str>),
            Err(Error::InvalidFormat)
        ));
        Ok(())
    }

    pub(crate) fn from_string_with_unicode<T: ObFile>() -> Result<(), Error> {
        init_test_logger();
        let data = "---\ndata: 💩\n---\nSuper data 💩💩💩";
        let file = T::from_string(data, None::<&str>)?;
        let properties = file.properties().unwrap();

        assert_eq!(properties["data"], "💩");
        assert_eq!(file.content(), "Super data 💩💩💩");
        Ok(())
    }

    pub(crate) fn from_string_space_with_properties<T: ObFile>() -> Result<(), Error> {
        init_test_logger();
        let data = "  ---\ntest: test-data\n---\n";
        let file = T::from_string(data, None::<&str>)?;
        let properties = file.properties();

        assert_eq!(file.content(), data);
        assert_eq!(properties, None);
        Ok(())
    }

    pub(crate) fn from_file<T: ObFile>() -> Result<(), Error> {
        init_test_logger();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"TEST_DATA").unwrap();

        let file = T::from_file(temp_file.path()).unwrap();
        assert_eq!(file.content(), "TEST_DATA");
        assert_eq!(file.path().unwrap(), temp_file.path());
        assert_eq!(file.properties(), None);
        Ok(())
    }

    pub(crate) fn from_file_note_name<T: ObFile>() -> Result<(), Error> {
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

    pub(crate) fn from_file_without_properties<T: ObFile>() -> Result<(), Error> {
        init_test_logger();
        let test_data = "TEST_DATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let file = T::from_file(test_file.path())?;

        assert_eq!(file.properties(), None);
        assert_eq!(file.content(), test_data);
        Ok(())
    }

    pub(crate) fn from_file_with_invalid_yaml<T: ObFile>() -> Result<(), Error> {
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

    pub(crate) fn from_file_invalid_format<T: ObFile>() -> Result<(), Error> {
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

    pub(crate) fn from_file_with_unicode<T: ObFile>() -> Result<(), Error> {
        init_test_logger();
        let data = "---\ndata: 💩\n---\nSuper data 💩💩💩";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(data.as_bytes()).unwrap();

        let file = T::from_file(test_file.path())?;
        let properties = file.properties();

        assert_eq!(properties.unwrap()["data"], "💩");
        assert_eq!(file.content(), "Super data 💩💩💩");
        Ok(())
    }

    pub(crate) fn from_file_space_with_properties<T: ObFile>() -> Result<(), Error> {
        init_test_logger();
        let data = "  ---\ntest: test-data\n---\n";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(data.as_bytes()).unwrap();

        let file = T::from_string(data, None::<&str>)?;

        assert_eq!(file.content(), data);
        assert_eq!(file.properties(), None);
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
            use crate::obfile::impl_tests::*;

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
            use crate::obfile::impl_tests::*;

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

    pub(crate) use impl_all_tests_from_file;
    pub(crate) use impl_all_tests_from_string;
}
