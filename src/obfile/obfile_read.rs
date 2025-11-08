use super::{Error, ObFile};
use serde::de::DeserializeOwned;
use std::path::Path;

pub trait ObFileRead: ObFile
where
    Self::Properties: DeserializeOwned,
{
    /// Parses an Obsidian note from a file
    ///
    /// # Arguments
    /// - `path`: Filesystem path to markdown file
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::trace!("Parse obsidian file from file: {}", path_buf.display());

        let data = std::fs::read(path)?;

        // SAFETY: Notes files in Obsidian (`*.md`) ensure that the file is encoded in UTF-8
        let text = unsafe { String::from_utf8_unchecked(data) };

        Self::from_string(&text, Some(path_buf))
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
}
