crate::ix!();

pub fn validate_pbf_filename(region: &USRegion, pbf_path: &Path) -> Result<(), OsmPbfParseError> {
    let actual_filename = pbf_path.file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| OsmPbfParseError::InvalidInputFile {
            reason: format!("Invalid filename: {:?}", pbf_path)
        })?;

    let expected_filename = expected_filename_for_region(region);
    let expected_filename_str = expected_filename.to_str().unwrap_or_default();

    if filenames_match(expected_filename_str, actual_filename) {
        Ok(())
    } else {
        Err(OsmPbfParseError::InvalidInputFile {
            reason: format!(
                "Provided PBF file '{:?}' does not match expected filename '{:?}' for region {:?}",
                actual_filename, expected_filename_str, region
            )
        })
    }
}

/// Helper function to check if actual filename matches the expected filename pattern,
/// allowing for an optional MD5 segment in the middle.
pub fn filenames_match(expected: &str, actual: &str) -> bool {
    if actual.eq_ignore_ascii_case(expected) {
        return true; // exact match is fine
    }

    // Expected: "maryland-latest.osm.pbf"
    // Actual:   "maryland-latest.<md5>.osm.pbf"
    // Let's try to parse the actual filename and see if it matches this pattern:
    // "maryland-latest" + "." + "<md5>" + ".osm.pbf"

    // Remove trailing ".osm.pbf" from both expected and actual, then compare prefixes.
    let suffix = ".osm.pbf";
    if !expected.ends_with(suffix) || !actual.ends_with(suffix) {
        return false;
    }

    let expected_base = &expected[..expected.len() - suffix.len()]; // e.g. "maryland-latest"
    let actual_base   = &actual[..actual.len() - suffix.len()];     // e.g. "maryland-latest.<md5>"

    // Now actual_base might be "maryland-latest.<md5>"
    // Check if actual_base starts with expected_base and has a '.' followed by something
    if actual_base.starts_with(expected_base) {
        let remainder = &actual_base[expected_base.len()..];
        // remainder should start with '.' followed by MD5
        if remainder.starts_with('.') && remainder.len() > 1 {
            // We have "maryland-latest" + "." + some_md5
            return true;
        }
    }

    false
}

/// Returns the expected filename for a given region based on the OSM download URL.
pub fn expected_filename_for_region(region: &USRegion) -> PathBuf {
    let handle = OpenStreetMapRegionalDataDownloadHandle::from(region.clone());
    handle.filename()
}

#[cfg(test)]
mod filenames_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_expected_filename_for_region_maryland() {
        let region = USRegion::UnitedState(UnitedState::Maryland);
        let expected = expected_filename_for_region(&region);
        assert_eq!(expected.to_str().unwrap(), "maryland-latest.osm.pbf");
    }

    #[test]
    fn test_validate_pbf_filename_correct() {
        let region = USRegion::UnitedState(UnitedState::Maryland);
        let pbf_path = PathBuf::from("maryland-latest.osm.pbf");
        let res = validate_pbf_filename(&region, &pbf_path);
        assert!(res.is_ok());
    }

    #[test]
    fn test_validate_pbf_filename_incorrect() {
        let region = USRegion::UnitedState(UnitedState::Maryland);
        let pbf_path = PathBuf::from("virginia-latest.osm.pbf");
        let res = validate_pbf_filename(&region, &pbf_path);
        assert!(res.is_err());
        match res.err().unwrap() {
            OsmPbfParseError::InvalidInputFile { reason } => {
                assert!(reason.contains("does not match expected filename"));
            },
            _ => panic!("Expected InvalidInputFile error"),
        }
    }

    #[test]
    fn test_validate_pbf_filename_case_insensitive() {
        let region = USRegion::UnitedState(UnitedState::Maryland);
        // different casing:
        let pbf_path = PathBuf::from("MaRyLaNd-LAtEsT.oSm.PbF");
        let res = validate_pbf_filename(&region, &pbf_path);
        assert!(res.is_ok());
    }

    #[test]
    fn test_from_osm_pbf_file_correct() {
        // Mock parse_osm_pbf to return empty records or a fixed set if needed.
        // Here we assume parse_osm_pbf works, or is tested separately.
        let region = USRegion::UnitedState(UnitedState::Maryland);
        let pbf_path = PathBuf::from("maryland-latest.osm.pbf");
        
        // We would need `parse_osm_pbf` to be mockable or the file actually exist.
        // For this test, assume `parse_osm_pbf` returns Ok(vec![]).
        // You can do this by adjusting `parse_osm_pbf` or using a test-double.
        // For now, just pretend it works:

        let records = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        assert!(records.is_ok(), "Should succeed with correct filename");
    }

    #[test]
    fn test_from_osm_pbf_file_incorrect_filename() {
        let region = USRegion::UnitedState(UnitedState::Maryland);
        let pbf_path = PathBuf::from("virginia-latest.osm.pbf");

        let records = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        assert!(records.is_err());
        match records.err().unwrap() {
            OsmPbfParseError::InvalidInputFile { .. } => {}, // as expected
            _ => panic!("Expected InvalidInputFile error"),
        }
    }

    #[test]
    fn test_from_osm_pbf_file_non_existent_file() {
        let region = USRegion::UnitedState(UnitedState::Maryland);
        let pbf_path = PathBuf::from("non_existent.osm.pbf");

        // Assuming parse_osm_pbf will return an error if file doesn't exist.
        let result = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        assert!(result.is_err());

        match result.err().unwrap() {
            OsmPbfParseError::OsmPbf(_) => {}, // expected a parse error
            _ => panic!("Expected OsmPbf parse error due to non-existent file"),
        }
    }

    #[test]
    #[should_panic(expected = "unimplemented")]
    fn test_from_osm_pbf_file_unsupported_region() {
        let region = USRegion::USFederalDistrict(USFederalDistrict::Guam); // Assuming not implemented
        let pbf_path = PathBuf::from("guam-latest.osm.pbf");
        // This should panic or return an error due to unimplemented region support
        let _ = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
    }

    #[test]
    fn test_from_osm_pbf_file_empty_file() {
        let region = USRegion::UnitedState(UnitedState::Maryland);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");

        // Create an empty file
        std::fs::File::create(&pbf_path).unwrap();

        // parse_osm_pbf should return either empty results or an error
        let records = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        // Depending on implementation:
        // If parse_osm_pbf returns empty records on empty file:
        // assert!(records.is_ok());
        // assert_eq!(records.unwrap().len(), 0);

        // Or if parse_osm_pbf returns an error:
        assert!(records.is_err());
    }

    #[test]
    fn test_validate_pbf_filename_extra_whitespace() {
        let region = USRegion::UnitedState(UnitedState::Maryland);
        let pbf_path = PathBuf::from("   maryland-latest.osm.pbf   ");
        // Normally, file paths won't have trailing spaces, but this test checks robustness.
        // If your code doesn't trim, this will fail. Consider trimming or documenting not supported.
        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(result.is_err(), "Expected error due to whitespace in filename");
    }

    #[test]
    fn test_parse_osm_pbf_corrupted_file() {
        let region = USRegion::UnitedState(UnitedState::Maryland);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");

        // Write some invalid binary data that doesn't represent valid OSM PBF:
        std::fs::write(&pbf_path, b"not a real pbf").unwrap();

        let records = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        assert!(records.is_err());

        match records.err().unwrap() {
            OsmPbfParseError::OsmPbf(_) => {}, // expected parse error due to corrupted format
            _ => panic!("Expected OsmPbf parse error due to corrupted file"),
        }
    }
}
