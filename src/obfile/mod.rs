pub mod obfile_in_memory;
pub mod obfile_on_disk;

use crate::error::Error;
use regex::{Regex, RegexBuilder};
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::LazyLock,
};

/// Represents an Obsidian note file with frontmatter properties and content
///
/// This trait provides a standardized interface for working with Obsidian markdown files,
/// handling frontmatter parsing, content extraction, and file operations.
///
/// # Type Parameters
/// - `T`: Frontmatter properties type (must implement `DeserializeOwned + Default + Clone + Send`)
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
/// println!("Note topic: {}", note.properties().topic);
/// ```
pub trait ObFile<T = HashMap<String, serde_yaml::Value>>: Sized
where
    T: DeserializeOwned + Default + Clone + Send,
{
    /// Returns the main content body of the note (excluding frontmatter)
    ///
    /// # Implementation Notes
    /// - Strips YAML frontmatter if present
    /// - Preserves original formatting and whitespace
    fn content(&self) -> String;

    /// Returns the source file path if available
    ///
    /// Returns `None` for in-memory notes without physical storage
    fn path(&self) -> Option<PathBuf>;

    /// Returns parsed frontmatter properties
    ///
    /// # Behavior
    /// - Returns default-initialized properties if frontmatter is missing/invalid
    /// - Automatically handles YAML deserialization
    fn properties(&self) -> T;

    /// Parses an Obsidian note from a string
    ///
    /// # Arguments
    /// - `raw_text`: Raw markdown content with optional YAML frontmatter
    /// - `path`: Optional source path for reference
    ///
    /// # Errors
    /// - `Error::InvalidFormat` for malformed frontmatter
    /// - `Error::Yaml` for invalid YAML syntax
    fn from_string<P: AsRef<Path>>(raw_text: &str, path: Option<P>) -> Result<Self, Error>;

    /// Parses an Obsidian note from a file
    ///
    /// # Arguments
    /// - `path`: Filesystem path to markdown file
    ///
    /// # Errors
    /// - `Error::Io` for filesystem errors
    /// - `Error::FromUtf8` for non-UTF8 content
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::trace!("Parse obsidian file from file: {}", path_buf.display());
        let data = std::fs::read(path)?;
        let text = String::from_utf8(data)?;

        Self::from_string(&text, Some(path_buf))
    }
}

/// Default implementation using `HashMap` for properties
///
/// Automatically implemented for all `ObFile<HashMap<..>>` types.
/// Provides identical interface with explicitly named methods.
pub trait ObFileDefault: ObFile<HashMap<String, serde_yaml::Value>> {
    /// Same as `ObFile::from_string` with default properties type
    fn from_string_default<P: AsRef<Path>>(text: &str, path: Option<P>) -> Result<Self, Error>;

    /// Same as `ObFile::from_file` with default properties type
    fn from_file_default<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
}

impl<T> ObFileDefault for T
where
    T: ObFile<HashMap<String, serde_yaml::Value>>,
{
    fn from_string_default<P: AsRef<Path>>(text: &str, path: Option<P>) -> Result<Self, Error> {
        Self::from_string(text, path)
    }

    fn from_file_default<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_file(path)
    }
}

/// Helper function with enhanced logging
fn parse_obfile(raw_text: &str) -> (bool, Vec<&str>) {
    static PROPERTIES_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        RegexBuilder::new(r"^---\s*$")
            .multi_line(true)
            .unicode(false)
            .build()
            .unwrap()
    });

    #[cfg(feature = "logging")]
    log::trace!("Parse obsidian file from string");

    let parts: Vec<_> = PROPERTIES_REGEX.splitn(raw_text, 3).collect();
    let valid_properties = raw_text.starts_with("---");

    (valid_properties, parts)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
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
        let file = T::from_string(TEST_DATA, None::<&str>)?;
        let properties = file.properties();

        assert_eq!(properties["topic"], "life");
        assert_eq!(properties["created"], "2025-03-16");
        assert_eq!(file.content(), "Test data\n---\nTwo test data");
        Ok(())
    }

    pub(crate) fn from_string_without_properties<T: ObFile>() -> Result<(), Error> {
        let test_data = "TEST_DATA";
        let file = T::from_string(test_data, None::<&str>)?;
        let properties = file.properties();

        assert_eq!(properties.len(), 0);
        assert_eq!(file.content(), test_data);
        Ok(())
    }

    pub(crate) fn from_string_with_invalid_yaml<T: ObFile>() -> Result<(), Error> {
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
        let broken_data = "---\n";

        assert!(matches!(
            T::from_string(broken_data, None::<&str>),
            Err(Error::InvalidFormat)
        ));
        Ok(())
    }

    pub(crate) fn from_string_with_unicode<T: ObFile>() -> Result<(), Error> {
        let data = "---\ndata: 💩\n---\nSuper data 💩💩💩";
        let file = T::from_string(data, None::<&str>)?;
        let properties = file.properties();

        assert_eq!(properties["data"], "💩");
        assert_eq!(file.content(), "Super data 💩💩💩");
        Ok(())
    }

    pub(crate) fn space_with_properties<T: ObFile>() -> Result<(), Error> {
        let data = "  ---\ntest: test-data\n---\n";
        let file = T::from_string(data, None::<&str>)?;
        let properties = file.properties();

        assert_eq!(file.content(), data);
        assert_eq!(properties.len(), 0);
        Ok(())
    }

    pub(crate) fn from_file<T: ObFile>() -> Result<(), Error> {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"TEST_DATA").unwrap();

        let file = T::from_file(temp_file.path()).unwrap();
        assert_eq!(file.content(), "TEST_DATA");
        assert_eq!(file.path().unwrap(), temp_file.path());
        assert_eq!(file.properties().len(), 0);
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

    macro_rules! impl_all_test_for_obfile {
        ($impl_obfile:path) => {
            use crate::obfile::tests::*;
            impl_test_for_obfile!(impl_from_string, from_string, $impl_obfile);

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
                impl_space_with_properties,
                space_with_properties,
                $impl_obfile
            );

            impl_test_for_obfile!(impl_from_file, from_file, $impl_obfile);
        };
    }

    pub(crate) use impl_all_test_for_obfile;
}
