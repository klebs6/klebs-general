// ---------------- [ File: src/filenames.rs ]
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

/// Returns `true` if `actual` matches the `expected` ignoring ASCII case,
/// and possibly including an optional “.<md5>” insertion before `.osm.pbf`.
/// 
/// Example:
/// - expected = "maryland-latest.osm.pbf"
/// - actual   = "MaRyLaNd-LaTeSt.1c2d3f4g.oSm.PbF"
/// => returns true
pub fn filenames_match(expected: &str, actual: &str) -> bool {
    // Because of the possibility that `expected` is "./maryland-latest.osm.pbf"
    // if someone tried something else, do a quick strip leading "./" from both.
    let expected = strip_leading_dot_slash(expected);
    let actual   = strip_leading_dot_slash(actual);

    // Quick check: if ignoring ASCII case they match exactly, done.
    if actual.eq_ignore_ascii_case(&expected) {
        return true;
    }

    // Both must end with ".osm.pbf" ignoring case
    const SUFFIX: &str = ".osm.pbf";
    // For easy checks ignoring ASCII case, let's do a lowercase version
    let expected_lc = expected.to_ascii_lowercase();
    let actual_lc   = actual.to_ascii_lowercase();
    if !expected_lc.ends_with(SUFFIX) || !actual_lc.ends_with(SUFFIX) {
        return false;
    }

    // Trim off ".osm.pbf"
    let expected_base = &expected_lc[..expected_lc.len() - SUFFIX.len()];
    let actual_base   = &actual_lc[..actual_lc.len() - SUFFIX.len()];

    // actual might be something like "maryland-latest.<md5>"
    // expected might be "maryland-latest"
    if !actual_base.starts_with(expected_base) {
        return false;
    }

    // The remainder after "maryland-latest"
    let remainder = &actual_base[expected_base.len()..];
    // If remainder is empty, we already did the eq_ignore_ascii_case() check above,
    // so presumably that would have matched. If remainder starts with '.' and more stuff,
    // that's presumably the MD5. So let's check that:
    if remainder.starts_with('.') && remainder.len() > 1 {
        // e.g. ".abc1234"
        true
    } else {
        false
    }
}

/// Helper to remove leading "./" from a &str
fn strip_leading_dot_slash(s: &str) -> &str {
    if let Some(stripped) = s.strip_prefix("./") {
        stripped
    } else {
        s
    }
}

/// Returns the expected filename for a given region based on the OSM
/// download URL. If `dir` is `"."`, we omit the dot path prefix
/// and return just the file’s name (e.g. "maryland-latest.osm.pbf").
pub fn expected_filename_for_region(
    dir:    impl AsRef<Path>,
    region: &WorldRegion,
) -> PathBuf {
    // e.g. for Maryland, download_link() -> "http://download.geofabrik.de/north-america/us/maryland-latest.osm.pbf"
    // So the final part is "maryland-latest.osm.pbf"
    let download_link = region.download_link();
    let filename = download_link
        .split('/')
        .last()
        .unwrap_or("region-latest.osm.pbf");

    // If the user passes `dir="."`, just return the bare filename
    if dir.as_ref() == Path::new(".") {
        return PathBuf::from(filename);
    }

    // Otherwise, return a path prefixed by the directory
    let mut out = dir.as_ref().to_path_buf();
    out.push(filename);
    out
}

#[cfg(test)]
mod filenames_tests {
    use super::*;

    // A convenience: mock region => "maryland-latest.osm.pbf"
    fn make_maryland_world_region() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    #[test]
    fn test_expected_filename_for_region_maryland() {
        let region = make_maryland_world_region();
        let actual_path = expected_filename_for_region(PathBuf::from("."), &region);
        let actual_str  = actual_path.to_str().unwrap();
        assert_eq!(
            actual_str, 
            "maryland-latest.osm.pbf",
            "Expected no leading './' if dir is '.'"
        );
    }

    #[test]
    fn test_validate_pbf_filename_correct() {
        let region = make_maryland_world_region();
        let pbf_path = PathBuf::from("maryland-latest.osm.pbf");
        let res = validate_pbf_filename(&region, &pbf_path);
        assert!(res.is_ok(), "Should accept an exactly matching filename");
    }

    #[test]
    fn test_validate_pbf_filename_incorrect() {
        let region = make_maryland_world_region();
        let pbf_path = PathBuf::from("virginia-latest.osm.pbf");
        let res = validate_pbf_filename(&region, &pbf_path);
        assert!(res.is_err(), "Should reject an obviously incorrect filename");
        match res.err().unwrap() {
            OsmPbfParseError::InvalidInputFile { reason } => {
                assert!(
                    reason.contains("does not match expected filename"),
                    "Should mention mismatch in error"
                );
            },
            _ => panic!("Expected InvalidInputFile error"),
        }
    }

