//! In-memory representation of an Obsidian note file

use super::{DefaultProperties, Note, NoteRead};
use crate::note::parser::{self, ResultParse, parse_note};
use serde::de::DeserializeOwned;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// In-memory representation of an Obsidian note file
///
/// This struct provides full access to parsed note content, properties, and path.
/// It stores the entire file contents in memory, making it suitable for:
/// - Frequent access to note content
/// - Transformation or analysis workflows
/// - Environments with fast storage (SSD/RAM disks)
///
/// # Performance Considerations
/// - Uses ~2x memory of original file size (UTF-8 + deserialized properties)
/// - Preferred for small-to-medium vaults (<10k notes)
///
/// For large vaults or read-heavy workflows, consider [`NoteOnDisk`].
///
/// [`NoteOnDisk`]: crate::note::note_on_disk::NoteOnDisk
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct NoteInMemory<T = DefaultProperties>
where
    T: Clone,
{
    /// Markdown content body (without frontmatter)
    content: String,

    /// Source file path (if loaded from disk)
    path: Option<PathBuf>,

    /// Parsed frontmatter properties
    properties: Option<T>,
}

/// Errors in [`NoteInMemory`]
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
}

impl<T> Note for NoteInMemory<T>
where
    T: Clone,
{
    type Properties = T;
    type Error = self::Error;

    #[inline]
    fn properties(&self) -> Result<Option<Cow<'_, T>>, Self::Error> {
        Ok(self.properties.as_ref().map(|p| Cow::Borrowed(p)))
    }

    #[inline]
    fn content(&self) -> Result<Cow<'_, str>, Self::Error> {
        Ok(Cow::Borrowed(&self.content))
    }

    #[inline]
    fn path(&self) -> Option<Cow<'_, Path>> {
        self.path.as_ref().map(|p| Cow::Borrowed(p.as_path()))
    }
}

impl<T> NoteRead for NoteInMemory<T>
where
    T: DeserializeOwned + Clone,
{
    /// Parses a string into an in-memory Obsidian note representation
    ///
    /// # Arguments
    /// * `raw_text` - Full note text including optional frontmatter
    /// * `path` - Optional source path for reference
    ///
    /// # Process
    /// 1. Splits text into frontmatter/content sections
    /// 2. Parses YAML frontmatter if present
    /// 3. Stores content without frontmatter delimiters
    ///
    /// # Errors
    /// - [`Error::InvalidFormat`] for malformed frontmatter
    /// - [`Error::Yaml`] for invalid YAML syntax
    ///
    /// # Example
    /// ```rust
    /// use obsidian_parser::prelude::*;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Clone, Default)]
    /// struct NoteProperties {
    ///     title: String
    /// }
    ///
    /// let note = r#"---
    /// title: Example
    /// ---
    /// Content"#;
    ///
    /// let file: NoteInMemory<NoteProperties> = NoteInMemory::from_string(note, None::<&str>).unwrap();
    /// let properties = file.properties().unwrap().unwrap();
    ///
    /// assert_eq!(properties.title, "Example");
    /// assert_eq!(file.content().unwrap(), "Content");
    /// ```
    fn from_string(
        raw_text: impl AsRef<str>,
        path: Option<impl AsRef<Path>>,
    ) -> Result<Self, Self::Error> {
        let path_buf = path.map(|x| x.as_ref().to_path_buf());
        let raw_text = raw_text.as_ref();

        #[cfg(feature = "logging")]
        log::trace!(
            "Parsing in-memory note{}",
            path_buf
                .as_ref()
                .map(|p| format!(" from {}", p.display()))
                .unwrap_or_default()
        );

        match parse_note(raw_text)? {
            ResultParse::WithProperties {
                content,
                properties,
            } => {
                #[cfg(feature = "logging")]
                log::trace!("Frontmatter detected, parsing properties");

                Ok(Self {
                    content: content.to_string(),
                    properties: Some(serde_yml::from_str(properties)?),
                    path: path_buf,
                })
            }
            ResultParse::WithoutProperties => {
                #[cfg(feature = "logging")]
                log::trace!("No frontmatter found, storing raw content");

                Ok(Self {
                    content: raw_text.to_string(),
                    path: path_buf,
                    properties: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::{
        note_read::tests::{
            impl_all_tests_from_file, impl_all_tests_from_reader, impl_all_tests_from_string,
        },
        note_write::tests::impl_all_tests_flush,
    };

    impl_all_tests_from_reader!(NoteInMemory);
    impl_all_tests_from_string!(NoteInMemory);
    impl_all_tests_from_file!(NoteInMemory);
    impl_all_tests_flush!(NoteInMemory);
}
