use super::{DefaultProperties, Error};
use crate::obfile::obfile_read::ObFileRead;
use std::path::Path;

/// Default implementation using [`HashMap`] for properties
///
/// Automatically implemented for all `ObFile<HashMap<..>>` types.
/// Provides identical interface with explicitly named methods.
pub trait ObFileDefault: ObFileRead<Properties = DefaultProperties> {
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
    fn from_file_default<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
}

impl<T> ObFileDefault for T
where
    T: ObFileRead<Properties = DefaultProperties>,
{
    fn from_string_default<P: AsRef<Path>>(text: &str, path: Option<P>) -> Result<Self, Error> {
        Self::from_string(text, path)
    }

    fn from_file_default<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_file(path)
    }
}
