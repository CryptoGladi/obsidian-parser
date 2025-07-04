use std::{fs::File, io::Write};
use tempfile::TempDir;

pub fn create_test_vault() -> Result<(TempDir, Vec<File>), std::io::Error> {
    let temp_dir = TempDir::new()?;

    static TEST_MAIN_DATA: &[u8] = b"---\ntopic: work\ncreated: 15-04-2006\n---\nMain data";
    static TEST_LINK_DATA: &[u8] = b"---\ntopic: kinl\ncreated: 15-04-2006\n---\n[[main]]";

    let mut main = File::create(temp_dir.path().join("main.md"))?;
    let mut link = File::create(temp_dir.path().join("link.md"))?;
    main.write_all(TEST_MAIN_DATA)?;
    link.write_all(TEST_LINK_DATA)?;

    Ok((temp_dir, vec![main, link]))
}
