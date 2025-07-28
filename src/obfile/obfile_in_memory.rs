//! In-memory representation of an Obsidian note file

use crate::error::Error;
use crate::obfile::{ObFile, ResultParse, parse_obfile};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, path::PathBuf};

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
/// For large vaults or read-heavy workflows, consider [`ObFileOnDisk`].
///
/// [`ObFileOnDisk`]: crate::obfile::obfile_on_disk::ObFileOnDisk
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ObFileInMemory<T = HashMap<String, serde_yml::Value>>
where
    T: DeserializeOwned + Clone,
{
    /// Markdown content body (without frontmatter)
    content: String,

    /// Source file path (if loaded from disk)
    path: Option<PathBuf>,

    /// Parsed frontmatter properties
    properties: Option<T>,
}

impl<T: DeserializeOwned + Clone> ObFile<T> for ObFileInMemory<T> {
    #[inline]
    fn content(&self) -> String {
        self.content.clone()
    }

    #[inline]
    fn path(&self) -> Option<PathBuf> {
        self.path.clone()
    }

    #[inline]
    fn properties(&self) -> Option<T> {
        self.properties.clone()
    }

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
    /// let file: ObFileInMemory<NoteProperties> = ObFileInMemory::from_string(note, None::<&str>).unwrap();
    /// assert_eq!(file.properties().unwrap().title, "Example");
    /// assert_eq!(file.content(), "Content");
    /// ```
    fn from_string<P: AsRef<std::path::Path>>(
        raw_text: &str,
        path: Option<P>,
    ) -> Result<Self, Error> {
        let path_buf = path.map(|x| x.as_ref().to_path_buf());

        #[cfg(feature = "logging")]
        log::trace!(
            "Parsing in-memory note{}",
            path_buf
                .as_ref()
                .map(|p| format!(" from {}", p.display()))
                .unwrap_or_default()
        );

        match parse_obfile(raw_text)? {
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
    use crate::obfile::impl_tests::{impl_all_tests_from_file, impl_all_tests_from_string};

    impl_all_tests_from_string!(ObFileInMemory);
    impl_all_tests_from_file!(ObFileInMemory);
}
