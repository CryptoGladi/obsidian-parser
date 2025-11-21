//! Impl trait [`NoteDefault`]

use serde::de::DeserializeOwned;

use super::{DefaultProperties, Note};
use crate::note::{NoteFromFile, NoteFromReader, NoteFromString};
use std::{io::Read, path::Path};

/// Default implementation using [`std::collections::HashMap`] for properties
///
/// Automatically implemented for all `Note<Properties = HashMap<..>>` types.
/// Provides identical interface with explicitly named methods.
pub trait NoteDefault: Note {
    /// Same as [`NoteRead::from_string`] with default properties type
    fn from_string_default(raw_text: impl AsRef<str>) -> Result<Self, Self::Error>
    where
        Self: NoteFromString,
        Self::Properties: DeserializeOwned;

    /// Same as [`NoteRead::from_file`] with default properties type
    fn from_file_default(path: impl AsRef<Path>) -> Result<Self, Self::Error>
    where
        Self: NoteFromFile,
        Self::Properties: DeserializeOwned,
        Self::Error: From<std::io::Error>;

    /// Same as [`NoteRead::from_reader`] with default properties type
    fn from_read_default(reader: &mut impl Read) -> Result<Self, Self::Error>
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

    fn from_file_default(path: impl AsRef<Path>) -> Result<Self, Self::Error>
    where
        Self: NoteFromFile,
        Self::Properties: DeserializeOwned,
        Self::Error: From<std::io::Error>,
    {
        Self::from_file(path)
    }

    fn from_read_default(reader: &mut impl Read) -> Result<Self, Self::Error>
    where
        Self: NoteFromReader,
        Self::Properties: DeserializeOwned,
        Self::Error: From<std::io::Error>,
    {
        Self::from_reader(reader)
    }
}
