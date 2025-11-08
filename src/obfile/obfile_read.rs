//! Impl trait [`ObFileRead`]

use super::{Error, ObFile};
use serde::de::DeserializeOwned;
use std::{fs::File, io::Read, path::Path};

/// [`ObFile`] support read operation
pub trait ObFileRead: ObFile
where
    Self::Properties: DeserializeOwned,
{
    /// Parses an Obsidian note from a reader
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    fn from_read(read: &mut impl Read, path: Option<impl AsRef<Path>>) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::trace!("Parse obsidian file from reader");

        let mut data = Vec::new();
        read.read_to_end(&mut data)?;

        // SAFETY: Notes files in Obsidian (`*.md`) ensure that the file is encoded in UTF-8
        let text = unsafe { String::from_utf8_unchecked(data) };

        Self::from_string(&text, path)
    }

    /// Parses an Obsidian note from a file
    ///
    /// # Arguments
    /// - `path`: Filesystem path to markdown file
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::trace!("Parse obsidian file from file: {}", path_buf.display());

        let mut file = File::open(&path_buf)?;
        Self::from_read(&mut file, Some(path_buf))
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
    fn from_string(
        raw_text: impl AsRef<str>,
        path: Option<impl AsRef<Path>>,
    ) -> Result<Self, Error>;
}
