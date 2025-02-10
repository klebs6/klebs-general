// ---------------- [ File: src/expected_filename_for_region.rs ]
crate::ix!();

/// Returns the "expected" filename for a region by extracting the final
/// segment after the last slash in `download_link()`.
///
/// - If the link is empty, or if the final segment is empty (i.e. link ends with `/`),
///   we return `Err(ExpectedFilenameError::NoValidFilename)`.
/// - Otherwise, we append that filename to `dir`, unless `dir == "."`, in which
///   case we just return the bare filename.
pub fn expected_filename_for_region(
    dir: impl AsRef<Path>,
    full_link: &str,
) -> Result<PathBuf, ExpectedFilenameError> {

    // 1) Check if the link is empty
    if full_link.is_empty() {
        return Err(ExpectedFilenameError::NoValidFilename);
    }

    // 2) Extract the final segment after the last slash
    let (prefix, suffix) = match full_link.rsplit_once('/') {
        None => {
            // No slash => entire link is “the final segment”
            ("", full_link)
        }
        Some((left, right)) => (left, right),
    };

    // 3) If suffix is empty => link ended with slash => no valid filename
    //    e.g. "http://host/dir/"
    if suffix.is_empty() {
        return Err(ExpectedFilenameError::NoValidFilename);
    }

    // Now `suffix` is presumably something like "maryland-latest.osm.pbf" or "somefile.zip"
    //  => that’s our final “filename.”

    // 4) If dir=".", return just that suffix
    if dir.as_ref() == Path::new(".") {
        return Ok(PathBuf::from(suffix));
    }

    // 5) Otherwise, join dir + suffix
    let mut out = dir.as_ref().to_path_buf();
    out.push(suffix);
    Ok(out)
}

// ---------------- [ tests/expected_filename_for_region_tests.rs ]

#[cfg(test)]
mod expected_filename_for_region_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_filename_dir_is_dot_success() {
        let link = "http://download.geofabrik.de/north-america/us/maryland-latest.osm.pbf";
        let result = expected_filename_for_region(".", &link);
        assert!(result.is_ok(), "Should succeed with a normal link");
        let path = result.unwrap();
        assert_eq!(path.to_str().unwrap(), "maryland-latest.osm.pbf");
    }

    #[test]
    fn test_filename_custom_dir() {
        let link = "http://download.geofabrik.de/north-america/us/maryland-latest.osm.pbf";
        let result = expected_filename_for_region("/some/path", &link);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.to_str().unwrap(), "/some/path/maryland-latest.osm.pbf");
    }

    #[test]
    fn test_no_slashes_in_link() {
        // entire link is "just_a_name_without_slash", no slash => that's the final segment
        // => dir="." => we yield "just_a_name_without_slash"
        let link = "just_a_name_without_slash";
        let result = expected_filename_for_region(".", &link);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().to_str().unwrap(),
            "just_a_name_without_slash"
        );
    }

    #[test]
    fn test_link_has_query() {
        // "maryland-latest.osm.pbf?v=123" => suffix is "maryland-latest.osm.pbf?v=123"
        // => we do not parse the query; we keep it in the filename
        let link = "http://fakehost/maryland-latest.osm.pbf?v=123";
        let result = expected_filename_for_region(".", &link);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().to_str().unwrap(),
            "maryland-latest.osm.pbf?v=123"
        );
    }

    #[test]
    fn test_empty_link_error() {
        // link="" => no valid filename => error
        let link = "";
        let result = expected_filename_for_region(".", &link);
        assert!(result.is_err(), "Should be an error for empty link");
        match result.err().unwrap() {
            ExpectedFilenameError::NoValidFilename => {
                // OK
            }
        }
    }

    #[test]
    fn test_link_ends_with_slash_error() {
        // e.g. "http://host/dir/" => suffix = "" => error
        let link = "http://host/dir/";
        let result = expected_filename_for_region(".", &link);
        assert!(result.is_err(), "Ending slash => no valid final segment => error");
        match result.err().unwrap() {
            ExpectedFilenameError::NoValidFilename => {
                // OK
            }
        }
    }
}
