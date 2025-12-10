//! On-disk representation of an Obsidian note file

use crate::note::parser::{self, ResultParse, parse_note};
use crate::note::{DefaultProperties, Note};
use serde::de::DeserializeOwned;
use std::borrow::Cow;
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
/// # Tradeoffs vs `NoteInMemory`
/// | Characteristic       | [`NoteOnDisk`]        | [`NoteInMemory`]          |
/// |----------------------|-------------------------|-----------------------------|
/// | Memory usage         | **Minimal** (~24 bytes) | High (content + properties) |
/// | File access          | On-demand               | Preloaded                   |
/// | Best for             | SSD-based vaults        | RAM-heavy workflows         |
/// | Content access cost  | Disk read               | Zero cost                   |
///
/// # Recommendation
/// Prefer `NoteOnDisk` for vault operations on modern hardware. The combination of
/// SSD speeds and Rust's efficient I/O makes this implementation ideal for:
/// - Large vaults (1000+ files)
/// - Graph processing
///
/// # Warning
/// Requires **persistent file access** throughout the object's lifetime
///
/// [`NoteInMemory`]: crate::note::note_in_memory::NoteInMemory
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct NoteOnDisk<T = DefaultProperties>
where
    T: Clone + DeserializeOwned,
{
    /// Absolute path to the source Markdown file
    path: PathBuf,

    /// For ignore `T`
    phantom: PhantomData<T>,
}

/// Errors for [`NoteOnDisk`]
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
    /// NoteOnDisk::from_file_default("/home/test");
    /// ```
    #[error("Path: `{0}` is not a directory")]
    IsNotFile(PathBuf),
}

impl<T> Note for NoteOnDisk<T>
where
    T: DeserializeOwned + Clone,
{
    type Properties = T;
    type Error = self::Error;

    /// Parses YAML frontmatter directly from disk
    ///
    /// # Errors
    /// - [`Error::Yaml`] if properties can't be deserialized
    /// - [`Error::IsNotFile`] If file doesn't exist
    /// - [`Error::IO`] on filesystem error
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(path = %self.path.display())))]
    fn properties(&self) -> Result<Option<Cow<'_, T>>, Error> {
        #[cfg(feature = "tracing")]
        tracing::trace!("Get properties from file");

        let data = std::fs::read(&self.path)?;

        // SAFETY: Notes files in Obsidian (`*.md`) ensure that the file is encoded in UTF-8
        let raw_text = unsafe { String::from_utf8_unchecked(data) };

        let result = match parse_note(&raw_text)? {
            ResultParse::WithProperties {
                content: _,
                properties,
            } => {
                #[cfg(feature = "tracing")]
                tracing::trace!("Frontmatter detected, parsing properties");

                Some(Cow::Owned(serde_yml::from_str(properties)?))
            }
            ResultParse::WithoutProperties => {
                #[cfg(feature = "tracing")]
                tracing::trace!("No frontmatter found, storing raw content");

                None
            }
        };

        Ok(result)
    }

    /// Returns the note's content body (without frontmatter)
    ///
    /// # Errors
    /// - [`Error::IO`] on filesystem error
    ///
    /// # Performance
    /// Performs disk read on every call. Suitable for:
    /// - Single-pass processing (link extraction, analysis)
    /// - Large files where in-memory storage is prohibitive
    ///
    /// For repeated access, consider caching or [`NoteInMemory`](crate::note::note_in_memory::NoteInMemory).
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(path = %self.path.display())))]
    fn content(&self) -> Result<Cow<'_, str>, Error> {
        #[cfg(feature = "tracing")]
        tracing::trace!("Get content from file");

        let data = std::fs::read(&self.path)?;

        // SAFETY: Notes files in Obsidian (`*.md`) ensure that the file is encoded in UTF-8
        let raw_text = unsafe { String::from_utf8_unchecked(data) };

        let result = match parse_note(&raw_text)? {
            ResultParse::WithProperties {
                content,
                properties: _,
            } => {
                #[cfg(feature = "tracing")]
                tracing::trace!("Frontmatter detected, parsing properties");

                content.to_string()
            }
            ResultParse::WithoutProperties => {
                #[cfg(feature = "tracing")]
                tracing::trace!("No frontmatter found, storing raw content");

                raw_text
            }
        };

        Ok(Cow::Owned(result))
    }

    /// Get path to note
    #[inline]
    fn path(&self) -> Option<Cow<'_, Path>> {
        Some(Cow::Borrowed(&self.path))
    }
}

impl<T> NoteOnDisk<T>
where
    T: DeserializeOwned + Clone,
{
    /// Set path to note
    #[inline]
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }
}

#[cfg(not(target_family = "wasm"))]
impl<T> crate::prelude::NoteFromFile for NoteOnDisk<T>
where
    T: DeserializeOwned + Clone,
{
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::NoteDefault;
    use crate::note::impl_tests::impl_test_for_note;
    use crate::note::note_aliases::tests::{from_file_have_aliases, from_file_have_not_aliases};
    use crate::note::note_is_todo::tests::{from_file_is_not_todo, from_file_is_todo};
    use crate::note::note_read::tests::{from_file, from_file_with_unicode};
    use crate::note::note_tags::tests::from_file_tags;
    use crate::note::note_write::tests::impl_all_tests_flush;
    use std::io::Write;
    use tempfile::NamedTempFile;

    impl_all_tests_flush!(NoteOnDisk);
    impl_test_for_note!(impl_from_file, from_file, NoteOnDisk);
    impl_test_for_note!(impl_from_file_tags, from_file_tags, NoteOnDisk);

    impl_test_for_note!(
        impl_from_file_with_unicode,
        from_file_with_unicode,
        NoteOnDisk
    );

    impl_test_for_note!(impl_from_file_is_todo, from_file_is_todo, NoteOnDisk);
    impl_test_for_note!(
        impl_from_file_is_not_todo,
        from_file_is_not_todo,
        NoteOnDisk
    );

    impl_test_for_note!(
        impl_from_file_have_aliases,
        from_file_have_aliases,
        NoteOnDisk
    );
    impl_test_for_note!(
        impl_from_file_have_not_aliases,
        from_file_have_not_aliases,
        NoteOnDisk
    );

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    #[should_panic]
    fn use_from_file_with_path_not_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        NoteOnDisk::from_file_default(temp_dir.path()).unwrap();
    }

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    fn get_path() {
        let test_file = NamedTempFile::new().unwrap();
        let file = NoteOnDisk::from_file_default(test_file.path()).unwrap();

        assert_eq!(file.path().unwrap(), test_file.path());
        assert_eq!(file.path, test_file.path());
    }

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    fn get_content() {
        let test_data = "DATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let file = NoteOnDisk::from_file_default(test_file.path()).unwrap();
        assert_eq!(file.content().unwrap(), test_data);
    }

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    fn get_properties() {
        let test_data = "---\ntime: now\n---\nDATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let file = NoteOnDisk::from_file_default(test_file.path()).unwrap();
        let properties = file.properties().unwrap().unwrap();

        assert_eq!(file.content().unwrap(), "DATA");
        assert_eq!(properties["time"], "now");
    }
}
