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
            (true, [_, _, content]) => (*content).trim().to_string(),
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
    ///
    /// Dont use this function. Use `from_file`
    fn from_string<P: AsRef<std::path::Path>>(
        _raw_text: &str,
        path: Option<P>,
    ) -> Result<Self, Error> {
        let path_buf = path.expect("Path is required").as_ref().to_path_buf();

        Self::from_file(path_buf)
    }

    /// Creates instance from path
    fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let path_buf = path.as_ref().to_path_buf();

        if !path_buf.is_file() {
            return Err(Error::IsNotFile(path_buf));
        }

        Ok(Self {
            path: path_buf,
            phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obfile::ObFileDefault;
    use crate::obfile::tests::{from_file, from_file_with_unicode, impl_test_for_obfile};
    use crate::test_utils::init_test_logger;
    use std::io::Write;
    use tempfile::NamedTempFile;

    impl_test_for_obfile!(impl_from_file, from_file, ObFileOnDisk);

    impl_test_for_obfile!(
        impl_from_file_with_unicode,
        from_file_with_unicode,
        ObFileOnDisk
    );

    #[test]
    #[should_panic]
    fn use_from_string_without_path() {
        init_test_logger();
        ObFileOnDisk::from_string_default("", None::<&str>).unwrap();
    }

    #[test]
    #[should_panic]
    fn use_from_file_with_path_not_file() {
        init_test_logger();
        let temp_dir = tempfile::tempdir().unwrap();

        ObFileOnDisk::from_file_default(temp_dir.path()).unwrap();
    }

    #[test]
    fn get_path() {
        init_test_logger();
        let test_file = NamedTempFile::new().unwrap();
        let file = ObFileOnDisk::from_file_default(test_file.path()).unwrap();

        assert_eq!(file.path().unwrap(), test_file.path());
        assert_eq!(file.path, test_file.path());
    }

    #[test]
    fn get_content() {
        init_test_logger();
        let test_data = "DATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let file = ObFileOnDisk::from_file_default(test_file.path()).unwrap();
        assert_eq!(file.content(), test_data);
    }

    #[test]
    fn get_properties() {
        init_test_logger();
        let test_data = "---\ntime: now\n---\nDATA";
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(test_data.as_bytes()).unwrap();

        let file = ObFileOnDisk::from_file_default(test_file.path()).unwrap();
        assert_eq!(file.content(), "DATA");
        assert_eq!(file.properties()["time"], "now");
    }
}
