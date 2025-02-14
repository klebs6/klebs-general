// ---------------- [ File: src/validate_pbf_filename.rs ]
crate::ix!();

/// Validates that `pbf_path` has a filename matching what we'd expect for `region`.
/// It also checks for an optional MD5 insertion in the filename.
/// Returns an error if mismatched or if filename is invalid/unreadable.
///
/// Specifically:
/// 1. If `pbf_path.is_dir()` => error with "Invalid filename: <path> is a directory".
/// 2. If `pbf_path.file_name()` is None or non‐UTF8 => error "Invalid filename".
/// 3. Otherwise, generate the expected name for this region and compare with `filenames_match(...)`.
pub fn validate_pbf_filename(
    region:   &WorldRegion,
    pbf_path: &Path,
) -> Result<(), OsmPbfParseError> {

    // (1) If the path is actually a directory, fail with "Invalid filename".
    if pbf_path.is_dir() {
        return Err(OsmPbfParseError::InvalidInputFile {
            reason: format!("Invalid filename: {:?} is a directory", pbf_path),
        });
    }

    // (2) Attempt to extract the actual filename
    let actual_filename = pbf_path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| OsmPbfParseError::InvalidInputFile {
            reason: format!("Invalid filename: {:?}", pbf_path),
        })?;

    // (3) Generate the “expected” filename (e.g. "maryland-latest.osm.pbf") using "." as a base dir
    let expected_path = expected_filename_for_region(Path::new("."), region.download_link())?;
    let expected_filename_str = expected_path.to_str().unwrap_or_default();

    // (4) Compare using filenames_match(...).
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

// ---------------- [ Tests for validate_pbf_filename ]
#[cfg(test)]
mod test_validate_pbf_filename {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    /// A sample region that leads to "maryland-latest.osm.pbf" as its expected filename.
    /// Adjust per your actual code.
    fn maryland_region() -> WorldRegion {
        // e.g. USRegion::UnitedState(UnitedState::Maryland).into()
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    #[traced_test]
    fn test_exact_filename_match_ok() {
        let pbf_path = PathBuf::from("maryland-latest.osm.pbf");
        let region = maryland_region();
        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(result.is_ok(), "Exact match => should be Ok(())");
    }

    #[traced_test]
    fn test_case_insensitive_match_ok() {
        let pbf_path = PathBuf::from("MaRyLaNd-LaTeSt.oSm.PbF");
        let region = maryland_region();
        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(result.is_ok(), "Case-insensitive => Ok");
    }

    #[traced_test]
    fn test_optional_md5_block_ok() {
        let pbf_path = PathBuf::from("maryland-latest.123abc45.osm.pbf");
        let region = maryland_region();
        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(result.is_ok(), "md5 substring => Ok");
    }

    #[traced_test]
    fn test_leading_dot_slash_ignored_ok() {
        let pbf_path = PathBuf::from("./maryland-latest.osm.pbf");
        let region = maryland_region();
        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(result.is_ok(), "Leading './' => Ok");
    }

    #[traced_test]
    fn test_mismatch_returns_error() {
        // e.g. "virginia-latest.osm.pbf" but region => maryland => mismatch
        let pbf_path = PathBuf::from("virginia-latest.osm.pbf");
        let region = maryland_region();
        let result = validate_pbf_filename(&region, &pbf_path);
        match result {
            Err(OsmPbfParseError::InvalidInputFile { reason }) => {
                assert!(
                    reason.contains("does not match expected filename"),
                    "Reason should mention mismatch"
                );
            }
            other => panic!("Expected mismatch => InvalidInputFile, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_path_is_directory_returns_error_on_filename_extraction() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let region = maryland_region();
        let result = validate_pbf_filename(&region, temp_dir.path());

        match result {
            Err(OsmPbfParseError::InvalidInputFile { reason }) => {
                assert!(
                    reason.contains("Invalid filename"),
                    "Expected error about invalid filename"
                );
            }
            other => panic!("Expected InvalidInputFile, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_no_file_name_component_returns_error() {
        #[cfg(unix)]
        {
            // On Unix, PathBuf::from("/") => no file_name
            let region = maryland_region();
            let root_path = PathBuf::from("/");
            let result = validate_pbf_filename(&region, &root_path);
            assert!(result.is_err(), "Should fail for root path => no filename");
        }
    }

    #[traced_test]
    fn test_file_name_unreadable_in_utf8_returns_error() {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            let region = maryland_region();
            let invalid_utf8 = std::ffi::OsString::from_vec(vec![0xf0, 0x9f, 0x92, 0x96, 0xff]);
            let path = PathBuf::from(invalid_utf8);
            let result = validate_pbf_filename(&region, &path);
            match result {
                Err(OsmPbfParseError::InvalidInputFile { reason }) => {
                    assert!(
                        reason.contains("Invalid filename"),
                        "Should mention invalid filename conversion"
                    );
                }
                other => panic!("Expected InvalidInputFile, got {:?}", other),
            }
        }
    }
}
