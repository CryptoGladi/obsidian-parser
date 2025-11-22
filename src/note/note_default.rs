//! Impl trait [`NoteDefault`]

use serde::de::DeserializeOwned;

use super::{DefaultProperties, Note};
use crate::note::{NoteFromReader, NoteFromString};
use std::{io::Read, path::Path};

/// Default implementation using [`std::collections::HashMap`] for properties
///
/// Automatically implemented for all `Note<Properties = HashMap<..>>` types.
/// Provides identical interface with explicitly named methods.
pub trait NoteDefault: Note {
    /// Same as [`NoteFromString::from_string`] with default properties type
    fn from_string_default(raw_text: impl AsRef<str>) -> Result<Self, Self::Error>
    where
        Self: NoteFromString,
        Self::Properties: DeserializeOwned;

    /// Same as [`crate::note::NoteFromFile::from_file`] with default properties type
    #[cfg(not(target_family = "wasm"))]
    fn from_file_default(path: impl AsRef<Path>) -> Result<Self, Self::Error>
    where
        Self: crate::prelude::NoteFromFile,
        Self::Properties: DeserializeOwned,
        Self::Error: From<std::io::Error>;

    /// Same as [`NoteFromReader::from_reader`] with default properties type
    fn from_reader_default(reader: &mut impl Read) -> Result<Self, Self::Error>
    where
        Self: NoteFromReader,
        Self::Properties: DeserializeOwned,
        Self::Error: From<std::io::Error>;
}

impl<T> NoteDefault for T
where
    T: Note<Properties = DefaultProperties>,
{
    fn from_string_default(raw_text: impl AsRef<str>) -> Result<Self, Self::Error>
    where
        Self: NoteFromString,
        Self::Properties: DeserializeOwned,
    {
        Self::from_string(raw_text)
    }

    #[cfg(not(target_family = "wasm"))]
    fn from_file_default(path: impl AsRef<Path>) -> Result<Self, Self::Error>
    where
        Self: crate::prelude::NoteFromFile,
        Self::Properties: DeserializeOwned,
        Self::Error: From<std::io::Error>,
    {
        Self::from_file(path)
    }

    fn from_reader_default(reader: &mut impl Read) -> Result<Self, Self::Error>
    where
        Self: NoteFromReader,
        Self::Properties: DeserializeOwned,
        Self::Error: From<std::io::Error>,
    {
        Self::from_reader(reader)
    }
}
