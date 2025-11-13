//! On-disk representation of an Obsidian note file

use crate::obfile::parser::{self, ResultParse, parse_obfile};
use crate::obfile::{DefaultProperties, ObFile, ObFileRead};
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::io::Read;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

/// On-disk representation of an Obsidian note file
///
/// Optimized for vault operations where:
/// 1. Memory efficiency is critical (large vaults)
/// 2. Storage is fast (SSD/NVMe)
/// 3. Content is accessed infrequently
///
/// # Tradeoffs vs `ObFileInMemory`
/// | Characteristic       | [`ObFileOnDisk`]        | [`ObFileInMemory`]          |
/// |----------------------|-------------------------|-----------------------------|
/// | Memory usage         | **Minimal** (~24 bytes) | High (content + properties) |
/// | File access          | On-demand               | Preloaded                   |
/// | Best for             | SSD-based vaults        | RAM-heavy workflows         |
/// | Content access cost  | Disk read               | Zero cost                   |
///
/// # Recommendation
/// Prefer `ObFileOnDisk` for vault operations on modern hardware. The combination of
/// SSD speeds and Rust's efficient I/O makes this implementation ideal for:
/// - Large vaults (1000+ files)
/// - Graph processing
///
/// # Warning
/// Requires **persistent file access** throughout the object's lifetime
///
/// [`ObFileInMemory`]: crate::obfile::obfile_in_memory::ObFileInMemory
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ObFileOnDisk<T = DefaultProperties>
where
    T: Clone + DeserializeOwned,
{
    /// Absolute path to the source Markdown file
    path: PathBuf,

    phantom: PhantomData<T>,
}

#[derive(Debug, Error)]
pub enum Error {
    /// I/O operation failed (file reading, directory traversal, etc.)
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    /// Invalid frontmatter format detected
    ///
    /// Occurs when:
    /// - Frontmatter delimiters are incomplete (`---` missing)
    /// - Content between delimiters is empty
    ///
    /// # Example
    /// Parsing a file with malformed frontmatter:
    /// ```text
    /// ---
    /// incomplete yaml
    /// // Missing closing ---
    /// ```
    #[error("Invalid frontmatter format")]
    InvalidFormat(#[from] parser::Error),

    /// YAML parsing error in frontmatter properties
    ///
    /// # Example
    /// Parsing invalid YAML syntax:
    /// ```text
    /// ---
    /// key: @invalid_value
    /// ---
    /// ```
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yml::Error),

    /// Expected a file path
    ///
    /// # Example
    /// ```no_run
    /// use obsidian_parser::prelude::*;
    ///
    /// // Will fail if passed a directory path
    /// ObFileOnDisk::from_file_default("/home/test");
    /// ```
    #[error("Path: `{0}` is not a directory")]
    IsNotFile(PathBuf),
}

impl<T> ObFile for ObFileOnDisk<T>
where
    T: DeserializeOwned + Clone,
{
    type Properties = T;
    type Error = self::Error;

    /// Parses YAML frontmatter directly from disk
    ///
    /// # Errors
    /// - If properties can't be deserialized
    /// - If file doesn't exist
    /// - On filesystem errors
    fn properties(&self) -> Result<Option<Cow<'_, T>>, Error> {
        #[cfg(feature = "logging")]
        log::trace!("Get properties from file: `{}`", self.path.display());

        let data = std::fs::read(&self.path)?;

        // SAFETY: Notes files in Obsidian (`*.md`) ensure that the file is encoded in UTF-8
        let raw_text = unsafe { String::from_utf8_unchecked(data) };

        let result = match parse_obfile(&raw_text)? {
            ResultParse::WithProperties {
                content: _,
                properties,
            } => {
                #[cfg(feature = "logging")]
                log::trace!("Frontmatter detected, parsing properties");

                Some(Cow::Owned(serde_yml::from_str(properties)?))
            }
            ResultParse::WithoutProperties => {
                #[cfg(feature = "logging")]
                log::trace!("No frontmatter found, storing raw content");

                None
            }
        };

        Ok(result)
    }

    /// Returns the note's content body (without frontmatter)
    ///
    /// # Errors
    /// - If file doesn't exist
    /// - On filesystem errors
    ///
    /// # Performance
    /// Performs disk read on every call. Suitable for:
    /// - Single-pass processing (link extraction, analysis)
    /// - Large files where in-memory storage is prohibitive
    ///
    /// For repeated access, consider caching or [`ObFileInMemory`](crate::obfile::obfile_in_memory::ObFileInMemory).
    fn content(&self) -> Result<Cow<'_, str>, Error> {
        #[cfg(feature = "logging")]
        log::trace!("Get content from file: `{}`", self.path.display());

        let data = std::fs::read(&self.path)?;

        // SAFETY: Notes files in Obsidian (`*.md`) ensure that the file is encoded in UTF-8
        let raw_text = unsafe { String::from_utf8_unchecked(data) };

        let result = match parse_obfile(&raw_text)? {
            ResultParse::WithProperties {
                content,
                properties: _,
            } => {
                #[cfg(feature = "logging")]
                log::trace!("Frontmatter detected, parsing properties");

                content.to_string()
            }
            ResultParse::WithoutProperties => {
                #[cfg(feature = "logging")]
                log::trace!("No frontmatter found, storing raw content");

                raw_text
            }
        };

        Ok(Cow::Owned(result))
    }

    #[inline]
    fn path(&self) -> Option<Cow<'_, Path>> {
        Some(Cow::Borrowed(&self.path))
    }
}