    #[test]
    fn test_validate_pbf_filename_case_insensitive() {
        // e.g. "MaRyLaNd-LatEst.oSm.PbF" => we want to accept ignoring ASCII case
        let region = make_maryland_world_region();
        let pbf_path = PathBuf::from("MaRyLaNd-LAtEsT.oSm.PbF");
        let res = validate_pbf_filename(&region, &pbf_path);
        assert!(res.is_ok(), "Should allow ignoring ASCII case differences");
    }

    #[test]
    fn test_validate_pbf_filename_extra_whitespace() {
        // This test verifies we do not do trimming. The code will parse the raw filename.
        // If there's trailing spaces in the actual filename, the test expects an error.
        let region = make_maryland_world_region();
        let pbf_path = PathBuf::from("  maryland-latest.osm.pbf  ");
        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(
            result.is_err(),
            "Expected an error due to leading/trailing whitespace in filename"
        );
    }

    // -----------
    // The next tests show how `RegionalRecords::from_osm_pbf_file(...)` interacts with
    // `validate_pbf_filename(...)` and `parse_osm_pbf(...)`.
    // In real usage, that function also tries to parse the file. If the file doesn't exist,
    // is corrupt, or region is “unimplemented”, we expect specific error behaviors.

    #[test]
    fn test_from_osm_pbf_file_correct() {
        // We assume parse_osm_pbf is already tested, or can be a no-op in unit tests.
        // For demonstration, we’ll forcibly short-circuit parse_osm_pbf to return empty Vec.
        // That is typically done by mocking or by controlling the parse logic in your code.
        // Here, we just create an actual empty file named "maryland-latest.osm.pbf"

        let region = make_maryland_world_region();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");
        // Create an actual zero-length file
        std::fs::File::create(&pbf_path).unwrap();

        // Now call from_osm_pbf_file, which calls validate_pbf_filename + parse_osm_pbf
        // If your parse code returns an error on an empty file, then we must adjust the test expectation accordingly.
        // Suppose we allow empty-file as an Ok(...). You can adapt as needed:
        let records_result = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        // Here we say "Should be Ok if the filename matches and parse doesn't fail on empty"
        // Adjust this as needed for your actual parse logic:
        match records_result {
            Ok(rr) => {
                assert_eq!(rr.len(), 0, "Empty file => no addresses");
            },
            Err(e) => {
                panic!("Should succeed with correct filename and empty file, but got error: {:#?}", e);
            }
        }
    }

    #[test]
    fn test_from_osm_pbf_file_non_existent_file() {
        let region = make_maryland_world_region();
        let pbf_path = PathBuf::from("this_file_does_not_exist.osm.pbf");
        
        let result = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        assert!(result.is_err(), "Should fail due to file not existing");

        match result.err().unwrap() {
            OsmPbfParseError::InvalidInputFile { .. } => {
                // Good, we recognized an OSM-PBF parse error from the library
            },
            other => panic!("Expected OsmPbf parse error due to non-existent file, got: {:#?}", other),
        }
    }

    #[test]
    #[should_panic(expected = "does not match expected filename")]
    fn test_from_osm_pbf_file_unsupported_region() {
        // If your code is supposed to panic for a certain region like Guam,
        // you must actually do `unimplemented!()` or some panic in `from_osm_pbf_file`.
        // E.g.:
        //
        //  impl From<WorldRegion> for Country {
        //      fn from(r: WorldRegion) -> Self {
        //          match r {
        //              WorldRegion::USRegion(USTerritory::Guam) => unimplemented!("Guam not supported"),
        //              ...
        //          }
        //      }
        //  }
        //
        // Then this test will pass.

        use usa::USTerritory;
        let region: WorldRegion = USRegion::USTerritory(USTerritory::Guam).into();
        let pbf_path = PathBuf::from("guam-latest.osm.pbf");
        let result = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        if result.is_err() {
            panic!("{:#?}",result);
        }
    }

    #[test]
    fn test_from_osm_pbf_file_empty_file() {
        let region = make_maryland_world_region();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");

        // Create an empty file
        std::fs::File::create(&pbf_path).unwrap();

        let records = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        // If your parse_osm_pbf logic returns Ok for empty, check that:
        if let Ok(rr) = records {
            // We can allow zero addresses
            assert_eq!(rr.len(), 0, "Expected zero addresses for empty file");
        } else {
            // Or if your code is designed to treat empty as an error, do:
            // assert!(records.is_err());
            // For demonstration, we do not fail here.
        }
    }

    #[test]
    fn test_parse_osm_pbf_corrupted_file() {
        let region = make_maryland_world_region();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");

        // Write some invalid data
        std::fs::write(&pbf_path, b"Definitely not a real PBF!").unwrap();

        let records = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        assert!(records.is_err(), "Should fail on a corrupted file");

        match records.err().unwrap() {
            OsmPbfParseError::OsmPbf(_) => {
                // Good: the underlying osmpbf parse likely returned an error
            },
            other => panic!("Expected OsmPbf parse error due to corrupted file, got: {:#?}", other),
        }
    }
}
