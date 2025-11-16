//! Impl trait [`NoteDefault`]

use super::DefaultProperties;
use crate::note::note_read::NoteRead;
use std::{io::Read, path::Path};

/// Default implementation using [`std::collections::HashMap`] for properties
///
/// Automatically implemented for all `Note<Properties = HashMap<..>>` types.
/// Provides identical interface with explicitly named methods.
pub trait NoteDefault: NoteRead<Properties = DefaultProperties>
where
    Self::Error: From<std::io::Error>,
{
    /// Same as [`NoteRead::from_string`] with default properties type
    ///
    /// # Errors
    /// - [`Error::InvalidFormat`] for malformed frontmatter
    /// - [`Error::Yaml`] for invalid YAML syntax
    fn from_string_default<P: AsRef<Path>>(
        text: &str,
        path: Option<P>,
    ) -> Result<Self, Self::Error>;

    /// Same as [`NoteRead::from_file`] with default properties type
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    fn from_file_default<P: AsRef<Path>>(path: P) -> Result<Self, Self::Error>;

    /// Same as [`NoteRead::from_read`] with default properties type
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    fn from_read_default<P: AsRef<Path>>(
        read: &mut impl Read,
        path: Option<P>,
    ) -> Result<Self, Self::Error>;
}

impl<T> NoteDefault for T
where
    T: NoteRead<Properties = DefaultProperties>,
    T::Error: From<std::io::Error>,
{
    #[inline]
    fn from_string_default<P: AsRef<Path>>(
        text: &str,
        path: Option<P>,
    ) -> Result<Self, Self::Error> {
        Self::from_string(text, path)
    }

    #[inline]
    fn from_file_default<P: AsRef<Path>>(path: P) -> Result<Self, Self::Error> {
        Self::from_file(path)
    }

    #[inline]
    fn from_read_default<P: AsRef<Path>>(
        read: &mut impl Read,
        path: Option<P>,
    ) -> Result<Self, Self::Error> {
        Self::from_reader(read, path)
    }
}