impl<T> ObFileRead for ObFileOnDisk<T>
where
    T: DeserializeOwned + Clone,
{
    /// Creates instance from [`std::io::Read`]
    #[inline]
    fn from_reader(_reader: &mut impl Read, path: Option<impl AsRef<Path>>) -> Result<Self, Error> {
        Self::from_string("", path)
    }

    /// Creates instance from path
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();

        if !path.is_file() {
            return Err(Error::IsNotFile(path));
        }

        Ok(Self {
            path,
            phantom: PhantomData,
        })
    }

    /// Creates instance from text (requires path!)
    ///
    /// Dont use this function. Use `from_file`
    #[inline]
    fn from_string(
        _raw_text: impl AsRef<str>,
        path: Option<impl AsRef<Path>>,
    ) -> Result<Self, Error> {
        let path_buf = path.expect("Path is required").as_ref().to_path_buf();

        Self::from_file(path_buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obfile::ObFileDefault;
    use crate::obfile::impl_tests::impl_test_for_obfile;
    use crate::obfile::obfile_read::tests::{from_file, from_file_with_unicode};
    use crate::obfile::obfile_write::tests::impl_all_tests_flush;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    impl_all_tests_flush!(ObFileOnDisk);
    impl_test_for_obfile!(impl_from_file, from_file, ObFileOnDisk);

    impl_test_for_obfile!(
        impl_from_file_with_unicode,
        from_file_with_unicode,
        ObFileOnDisk
    );

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[should_panic]
    fn use_from_string_without_path() {
        ObFileOnDisk::from_string_default("", None::<&str>).unwrap();
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[should_panic]
    fn use_from_file_with_path_not_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        ObFileOnDisk::from_file_default(temp_dir.path()).unwrap();
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn get_path() {
        let test_file = NamedTempFile::new().unwrap();
        let file = ObFileOnDisk::from_file_default(test_file.path()).unwrap();

        assert_eq!(file.path().unwrap(), test_file.path());
        assert_eq!(file.path, test_file.path());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn get_content() {
        let test_data = "DATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let file = ObFileOnDisk::from_file_default(test_file.path()).unwrap();
        assert_eq!(file.content().unwrap(), test_data);
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn get_properties() {
        let test_data = "---\ntime: now\n---\nDATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let file = ObFileOnDisk::from_file_default(test_file.path()).unwrap();
        let properties = file.properties().unwrap().unwrap();

        assert_eq!(file.content().unwrap(), "DATA");
        assert_eq!(properties["time"], "now");
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn from_read() {
        let test_data = "---\ntime: now\n---\nDATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let file = ObFileOnDisk::from_read_default(
            &mut File::open(test_file.path()).unwrap(),
            Some(test_file.path()),
        )
        .unwrap();

        let properties = file.properties().unwrap().unwrap();

        assert_eq!(file.content().unwrap(), "DATA");
        assert_eq!(properties["time"], "now");
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[should_panic]
    fn from_read_but_without_path() {
        let test_data = "---\ntime: now\n---\nDATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let _file = ObFileOnDisk::from_read_default(
            &mut File::open(test_file.path()).unwrap(),
            None::<&str>,
        )
        .unwrap();
    }
}
