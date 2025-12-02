//! Represents an Obsidian note file with frontmatter properties and content

pub mod note_aliases;
pub mod note_default;
pub mod note_in_memory;
pub mod note_is_todo;
pub mod note_on_disk;
pub mod note_once_cell;
pub mod note_once_lock;
pub mod note_read;
pub mod parser;

#[cfg(not(target_family = "wasm"))]
pub mod note_write;

use std::{borrow::Cow, collections::HashMap, fs::OpenOptions, path::Path};

pub use note_default::NoteDefault;
pub use note_read::{NoteFromReader, NoteFromString};

#[cfg(not(target_family = "wasm"))]
pub use note_read::NoteFromFile;

#[cfg(not(target_family = "wasm"))]
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
/// let note: NoteInMemory<NoteProperties> = NoteFromFile::from_file("note.md").unwrap();
/// let properties = note.properties().unwrap().unwrap();
/// println!("Note topic: {}", properties.topic);
/// ```
///
/// # Other
/// * To open and read [`Note`] to a file, see [`note_read`] module.
/// * To write and modify [`Note`] to a file, use the [`NoteWrite`] trait.
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

    /// Get count words from content
    ///
    /// # Example
    ///
    /// ```
    /// use obsidian_parser::prelude::*;
    ///
    /// let data = "---\ntags:\n- my_tag\n---\n My super note";
    /// let note = NoteInMemory::from_string_default(data).unwrap();
    ///
    /// assert_eq!(note.count_words_from_content().unwrap(), 3);
    /// ```
    fn count_words_from_content(&self) -> Result<usize, Self::Error> {
        let content = self.content()?;
        Ok(content.split_whitespace().count())
    }

    /// Get count symbols from content
    ///
    /// # Example
    ///
    /// ```
    /// use obsidian_parser::prelude::*;
    ///
    /// let data = "---\ntags:\n- my_tag\n---\n My super note";
    /// let content = "My super note";
    ///
    /// let note = NoteInMemory::from_string_default(data).unwrap();
    ///
    /// assert_eq!(note.count_symbols_from_content().unwrap(), content.len());
    /// ``````
    fn count_symbols_from_content(&self) -> Result<usize, Self::Error> {
        let content = self.content()?;
        Ok(content.len())
    }
}

#[cfg(test)]
pub(crate) mod impl_tests {
    macro_rules! impl_test_for_note {
        ($name_test:ident, $fn_test:ident, $impl_note:path) => {
            #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
            #[test]
            fn $name_test() {
                $fn_test::<$impl_note>().unwrap();
            }
        };
    }

    pub(crate) use impl_test_for_note;
}
