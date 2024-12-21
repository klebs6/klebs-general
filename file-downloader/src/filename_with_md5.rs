crate::ix!();

pub fn filename_with_md5(filename: impl AsRef<Path>, md5: &str) -> PathBuf {
    let original_str = filename.as_ref().to_str().unwrap();
    let parts: Vec<&str> = original_str.rsplitn(3, '.').collect();

    if parts.len() == 3 {
        PathBuf::from(format!("{}.{}.{}.{}", parts[2], md5, parts[1], parts[0]))
    } else {
        PathBuf::from(format!("{}.{}", original_str, md5))
    }
}
