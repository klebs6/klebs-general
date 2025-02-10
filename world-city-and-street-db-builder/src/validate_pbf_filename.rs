// ---------------- [ File: src/validate_pbf_filename.rs ]
// ---------------- [ File: src/validate_pbf_filename.rs ]
crate::ix!();

/// Validates that `pbf_path` has a filename matching what we'd expect for `region`.
/// It also checks for optional MD5 insertion in the filename.
/// Returns an error if mismatched or if filename is invalid/unreadable.
pub fn validate_pbf_filename(
    region:   &WorldRegion,
    pbf_path: &Path,
) -> Result<(), OsmPbfParseError> {
    // Attempt to extract the actual filename
    let actual_filename = pbf_path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| OsmPbfParseError::InvalidInputFile {
            reason: format!("Invalid filename: {:?}", pbf_path),
        })?;

    // Generate the “expected” filename. We pass `dir="."` to indicate “no path prefix,”
    // then remove that prefix from the result.
    let expected_path = expected_filename_for_region(Path::new("."), region);
    let expected_filename_str = expected_path.to_str().unwrap_or_default();

    // If these do not match (case-insensitive, optional MD5 block), return error
    if filenames_match(expected_filename_str, actual_filename) {
        Ok(())
    } else {
        Err(OsmPbfParseError::InvalidInputFile {
            reason: format!(
                "Provided PBF file '{:?}' does not match expected filename '{:?}' for region {:?}",
                actual_filename, 
                expected_filename_str, 
                region
            ),
        })
    }
}

#[cfg(test)]
#[disable]
mod test_validate_pbf_filename {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    /// A helper region that we know yields `"maryland-latest.osm.pbf"` from
    /// `expected_filename_for_region(Path::new("."), region)`. 
    /// Replace with your real region, e.g. 
    /// `USRegion::UnitedState(UnitedState::Maryland).into()`.
    fn maryland_region() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    #[traced_test]
    fn test_exact_filename_match_ok() {
        // If the expected filename is "maryland-latest.osm.pbf" 
        // and actual is exactly that, ignoring ASCII case => Ok(()).

        // We'll define a path with that exact file name. 
        let pbf_path = PathBuf::from("maryland-latest.osm.pbf");
        let region = maryland_region();

        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(result.is_ok(), "Exact match => should be Ok(())");
    }

    #[traced_test]
    fn test_case_insensitive_match_ok() {
        // e.g. "MaRyLaNd-LaTeSt.oSm.PbF" => ignoring case => ok
        let pbf_path = PathBuf::from("MaRyLaNd-LaTeSt.oSm.PbF");
        let region = maryland_region();

        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(result.is_ok(), "Case-insensitive match => should be Ok");
    }

    #[traced_test]
    fn test_optional_md5_block_ok() {
        // e.g. "maryland-latest.123abc45.oSm.PbF"
        // If filenames_match(...) function allows "maryland-latest.<md5>.osm.pbf", it should pass
        let pbf_path = PathBuf::from("maryland-latest.123abc45.osm.pbf");
        let region = maryland_region();

        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(result.is_ok(), "Filename with optional .<md5> => Ok");
    }

    #[traced_test]
    fn test_leading_dot_slash_ignored_ok() {
        // e.g. "./maryland-latest.osm.pbf"
        // The code strips leading "./" if present. => should pass
        let pbf_path = PathBuf::from("./maryland-latest.osm.pbf");
        let region = maryland_region();

        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(result.is_ok(), "Leading './' => ignored => Ok");
    }

    #[traced_test]
    fn test_mismatch_returns_error() {
        // e.g. "virginia-latest.osm.pbf" but region => maryland => error
        let pbf_path = PathBuf::from("virginia-latest.osm.pbf");
        let region = maryland_region();

        let result = validate_pbf_filename(&region, &pbf_path);
        match result {
            Err(OsmPbfParseError::InvalidInputFile { reason }) => {
                assert!(
                    reason.contains("does not match expected filename"),
                    "Error reason should mention mismatch"
                );
            }
            other => panic!("Expected InvalidInputFile mismatch error, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_path_is_directory_returns_error_on_filename_extraction() {
        // If the path is a directory, .file_name() => None => we get InvalidInputFile
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let region = maryland_region();

        let result = validate_pbf_filename(&region, temp_dir.path());
        match result {
            Err(OsmPbfParseError::InvalidInputFile { reason }) => {
                assert!(reason.contains("Invalid filename"), "Expected error about invalid filename");
            }
            other => panic!("Expected InvalidInputFile, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_no_file_name_component_returns_error() {
        // Another scenario: PathBuf::from("/") => file_name => None
        // e.g. on Unix, root path => no filename
        #[cfg(unix)]
        {
            let region = maryland_region();
            let root_path = PathBuf::from("/");
            let result = validate_pbf_filename(&region, &root_path);
            assert!(result.is_err(), "Should yield an InvalidInputFile error for root path");
        }
    }

    #[traced_test]
    fn test_file_name_unreadable_in_utf8_returns_error() {
        // On some platforms, you could have non-UTF8 filenames => .to_str() => None => error
        // We'll simulate that with ill-formed UTF-8. 
        // On many standard filesystems it's tricky. We'll do a partial approach on Unix:

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;

            let region = maryland_region();
            // Bytes: [0xf0, 0x9f, 0x92, 0x96, 0xff] => the last 0xff is invalid in UTF-8
            let invalid_utf8 = std::ffi::OsString::from_vec(vec![0xf0, 0x9f, 0x92, 0x96, 0xff]);
            let path = PathBuf::from(invalid_utf8);

            let result = validate_pbf_filename(&region, &path);
            match result {
                Err(OsmPbfParseError::InvalidInputFile { reason }) => {
                    assert!(reason.contains("Invalid filename"), "Should mention invalid filename conversion");
                }
                other => panic!("Expected InvalidInputFile for non-UTF8 path, got {:?}", other),
            }
        }
    }
}
