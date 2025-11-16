//! Module for impl [`NoteReadWrite`]

use super::{NoteRead, NoteWrite};
use crate::note::parser;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Trait for unification [`NoteRead`] and [`NoteWrite`]
pub trait NoteReadWrite: NoteRead + NoteWrite
where
    Self::Properties: Serialize + DeserializeOwned,
    Self::Error: From<std::io::Error> + From<serde_yml::Error> + From<parser::Error>,
{
}

impl<T> NoteReadWrite for T
where
    T: NoteRead + NoteWrite,
    T::Properties: Serialize + DeserializeOwned,
    T::Error: From<std::io::Error> + From<serde_yml::Error> + From<parser::Error>,
{
}
