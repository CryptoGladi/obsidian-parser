//! Represents an Obsidian note file with frontmatter properties and content

pub mod obfile_default;
pub mod obfile_in_memory;
pub mod obfile_on_disk;
pub mod obfile_read;
pub mod obfile_read_write;
pub mod obfile_write;
pub mod parser;

use std::{borrow::Cow, collections::HashMap, fs::OpenOptions, path::Path};

pub use obfile_default::ObFileDefault;
pub use obfile_read::ObFileRead;
pub use obfile_read_write::ObFileReadWrite;
pub use obfile_write::ObFileWrite;

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
/// let note: ObFileInMemory<NoteProperties> = ObFileRead::from_file("note.md").unwrap();
/// let properties = note.properties().unwrap().unwrap();
/// println!("Note topic: {}", properties.topic);
/// ```
///
/// # Other
/// * To open and read [`ObFile`] to a file, use the [`ObFileRead`] trait.
/// * To write and modify [`ObFile`] to a file, use the [`ObFileWrite`] trait.
pub trait ObFile: Sized {
    /// Frontmatter properties type
    type Properties: Clone;

    type Error: std::error::Error;

    /// Returns the parsed properties of frontmatter
    ///
    /// Returns [`None`] if the note has no properties
    ///
    /// # Errors
    /// Usually errors are related to [`Error::Io`]
    fn properties(&self) -> Result<Option<Cow<'_, Self::Properties>>, Self::Error>;

    /// Returns the main content body of the note (excluding frontmatter)
    ///
    /// # Implementation Notes
    /// - Strips YAML frontmatter if present
    /// - Preserves original formatting and whitespace
    ///
    /// # Errors
    /// Usually errors are related to [`Error::Io`]
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
    macro_rules! impl_test_for_obfile {
        ($name_test:ident, $fn_test:ident, $impl_obfile:path) => {
            #[test]
            fn $name_test() {
                $fn_test::<$impl_obfile>().unwrap();
            }
        };
    }

    pub(crate) use impl_test_for_obfile;
}
