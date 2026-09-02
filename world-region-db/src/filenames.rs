// ---------------- [ File: src/filenames.rs ]
crate::ix!();

#[cfg(test)]
mod filenames_tests {
    use super::*;

    // A convenience: mock region => "florida-latest.osm.pbf"
    fn make_florida_world_region() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Florida).into()
    }

    #[traced_test]
    fn test_expected_filename_for_region_florida() {
        info!("Testing expected filename for Florida region");
        let region = make_florida_world_region();
        let actual_path = expected_filename_for_region(PathBuf::from("."), region.download_link())
            .expect("expected florida to have a pbf path");
        let actual_str  = actual_path.to_str().unwrap();
        assert_eq!(
            actual_str,
            "florida-latest.osm.pbf",
            "Expected no leading './' if dir is '.'"
        );
    }

    #[traced_test]
    fn test_validate_pbf_filename_correct_florida() {
        info!("Testing validate_pbf_filename with correct Florida filename");
        let region = make_florida_world_region();
        let pbf_path = PathBuf::from("florida-latest.osm.pbf");
        let res = validate_pbf_filename(&region, &pbf_path);
        assert!(res.is_ok(), "Should accept an exactly matching filename");
    }

    #[traced_test]
    fn test_validate_pbf_filename_incorrect_florida() {
        info!("Testing validate_pbf_filename with incorrect filename for Florida region");
        let region = make_florida_world_region();
        let pbf_path = PathBuf::from("texas-latest.osm.pbf");
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
    fn test_validate_pbf_filename_case_insensitive_florida() {
        info!("Testing validate_pbf_filename with case-insensitive match for Florida");
        let region = make_florida_world_region();
        let pbf_path = PathBuf::from("FlOrIdA-LAtEsT.oSm.PbF");
        let res = validate_pbf_filename(&region, &pbf_path);
        assert!(res.is_ok(), "Should allow ignoring ASCII case differences");
    }

    #[traced_test]
    fn test_validate_pbf_filename_extra_whitespace_florida() {
        info!("Testing validate_pbf_filename with extra whitespace for Florida filename");
        let region = make_florida_world_region();
        let pbf_path = PathBuf::from("  florida-latest.osm.pbf  ");
        let result = validate_pbf_filename(&region, &pbf_path);
        assert!(
            result.is_err(),
            "Expected an error due to leading/trailing whitespace in filename"
        );
    }

    #[traced_test]
    async fn test_from_osm_pbf_file_correct_florida() {
        info!("Testing from_osm_pbf_file with correct Florida filename and empty file");
        let region = make_florida_world_region();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("florida-latest.osm.pbf");
        File::create(&pbf_path).await.unwrap();

        let records_result = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
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
    fn test_from_osm_pbf_file_non_existent_file_florida() {
        info!("Testing from_osm_pbf_file with non-existent Florida file");
        let region = make_florida_world_region();
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
        info!("Testing from_osm_pbf_file with an unsupported region (Guam)");
        use usa::USTerritory;
        let region: WorldRegion = USRegion::USTerritory(USTerritory::Guam).into();
        let pbf_path = PathBuf::from("guam-latest.osm.pbf");
        let result = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        assert!(result.is_err());
    }

    #[traced_test]
    async fn test_from_osm_pbf_file_empty_file_florida() {
        info!("Testing from_osm_pbf_file with an empty Florida file");
        let region = make_florida_world_region();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("florida-latest.osm.pbf");

        File::create(&pbf_path).await.unwrap();

        let records = RegionalRecords::from_osm_pbf_file(region, &pbf_path);
        if let Ok(rr) = records {
            assert_eq!(rr.len(), 0, "Expected zero addresses for empty file");
        }
    }

    #[traced_test]
    fn test_parse_osm_pbf_corrupted_file_florida() {
        info!("Testing from_osm_pbf_file with a corrupted Florida file");
        let region = make_florida_world_region();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("florida-latest.osm.pbf");

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
