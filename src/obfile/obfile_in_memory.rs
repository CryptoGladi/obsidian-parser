use crate::error::Error;
use crate::obfile::{ObFile, parse_obfile};
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
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ObFileInMemory<T = HashMap<String, serde_yaml::Value>>
where
    T: DeserializeOwned + Default + Clone + Send,
{
    /// Markdown content body (without frontmatter)
    pub content: String,

    /// Source file path (if loaded from disk)
    pub path: Option<PathBuf>,

    /// Parsed frontmatter properties
    pub properties: T,
}

impl<T: DeserializeOwned + Default + Clone + Send> ObFile<T> for ObFileInMemory<T> {
    fn content(&self) -> String {
        self.content.clone()
    }

    fn path(&self) -> Option<PathBuf> {
        self.path.clone()
    }

    fn properties(&self) -> T {
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
    /// assert_eq!(file.properties().title, "Example");
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

        let (valid_properties, parts) = parse_obfile(raw_text);
        match (valid_properties, &parts[..]) {
            (false, _) => {
                #[cfg(feature = "logging")]
                log::debug!("No frontmatter found, storing raw content");

                Ok(Self {
                    content: raw_text.to_string(),
                    path: path_buf,
                    properties: T::default(),
                })
            }
            (true, [_, properties, content]) => {
                #[cfg(feature = "logging")]
                log::debug!("Frontmatter detected, parsing properties");

                Ok(Self {
                    content: content.trim().to_string(),
                    properties: serde_yaml::from_str(properties)?,
                    path: path_buf,
                })
            }
            _ => {
                #[cfg(feature = "logging")]
                log::error!(
                    "Invalid frontmatter format{}",
                    path_buf
                        .as_ref()
                        .map(|p| format!(" in {}", p.display()))
                        .unwrap_or_default()
                );

                Err(Error::InvalidFormat)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obfile::tests::impl_all_test_for_obfile;

    impl_all_test_for_obfile!(ObFileInMemory);
}
