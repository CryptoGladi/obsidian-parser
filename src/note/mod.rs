//! Represents an Obsidian note file with frontmatter properties and content

pub mod note_default;
pub mod note_in_memory;
pub mod note_on_disk;
pub mod note_once_cell;
pub mod note_once_lock;
pub mod note_read;
pub mod note_read_write;
pub mod note_write;
pub mod parser;

use std::{borrow::Cow, collections::HashMap, fs::OpenOptions, path::Path};

pub use note_default::NoteDefault;
pub use note_read::NoteRead;
pub use note_read_write::NoteReadWrite;
pub use note_write::NoteWrite;

pub(crate) type DefaultProperties = HashMap<String, serde_yml::Value>;

/// Represents an Obsidian note file with frontmatter properties and content
///
/// This trait provides a standardized interface for working with Obsidian markdown files,
/// handling frontmatter parsing, content extraction, and file operations.
///
/// # Example
/// ```no_run
/// use obsidian_parser::prelude::*;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Clone)]
/// struct NoteProperties {
///     topic: String,
///     created: String,
/// }
///
/// let note: NoteInMemory<NoteProperties> = NoteRead::from_file("note.md").unwrap();
/// let properties = note.properties().unwrap().unwrap();
/// println!("Note topic: {}", properties.topic);
/// ```
///
/// # Other
/// * To open and read [`Note`] to a file, use the [`NoteRead`] trait.
/// * To write and modify [`Note`] to a file, use the [`NoteWrite`] trait.
/// * To read and write [`Note`] to a file, use the [`NoteReadWrite`] trait.
pub trait Note: Sized {
    /// Frontmatter properties type
    type Properties: Clone;

    /// Error type
    type Error: std::error::Error;

    /// Returns the parsed properties of frontmatter
    ///
    /// Returns [`None`] if the note has no properties
    fn properties(&self) -> Result<Option<Cow<'_, Self::Properties>>, Self::Error>;

    /// Returns the main content body of the note (excluding frontmatter)
    ///
    /// # Implementation Notes
    /// - Strips YAML frontmatter if present
    /// - Preserves original formatting and whitespace
    fn content(&self) -> Result<Cow<'_, str>, Self::Error>;

    /// Returns the source file path if available
    ///
    /// Returns [`None`] for in-memory notes without physical storage
    fn path(&self) -> Option<Cow<'_, Path>>;

    /// Get note name
    fn note_name(&self) -> Option<String> {
        self.path().as_ref().map(|path| {
            path.file_stem()
                .expect("Path is not file")
                .to_string_lossy()
                .to_string()
        })
    }
}

#[cfg(test)]
pub(crate) mod impl_tests {
    macro_rules! impl_test_for_note {
        ($name_test:ident, $fn_test:ident, $impl_note:path) => {
            #[cfg_attr(feature = "logging", test_log::test)]
            #[cfg_attr(not(feature = "logging"), test)]
            fn $name_test() {
                $fn_test::<$impl_note>().unwrap();
            }
        };
    }

    pub(crate) use impl_test_for_note;
}
