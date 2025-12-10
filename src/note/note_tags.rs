//! Impl trait [`NoteTags`]

use unic_emoji_char::is_emoji;

use super::{DefaultProperties, Note};

/// Trait for get tags from note
pub trait NoteTags: Note {
    /// Return tags from Note
    ///
    /// # Example
    /// ```
    /// use obsidian_parser::prelude::*;
    ///
    /// let raw_text = "---\ntags:\n- my_tag\n---\nSameData #super_tag ##no_tag and #warning_tag! #😭";
    /// let note = NoteInMemory::from_string(raw_text).unwrap();
    ///
    /// let tags = note.tags().unwrap();
    /// assert_eq!(tags, vec!["my_tag", "super_tag", "warning_tag", "😭"])
    /// ```
    fn tags(&self) -> Result<Vec<String>, Self::Error>;
}

impl<N> NoteTags for N
where
    N: Note<Properties = DefaultProperties>,
    N::Error: From<serde_yml::Error>,
{
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), ret, fields(path = format!("{:?}", self.path()))))]
    fn tags(&self) -> Result<Vec<String>, N::Error> {
        #[cfg(feature = "tracing")]
        tracing::trace!("Get tags");

        let properties = self.properties()?.unwrap_or_default();
        let tags_from_properties: Vec<String> = match properties.get("tags") {
            Some(value) => serde_yml::from_value(value.clone())?,
            None => Vec::default(),
        };

        let check_good =
            |c: char| c.is_alphanumeric() || (is_emoji(c) && c != '#') || c == '_' || c == '-';

        let content = self.content()?;
        let tags_from_content: Vec<_> = content
            .split_whitespace()
            .filter(|word| word.starts_with('#'))
            .filter(|word| word.bytes().nth(1) != Some(b'#'))
            .map(|word| word[1..].to_string())
            .filter_map(|tag| {
                let end_index = tag.find(|c| !check_good(c)).unwrap_or(tag.len());

                if end_index > 0 {
                    return Some(tag[..end_index].to_string());
                }

                None
            })
            .collect();

        Ok([tags_from_properties, tags_from_content].concat())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::prelude::*;
    use serde::de::DeserializeOwned;
    use std::io::{Cursor, Write};
    use tempfile::NamedTempFile;

    const TEST_STR_DATA: &str = "---\ntags:\n- my_tag\n---\nSameData #super_tag ##no_tag and #warning_tag! #two-tag #kek;d #dfds# #all, #татар #d😭";
    const TEST_ARRAY_DATA: &[&str] = &[
        "my_tag",
        "super_tag",
        "warning_tag",
        "two-tag",
        "kek",
        "dfds",
        "all",
        "татар",
        "d😭",
    ];

    pub(crate) fn tags<N>(note: &N) -> Result<(), N::Error>
    where
        N: NoteTags,
    {
        let tags = note.tags()?;
        assert_eq!(tags, TEST_ARRAY_DATA);

        Ok(())
    }

    pub(crate) fn from_string_tags<N>() -> Result<(), N::Error>
    where
        N: NoteFromString + NoteTags,
        N::Properties: DeserializeOwned,
    {
        let note = N::from_string(TEST_STR_DATA)?;
        tags(&note)
    }

    pub(crate) fn from_reader_tags<N>() -> Result<(), N::Error>
    where
        N: NoteFromReader + NoteTags,
        N::Properties: DeserializeOwned,
        N::Error: From<std::io::Error>,
    {
        let note = N::from_reader(&mut Cursor::new(TEST_STR_DATA))?;
        tags(&note)
    }

    pub(crate) fn from_file_tags<N>() -> Result<(), N::Error>
    where
        N: NoteFromFile + NoteTags,
        N::Properties: DeserializeOwned,
        N::Error: From<std::io::Error>,
    {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(TEST_STR_DATA.as_bytes()).unwrap();

        let note = N::from_file(file.path())?;
        tags(&note)
    }

    macro_rules! impl_all_tests_tags {
        ($impl_note:path) => {
            #[allow(unused_imports)]
            use $crate::note::note_tags::tests::*;

            impl_test_for_note!(impl_from_string_tags, from_string_tags, $impl_note);
            impl_test_for_note!(impl_from_reader_tags, from_reader_tags, $impl_note);
            impl_test_for_note!(impl_from_file_tags, from_file_tags, $impl_note);
        };
    }

    pub(crate) use impl_all_tests_tags;
}
