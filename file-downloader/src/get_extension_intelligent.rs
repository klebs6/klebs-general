crate::ix!();

/// Retrieves the intelligent full extension from a given filename, supporting known multi-segment extensions like `.tar.gz`.
///
/// This function checks the filename against a predefined list of compound extensions. If the filename ends with one of
/// these compound extensions, it returns that extension. Otherwise, it falls back to returning the standard single extension,
/// prefixed with a dot.
///
/// # Arguments
///
/// * `filename` - A type that can be referenced as a `Path`.
///
/// # Returns
///
/// * `Option<String>` - Returns `Some(full_extension)` with the dot if an extension exists, otherwise `None`.
///
/// # Examples
///
/// ```
/// use file_downloader::get_extension_intelligent;
/// let ext = get_extension_intelligent("filename.030304949j4kl2394jij94534.tar.gz");
/// assert_eq!(ext, Some(".tar.gz".to_string()));
///
/// let ext_single = get_extension_intelligent("document.pdf");
/// assert_eq!(ext_single, Some(".pdf".to_string()));
///
/// let ext_no = get_extension_intelligent("README");
/// assert_eq!(ext_no, None);
///
/// let ext_hidden = get_extension_intelligent(".gitignore");
/// assert_eq!(ext_hidden, None);
///
/// let ext_multi_dot_hidden = get_extension_intelligent("..config");
/// assert_eq!(ext_multi_dot_hidden, None);
/// ```
pub fn get_extension_intelligent<P: AsRef<Path>>(filename: P) -> Option<String> {
    // Define a list of known compound extensions, sorted by length descending
    const COMPOUND_EXTENSIONS: &[&str] = &[
        ".tar.gz",
        ".tar.bz2",
        ".tar.xz",
        ".tar.Z",
        ".tar.lz",
        ".tar.lzma",
        ".tar.lzo",
        ".tar.sz",
        ".tar.zst",
        ".user.js",
        ".min.js",
        ".min.css",
        ".map.js",
        ".test.js",
        ".osm.pbf",
        // Add more compound extensions as needed
    ];

    let path = filename.as_ref();
    let file_name = path.file_name()?.to_str()?;

    // Handle hidden files:
    // - If the filename starts with two dots, treat it as having no extension.
    // - If it starts with one dot and has only one dot, treat it as having no extension.
    // - If it starts with one dot and has multiple dots, proceed to extract the extension.
    if file_name.starts_with("..") {
        // Filenames starting with two or more dots are treated as having no extension
        return None;
    } else if file_name.starts_with('.') {
        // Check if there's only one dot in the filename
        if file_name.matches('.').count() == 1 {
            return None;
        }
    }

    // Convert to lowercase for case-insensitive comparison
    let lower_file_name = file_name.to_lowercase();

    // Iterate through compound extensions and check if filename ends with any
    for &compound_ext in COMPOUND_EXTENSIONS.iter() {
        if lower_file_name.ends_with(compound_ext) {
            let start = file_name.len() - compound_ext.len();
            // Extract the compound extension while preserving original casing
            return Some(file_name[start..].to_string());
        }
    }

    // Fallback to single extension, prefixed with a dot
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|_| {
            let dot_index = file_name.rfind('.')?;
            // Ensure there's something after the dot and it's not the first character
            if dot_index + 1 >= file_name.len() || dot_index == 0 {
                return None;
            }
            // Extract the extension, prefixed with a dot
            Some(file_name[dot_index..].to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extension_intelligent() {
        // Compound extensions
        assert_eq!(
            get_extension_intelligent("filename.030304949j4kl2394jij94534.tar.gz"),
            Some(".tar.gz".to_string())
        );
        assert_eq!(
            get_extension_intelligent("archive.tar.bz2"),
            Some(".tar.bz2".to_string())
        );
        assert_eq!(
            get_extension_intelligent("backup.2023.04.01.tar.xz"),
            Some(".tar.xz".to_string())
        );
        assert_eq!(
            get_extension_intelligent("script.min.js"),
            Some(".min.js".to_string())
        );
        assert_eq!(
            get_extension_intelligent("styles.min.css"),
            Some(".min.css".to_string())
        );
        assert_eq!(
            get_extension_intelligent("source.map.js"),
            Some(".map.js".to_string())
        );
        assert_eq!(
            get_extension_intelligent("code.test.js"),
            Some(".test.js".to_string())
        );
        assert_eq!(
            get_extension_intelligent("data.osm.pbf"),
            Some(".osm.pbf".to_string())
        );

        // Single extensions
        assert_eq!(
            get_extension_intelligent("document.pdf"),
            Some(".pdf".to_string())
        );
        assert_eq!(
            get_extension_intelligent("photo.jpeg"),
            Some(".jpeg".to_string())
        );
        assert_eq!(
            get_extension_intelligent("video.mp4"),
            Some(".mp4".to_string())
        );

        // No extension
        assert_eq!(
            get_extension_intelligent("README"),
            None
        );
        assert_eq!(
            get_extension_intelligent("LICENSE"),
            None
        );

        // Hidden files without extensions
        assert_eq!(
            get_extension_intelligent(".gitignore"),
            None
        );
        assert_eq!(
            get_extension_intelligent(".env"),
            None
        );

        // Filenames ending with a dot
        assert_eq!(
            get_extension_intelligent("file."),
            None
        );
        assert_eq!(
            get_extension_intelligent("archive.tar."),
            None
        );

        // Complex paths
        assert_eq!(
            get_extension_intelligent("folder/name.with.dots/file.tar.bz2"),
            Some(".tar.bz2".to_string())
        );
        assert_eq!(
            get_extension_intelligent("path.to/some.folder/file.name.with.many.dots.min.js"),
            Some(".min.js".to_string())
        );

        // Edge cases
        assert_eq!(
            get_extension_intelligent("a.b.c.d.e"),
            Some(".e".to_string()) // Not in the compound list, so returns the last extension
        );
        assert_eq!(
            get_extension_intelligent("..config"),
            None
        );
        assert_eq!(
            get_extension_intelligent("."),
            None
        );
        assert_eq!(
            get_extension_intelligent("..."),
            None
        );

        // Not matching compound extensions
        assert_eq!(
            get_extension_intelligent("file.tar.bz3"),
            Some(".bz3".to_string()) // Not in the compound list, so it falls back to the last extension
        );
    }
}
