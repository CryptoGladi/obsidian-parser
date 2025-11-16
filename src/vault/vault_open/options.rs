use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VaultOptions {
    path: PathBuf,
}

impl VaultOptions {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    #[inline]
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[inline]
    #[must_use]
    pub fn into_path(self) -> PathBuf {
        self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn new() {
        let path = PathBuf::from("path/to/vault");
        let options = VaultOptions::new(&path);

        assert_eq!(options.path, path);
        assert_eq!(options.path(), path);
    }
}
