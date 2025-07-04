use crate::error::Error;
use crate::obfile::{ObFile, parse_obfile};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::{collections::HashMap, path::PathBuf};

/// On-disk representation of an Obsidian note file
///
/// Optimized for vault operations where:
/// 1. Memory efficiency is critical (large vaults)
/// 2. Storage is fast (SSD/NVMe)
/// 3. Content is accessed infrequently
///
/// # Tradeoffs vs `ObFileInMemory`
/// | Characteristic       | `ObFileOnDisk`          | `ObFileInMemory`            |
/// |----------------------|-------------------------|-----------------------------|
/// | Memory usage         | **Minimal** (~24 bytes) | High (content + properties) |
/// | File access          | On-demand               | Preloaded                   |
/// | Best for             | SSD-based vaults        | RAM-heavy workflows         |
/// | Content access cost  | Disk read               | Zero cost                   |
///
/// # Recommendation
/// Prefer `ObFileOnDisk` for vault operations on modern hardware. The combination of
/// SSD speeds and Rust's efficient I/O makes this implementation ideal for:
/// - Large vaults (1000+ files)
/// - Graph processing
///
/// # Warning
/// Requires **persistent file access** throughout the object's lifetime. If files are moved/deleted,
/// calling `content()` or `properties()` will **panic**
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ObFileOnDisk<T = HashMap<String, serde_yaml::Value>>
where
    T: DeserializeOwned + Default + Clone + Send,
{
    /// Absolute path to the source Markdown file
    pub path: PathBuf,

    phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Default + Clone + Send> ObFile<T> for ObFileOnDisk<T> {
    /// Returns the note's content body (without frontmatter)
    ///
    /// # Panics
    /// - If file doesn't exist
    /// - On filesystem errors
    /// - If file contains invalid UTF-8
    ///
    /// # Performance
    /// Performs disk read on every call. Suitable for:
    /// - Single-pass processing (link extraction, analysis)
    /// - Large files where in-memory storage is prohibitive
    ///
    /// For repeated access, consider caching or `ObFileInMemory`.
    fn content(&self) -> String {
        let raw_text = std::fs::read_to_string(&self.path).unwrap();
        let (valid_properties, parts) = parse_obfile(&raw_text);

        match (valid_properties, &parts[..]) {
            (false, _) => raw_text,
            (true, [_, _, content]) => (*content).to_string(),
            _ => unimplemented!(),
        }
    }

    /// Parses YAML frontmatter directly from disk
    ///
    /// # Panics
    /// - If properties can't be deserialized
    fn properties(&self) -> T {
        let raw_text = std::fs::read_to_string(&self.path).unwrap();
        let (valid_properties, parts) = parse_obfile(&raw_text);

        match (valid_properties, &parts[..]) {
            (false, _) => T::default(),
            (true, [_, properties, _]) => serde_yaml::from_str(properties).unwrap(),
            _ => unreachable!(),
        }
    }

    fn path(&self) -> Option<PathBuf> {
        Some(self.path.clone())
    }

    /// Creates instance from text (requires path!)
    fn from_string<P: AsRef<std::path::Path>>(
        _raw_text: &str,
        path: Option<P>,
    ) -> Result<Self, Error> {
        let path_buf = path.expect("Path is required").as_ref().to_path_buf();

        Ok(Self {
            path: path_buf,
            phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obfile::{
        ObFileDefault,
        tests::{from_file, impl_test_for_obfile},
    };

    impl_test_for_obfile!(impl_from_file, from_file, ObFileOnDisk);

    #[test]
    #[should_panic]
    fn use_from_string_without_path() {
        ObFileOnDisk::from_string_default("", None::<&str>).unwrap();
    }
}
