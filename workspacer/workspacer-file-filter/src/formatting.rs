crate::ix!();

pub fn filename_to_snake<P>(file_path: &P) -> String
where
    P: Debug + AsRef<Path>,
{
    // Convert the entire path to a string for logging.
    let path_str = file_path.as_ref().to_string_lossy();
    tracing::trace!("Processing path: {}", path_str);

    // If the input path ends with a directory separator (e.g. "/" or "\"),
    // treat it as a directory and return an empty string.
    if path_str.ends_with('/') || path_str.ends_with('\\') {
        return String::new();
    }

    // Extract the final component (the file name).
    let file_name = match file_path.as_ref().file_name() {
        Some(name) => name.to_string_lossy(),
        None => return String::new(),
    };

    // If the filename starts with a dot (i.e. it's a dotfile), return empty.
    if file_name.starts_with('.') {
        return String::new();
    }

    // If the file has no extension, we consider it not a valid file for our purposes;
    // return empty.
    if file_path.as_ref().extension().is_none() {
        return String::new();
    }

    // Extract the file stem (filename without extension).
    let stem = match file_path.as_ref().file_stem() {
        Some(stem) => stem.to_string_lossy(),
        None => return String::new(),
    };

    if stem.is_empty() {
        return String::new();
    }

    // Step 1: Insert an underscore before uppercase letters when preceded by a lowercase letter.
    let mut processed = String::new();
    let mut prev_was_lower = false;
    for (i, ch) in stem.char_indices() {
        if i > 0 && ch.is_ascii_uppercase() && prev_was_lower {
            processed.push('_');
        }
        // Update flag: ch is considered lowercase if it is indeed lowercase.
        prev_was_lower = ch.is_ascii_lowercase();
        processed.push(ch);
    }

    // Step 2: Build the result by converting to lowercase and replacing any character
    // not in the set [a-z0-9_] with an underscore.
    let mut result = String::new();
    for ch in processed.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            result.extend(ch.to_lowercase());
        } else {
            result.push('_');
        }
    }

    // Remove any trailing underscores from the result.
    result = result.trim_end_matches('_').to_string();

    tracing::debug!("Final snake_case result: {}", result);
    result
}

#[cfg(test)]
mod formatting_tests {
    use super::*;

    #[test]
    fn test_simple_case() {
        // "SomeFile.rs" should become "some_file"
        assert_eq!(filename_to_snake(&"SomeFile.rs"), "some_file");
    }

    #[test]
    fn test_spaces_and_extension() {
        // "User Data.txt" should become "user_data"
        assert_eq!(filename_to_snake(&"User Data.txt"), "user_data");
    }

    #[test]
    fn test_weird_file() {
        // "my-file(1).TXT" should become "my_file_1" (trailing underscore trimmed)
        assert_eq!(filename_to_snake(&"my-file(1).TXT"), "my_file_1");
    }

    #[test]
    fn test_dotfile() {
        // Dotfiles (e.g. ".profile") should return an empty string.
        assert_eq!(filename_to_snake(&".profile"), "");
        assert_eq!(filename_to_snake(&".gitignore"), "");
    }

    #[test]
    fn test_already_snake_case() {
        // Files that are already in snake_case should remain unchanged.
        // Note: multiple underscores (if already present) are preserved.
        assert_eq!(filename_to_snake(&"already_snake_case.rs"), "already_snake_case");
        assert_eq!(filename_to_snake(&"multiple__underscores.html"), "multiple__underscores");
    }

    #[test]
    fn test_no_filename() {
        // If the input has no extension (like a directory name, e.g., "src"), we return empty.
        assert_eq!(filename_to_snake(&"src"), "");
        assert_eq!(filename_to_snake(&"/some/directory"), "");
    }

    #[test]
    fn test_directory_is_ignored() {
        // Even if the file is inside directories, only the final file stem is processed.
        // E.g., "folder/subfolder/MyServiceHTTPS.rs" should yield "my_service_https"
        let path = Path::new("folder/subfolder/MyServiceHTTPS.rs");
        assert_eq!(filename_to_snake(&path), "my_service_https");
    }
}
