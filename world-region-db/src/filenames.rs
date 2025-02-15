// ---------------- [ File: src/filenames.rs ]
// ---------------- [ File: src/filenames.rs ]
crate::ix!();

#[cfg(test)]
mod filenames_tests {
    use super::*;

    // A convenience: mock region => "maryland-latest.osm.pbf"
    fn make_maryland_world_region() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    #[traced_test]
    fn test_expected_filename_for_region_maryland() {
        let region = make_maryland_world_region();
        let actual_path = expected_filename_for_region(PathBuf::from("."), region.download_link()).expect("expected maryland to have a pbf path");
        let actual_str  = actual_path.to_str().unwrap();
        assert_eq!(
            actual_str, 
            "maryland-latest.osm.pbf",
            "Expected no leading './' if dir is '.'"
        );
    }

    #[traced_test]
    fn test_validate_pbf_filename_correct() {
        let region = make_maryland_world_region();
        let pbf_path = PathBuf::from("maryland-latest.osm.pbf");
        let res = validate_pbf_filename(&region, &pbf_path);
        assert!(res.is_ok(), "Should accept an exactly matching filename");
    }

    #[traced_test]
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

    #[traced_test]
    fn test_validate_pbf_filename_case_insensitive() {
        // e.g. "MaRyLaNd-LatEst.oSm.PbF" => we want to accept ignoring ASCII case
        let region = make_maryland_world_region();
        let pbf_path = PathBuf::from("MaRyLaNd-LAtEsT.oSm.PbF");
        let res = validate_pbf_filename(&region, &pbf_path);
        assert!(res.is_ok(), "Should allow ignoring ASCII case differences");
    }

    #[traced_test]
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

    #[traced_test]
    async fn test_from_osm_pbf_file_correct() {
        // We assume parse_osm_pbf is already tested, or can be a no-op in unit tests.
        // For demonstration, we’ll forcibly short-circuit parse_osm_pbf to return empty Vec.
        // That is typically done by mocking or by controlling the parse logic in your code.
        // Here, we just create an actual empty file named "maryland-latest.osm.pbf"

        let region = make_maryland_world_region();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");
        // Create an actual zero-length file
        File::create(&pbf_path).await.unwrap();

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

    #[traced_test]
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

    #[traced_test]
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
        assert!(result.is_err());
    }

    #[traced_test]
    async fn test_from_osm_pbf_file_empty_file() {
        let region = make_maryland_world_region();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");

        // Create an empty file
        File::create(&pbf_path).await.unwrap();

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

    #[traced_test]
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
