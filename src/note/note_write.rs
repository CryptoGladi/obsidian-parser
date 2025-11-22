//! Impl trait [`NoteWrite`]

use super::{Note, OpenOptions};
use crate::note::parser;
use serde::Serialize;
use std::io::Write;

/// [`Note`] support write operation
pub trait NoteWrite: Note
where
    Self::Properties: Serialize,
    Self::Error: From<std::io::Error> + From<serde_yml::Error> + From<parser::Error>,
{
    /// Flush only `content`
    ///
    /// Ignore if path is `None`
    fn flush_content(&self, open_option: &OpenOptions) -> Result<(), Self::Error> {
        if let Some(path) = self.path() {
            let text = std::fs::read_to_string(&path)?;
            let parsed = parser::parse_note(&text)?;

            let mut file = open_option.open(path)?;

            match parsed {
                parser::ResultParse::WithProperties {
                    content: _,
                    properties,
                } => file.write_all(
                    format!("---\n{}\n---\n{}", properties, self.content()?).as_bytes(),
                )?,
                parser::ResultParse::WithoutProperties => {
                    file.write_all(self.content()?.as_bytes())?;
                }
            }
        }

        Ok(())
    }

    /// Flush only `content`
    ///
    /// Ignore if path is `None`
    fn flush_properties(&self, open_option: &OpenOptions) -> Result<(), Self::Error> {
        if let Some(path) = self.path() {
            let text = std::fs::read_to_string(&path)?;
            let parsed = parser::parse_note(&text)?;

            let mut file = open_option.open(path)?;

            match parsed {
                parser::ResultParse::WithProperties {
                    content,
                    properties: _,
                } => match self.properties()? {
                    Some(properties) => file.write_all(
                        format!(
                            "---\n{}\n---\n{}",
                            serde_yml::to_string(&properties)?,
                            content
                        )
                        .as_bytes(),
                    )?,
                    None => file.write_all(self.content()?.as_bytes())?,
                },
                parser::ResultParse::WithoutProperties => {
                    file.write_all(self.content()?.as_bytes())?;
                }
            }
        }

        Ok(())
    }

    /// Flush [`Note`] to [`Note::path`]
    ///
    /// Ignore if path is `None`
    fn flush(&self, open_option: &OpenOptions) -> Result<(), Self::Error> {
        if let Some(path) = self.path() {
            let mut file = open_option.open(path)?;

            match self.properties()? {
                Some(properties) => file.write_all(
                    format!(
                        "---\n{}\n---\n{}",
                        serde_yml::to_string(&properties)?,
                        self.content()?
                    )
                    .as_bytes(),
                )?,
                None => file.write_all(self.content()?.as_bytes())?,
            }
        }

        Ok(())
    }
}

impl<T: Note> NoteWrite for T
where
    T::Properties: Serialize,
    Self::Error: From<std::io::Error> + From<serde_yml::Error> + From<super::parser::Error>,
{
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::note::{DefaultProperties, NoteFromFile};
    use tempfile::NamedTempFile;

    const TEST_DATA: &str = "---\n\
topic: life\n\
created: 2025-03-16\n\
---\n\
Test data\n\
---\n\
Two test data";

    pub(crate) fn flush_properties<T>() -> Result<(), T::Error>
    where
        T: NoteFromFile<Properties = DefaultProperties> + NoteWrite,
        T::Error: From<std::io::Error> + From<serde_yml::Error> + From<parser::Error>,
    {
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(TEST_DATA.as_bytes()).unwrap();

        let file = T::from_file(test_file.path())?;
        let open_options = OpenOptions::new().write(true).create(false).clone();
        file.flush_properties(&open_options)?;
        drop(file);

        let file = T::from_file(test_file.path())?;

        let properties = file.properties()?.unwrap();
        assert_eq!(properties["topic"], "life");
        assert_eq!(properties["created"], "2025-03-16");
        assert_eq!(file.content().unwrap(), "Test data\n---\nTwo test data");

        Ok(())
    }

    pub(crate) fn flush_content<T>() -> Result<(), T::Error>
    where
        T: NoteFromFile<Properties = DefaultProperties> + NoteWrite,
        T::Error: From<std::io::Error> + From<serde_yml::Error> + From<parser::Error>,
    {
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(TEST_DATA.as_bytes()).unwrap();

        let file = T::from_file(test_file.path())?;
        let open_options = OpenOptions::new().write(true).create(false).clone();
        file.flush_content(&open_options)?;
        drop(file);

        let file = T::from_file(test_file.path())?;
        let properties = file.properties()?.unwrap();
        assert_eq!(properties["topic"], "life");
        assert_eq!(properties["created"], "2025-03-16");
        assert_eq!(file.content().unwrap(), "Test data\n---\nTwo test data");

        Ok(())
    }

    pub(crate) fn flush<T>() -> Result<(), T::Error>
    where
        T: NoteFromFile<Properties = DefaultProperties> + NoteWrite,
        T::Error: From<std::io::Error> + From<serde_yml::Error> + From<parser::Error>,
    {
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(TEST_DATA.as_bytes()).unwrap();

        let file = T::from_file(test_file.path())?;
        let open_options = OpenOptions::new().write(true).create(false).clone();
        file.flush(&open_options)?;
        drop(file);

        let file = T::from_file(test_file.path())?;
        let properties = file.properties()?.unwrap();
        assert_eq!(properties["topic"], "life");
        assert_eq!(properties["created"], "2025-03-16");
        assert_eq!(file.content().unwrap(), "Test data\n---\nTwo test data");

        Ok(())
    }

    macro_rules! impl_all_tests_flush {
        ($impl_note:path) => {
            #[allow(unused_imports)]
            use $crate::note::note_write::tests::*;

            impl_test_for_note!(impl_flush, flush, $impl_note);
            impl_test_for_note!(impl_flush_content, flush_content, $impl_note);
            impl_test_for_note!(impl_flush_properties, flush_properties, $impl_note);
        };
    }

    pub(crate) use impl_all_tests_flush;
}
