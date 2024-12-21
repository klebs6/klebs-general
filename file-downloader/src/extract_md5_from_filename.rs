crate::ix!();

pub fn extract_md5_from_filename(filename: impl AsRef<Path>) -> Option<String> {
    let filename_str = filename.as_ref().file_name()?.to_str()?;
    let parts = filename_str.split('.');

    for part in parts {
        // Check if `part` is exactly 32 characters long and all hex digits
        if part.len() == 32 && part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(part.to_string());
        }
    }

    None
}

#[cfg(test)]
mod extract_md5_from_filename_tests {
    use super::*;

    #[test]
    fn test_extract_md5_from_filename() {
        assert_eq!(
            extract_md5_from_filename("myfile.09f7e02f1290be211da707a266f153b3.txt"),
            Some("09f7e02f1290be211da707a266f153b3".to_string())
        );

        assert_eq!(
            extract_md5_from_filename("/some/path/to/myfile.09f7e02f1290be211da707a266f153b3"),
            Some("09f7e02f1290be211da707a266f153b3".to_string())
        );

        assert_eq!(
            extract_md5_from_filename("no_md5_here.txt"),
            None
        );
    }
}
