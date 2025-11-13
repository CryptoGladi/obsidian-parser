use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

fn is_hidden(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .file_name()
        .is_some_and(|e| e.to_str().is_some_and(|name| name.starts_with('.')))
}

fn is_md_file(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .is_some_and(|p| p.eq_ignore_ascii_case("md"))
}

pub fn get_files_for_parse(path: impl AsRef<Path>) -> Vec<PathBuf> {
    #[cfg(feature = "logging")]
    log::trace!("Get files for parse: {}", path.as_ref().display());

    let walker = WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !is_hidden(e.path()))
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_md_file(e.path()));

    walker.map(DirEntry::into_path).collect()
}

#[cfg(test)]
mod tests {
    use crate::vault::vault_test::create_test_vault;

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn is_hidden() {
        assert!(super::is_hidden(".test"));
        assert!(!super::is_hidden("test"));
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn is_md_file() {
        assert!(super::is_md_file("test.md"));
        assert!(super::is_md_file(".test.md"));

        assert!(!super::is_md_file("test.txt"));
        assert!(!super::is_md_file("test"));
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    fn get_files_for_parse() {
        let (path, files) = create_test_vault().unwrap();

        assert_eq!(super::get_files_for_parse(path.path()).len(), files.len());
    }
}
