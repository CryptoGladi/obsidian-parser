use crate::{
    prelude::{FilesBuilder, IteratorFilesBuilder, VaultOptions},
    vault::Vault,
};
use std::{fs::File, io::Write};
use tempfile::TempDir;

pub(crate) fn create_files_for_vault() -> Result<(TempDir, Vec<File>), std::io::Error> {
    let temp_dir = TempDir::new()?;

    static TEST_MAIN_DATA: &[u8] =
        b"---\ntopic: work\ncreated: 15-04-2006\n---\nMain data. Other [[data/main|main]]";
    static TEST_LINK_DATA: &[u8] = b"---\ntopic: kinl\ncreated: 15-04-2006\n---\n[[main]]";

    let mut main = File::create(temp_dir.path().join("main.md"))?;
    let mut link = File::create(temp_dir.path().join("link.md"))?;
    main.write_all(TEST_MAIN_DATA)?;
    link.write_all(TEST_LINK_DATA)?;

    std::fs::create_dir(temp_dir.path().join("data"))?;
    let mut main2 = File::create(temp_dir.path().join("data").join("main.md"))?;
    main2.write_all(b"New main. [[link]]")?;

    #[cfg(feature = "logging")]
    log::debug!(
        "Created test files for vault in: {}",
        temp_dir.path().display()
    );

    Ok((temp_dir, vec![main, main2, link]))
}

pub(crate) fn create_test_vault() -> Result<(Vault, TempDir), std::io::Error> {
    let (path, _) = create_files_for_vault()?;

    let options = VaultOptions::new(&path);
    let vault = FilesBuilder::new(&options)
        .into_iter()
        .map(|file| file.unwrap())
        .build_vault(&options)
        .unwrap();

    Ok((vault, path))
}
