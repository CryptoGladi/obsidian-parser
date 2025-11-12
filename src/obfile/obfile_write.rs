//! Impl trait [`ObFileWrite`]

use super::{Error, ObFile, OpenOptions, ResultParse, parse_obfile};
use serde::Serialize;
use std::io::Write;

/// [`ObFile`] support write operation
pub trait ObFileWrite: ObFile
where
    Self::Properties: Serialize,
    Self::Error: From<std::io::Error> + From<serde_yml::Error>,
{
    /// Flush only `content`
    ///
    /// Ignore if path is `None`
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    fn flush_content(&self, open_option: &OpenOptions) -> Result<(), Self::Error> {
        if let Some(path) = self.path() {
            let text = std::fs::read_to_string(&path)?;
            let parsed = parse_obfile(&text)?;

            let mut file = open_option.open(path)?;

            match parsed {
                ResultParse::WithProperties {
                    content: _,
                    properties,
                } => file.write_all(
                    format!("---\n{}\n---\n{}", properties, self.content()?).as_bytes(),
                )?,
                ResultParse::WithoutProperties => file.write_all(self.content()?.as_bytes())?,
            }
        }

        Ok(())
    }

    /// Flush only `content`
    ///
    /// Ignore if path is `None`
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
    fn flush_properties(&self, open_option: &OpenOptions) -> Result<(), Self::Error> {
        if let Some(path) = self.path() {
            let text = std::fs::read_to_string(&path)?;
            let parsed = parse_obfile(&text)?;

            let mut file = open_option.open(path)?;

            match parsed {
                ResultParse::WithProperties {
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
                ResultParse::WithoutProperties => file.write_all(self.content()?.as_bytes())?,
            }
        }

        Ok(())
    }

    /// Flush [`ObFile`] to [`ObFile::path`]
    ///
    /// Ignore if path is `None`
    /// # Errors
    /// - [`Error::Io`] for filesystem errors
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

impl<T: ObFile> ObFileWrite for T
where
    T::Properties: Serialize,
    T::Error: From<std::io::Error>,
{
}
