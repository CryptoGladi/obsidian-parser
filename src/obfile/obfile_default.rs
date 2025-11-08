//! Impl trait [`ObFileDefault`]

use super::{DefaultProperties, Error};
use crate::obfile::obfile_read::ObFileRead;
use std::{io::Read, path::Path};

/// Default implementation using [`std::collections::HashMap`] for properties
///
/// Automatically implemented for all `ObFile<HashMap<..>>` types.
/// Provides identical interface with explicitly named methods.
pub trait ObFileDefault: ObFileRead<Properties = DefaultProperties> {
    /// Same as [`ObFileRead::from_string`] with default properties type
    ///
    /// # Errors
    /// - [`Error::InvalidFormat`] for malformed frontmatter
    /// - [`Error::Yaml`] for invalid YAML syntax
    fn from_string_default<P: AsRef<Path>>(text: &str, path: Option<P>) -> Result<Self, Error>;

    /// Same as [`ObFileRead::from_file`] with default properties type
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    fn from_file_default<P: AsRef<Path>>(path: P) -> Result<Self, Error>;

    /// Same as [`ObFileRead::from_read`] with default properties type
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    fn from_read_default<P: AsRef<Path>>(
        read: &mut impl Read,
        path: Option<P>,
    ) -> Result<Self, Error>;
}

impl<T> ObFileDefault for T
where
    T: ObFileRead<Properties = DefaultProperties>,
{
    #[inline]
    fn from_string_default<P: AsRef<Path>>(text: &str, path: Option<P>) -> Result<Self, Error> {
        Self::from_string(text, path)
    }

    #[inline]
    fn from_file_default<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_file(path)
    }

    #[inline]
    fn from_read_default<P: AsRef<Path>>(
        read: &mut impl Read,
        path: Option<P>,
    ) -> Result<Self, Error> {
        Self::from_read(read, path)
    }
}
