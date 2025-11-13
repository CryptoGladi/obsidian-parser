use thiserror::Error;

/// Parses Obsidian-style links in note content
///
/// Handles all link formats:
/// - `[[Note]]`
/// - `[[Note|Alias]]`
/// - `[[Note^block]]`
/// - `[[Note#heading]]`
/// - `[[Note#heading|Alias]]`
///
/// # Example
/// ```
/// # use obsidian_parser::obfile::parse_links;
/// let content = "[[Physics]] and [[Math|Mathematics]]";
/// let links: Vec<_> = parse_links(content).collect();
/// assert_eq!(links, vec!["Physics", "Math"]);
/// ```
pub fn parse_links(text: &str) -> impl Iterator<Item = &str> {
    text.match_indices("[[").filter_map(move |(start_pos, _)| {
        let end_pos = text[start_pos + 2..].find("]]")?;
        let inner = &text[start_pos + 2..start_pos + 2 + end_pos];

        let note_name = inner
            .split('#')
            .next()?
            .split('^')
            .next()?
            .split('|')
            .next()?
            .trim();

        Some(note_name)
    })
}

#[derive(Debug, PartialEq)]
pub enum ResultParse<'a> {
    WithProperties {
        content: &'a str,
        properties: &'a str,
    },
    WithoutProperties,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid format")]
    InvalidFormat,
}

pub fn parse_obfile(raw_text: &str) -> Result<ResultParse<'_>, Error> {
    let have_start_properties = raw_text
        .lines()
        .next()
        .is_some_and(|line| line.trim_end() == "---");

    if have_start_properties {
        let closed = raw_text["---".len()..]
            .find("---")
            .ok_or(Error::InvalidFormat)?;

        return Ok(ResultParse::WithProperties {
            content: raw_text[(closed + 2 * "...".len())..].trim(),
            properties: raw_text["...".len()..(closed + "...".len())].trim(),
        });
    }

    Ok(ResultParse::WithoutProperties)
}

#[cfg(test)]
mod tests {
    use super::{ResultParse, parse_obfile};
    use crate::test_utils::init_test_logger;

    #[test]
    fn parse_obfile_without_properties() {
        init_test_logger();
        let test_data = "test_data";
        let result = parse_obfile(test_data).unwrap();

        assert_eq!(result, ResultParse::WithoutProperties);
    }

    #[test]
    fn parse_obfile_with_properties() {
        init_test_logger();
        let test_data = "---\nproperties data\n---\ntest data";
        let result = parse_obfile(test_data).unwrap();

        assert_eq!(
            result,
            ResultParse::WithProperties {
                content: "test data",
                properties: "properties data"
            }
        );
    }

    #[test]
    fn parse_obfile_without_properties_but_with_closed() {
        init_test_logger();
        let test_data1 = "test_data---";
        let test_data2 = "test_data\n---\n";

        let result1 = parse_obfile(test_data1).unwrap();
        let result2 = parse_obfile(test_data2).unwrap();

        assert_eq!(result1, ResultParse::WithoutProperties);
        assert_eq!(result2, ResultParse::WithoutProperties);
    }

    #[test]
    #[should_panic]
    fn parse_obfile_with_properties_but_without_closed() {
        init_test_logger();
        let test_data = "---\nproperties data\ntest data";
        let _ = parse_obfile(test_data).unwrap();
    }

    #[test]
    fn parse_obfile_with_() {
        init_test_logger();
        let test_data = "---properties data";

        let result = parse_obfile(test_data).unwrap();
        assert_eq!(result, ResultParse::WithoutProperties);
    }

    #[test]
    fn parse_obfile_without_properties_but_with_spaces() {
        init_test_logger();
        let test_data = "   ---\ndata";

        let result = parse_obfile(test_data).unwrap();
        assert_eq!(result, ResultParse::WithoutProperties);
    }

    #[test]
    fn parse_obfile_with_properties_but_check_trim_end() {
        init_test_logger();
        let test_data = "---\r\nproperties data\r\n---\r   \ntest data";
        let result = parse_obfile(test_data).unwrap();

        assert_eq!(
            result,
            ResultParse::WithProperties {
                content: "test data",
                properties: "properties data"
            }
        );
    }

    #[test]
    fn test_parse_links() {
        init_test_logger();
        let test_data =
            "[[Note]] [[Note|Alias]] [[Note^block]] [[Note#Heading|Alias]] [[Note^block|Alias]]";

        let ds: Vec<_> = super::parse_links(test_data).collect();

        assert!(ds.iter().all(|x| *x == "Note"))
    }
}
