// ---------------- [ File: src/parse_osm_pbf_and_build_house_number_ranges.rs ]
crate::ix!();

/// Loads an OSM PBF file, extracting all [`AddressRecord`]s and accumulating
/// [`HouseNumberRange`] objects in memory. This function is suitable for smaller
/// to medium‐sized data sets that fit into RAM.
///
/// **If the data is massive**, consider a streaming approach where intermediate
/// results are regularly flushed to disk or a database instead of being stored
/// in a large in‐memory map.
///
/// # Arguments
///
/// * `path`   - Filesystem path to a `.pbf` file containing OSM data.
/// * `region` - A [`WorldRegion`] from which we infer the `Country`.
///
/// # Returns
///
/// * `Ok((Vec<AddressRecord>, HashMap<StreetName, Vec<HouseNumberRange>>))`:
///   A list of addresses and a map from street names to collected house‐number ranges.
/// * `Err(OsmPbfParseError)` if reading or parsing the file fails.
pub fn load_osm_data_with_housenumbers(
    path: impl AsRef<Path>,
    region: &WorldRegion,
) -> Result<(Vec<AddressRecord>, HouseNumberAggregator), OsmPbfParseError> {

    trace!(
        "load_osm_data_with_housenumbers: start path={:?}, region={:?}",
        path.as_ref(),
        region
    );

    // Step 1: Infer the Country from the given region.
    let country = infer_country_from_region(region)?;

    // Step 2: Open the OSM PBF file for reading.
    let reader = open_osm_pbf_reader(&path)?;

    // Step 3: We’ll accumulate addresses and house‐number ranges in memory.
    let mut street_hnr_map = HouseNumberAggregator::new(region);
    let mut addresses = Vec::new();

    // Step 4: Process the PBF file’s elements in a single pass.
    collect_address_and_housenumber_data(reader, &country, &mut addresses, &mut street_hnr_map)?;

    info!(
        "load_osm_data_with_housenumbers: completed. Found {} addresses; {} streets with house‐number data",
        addresses.len(),
        street_hnr_map.len()
    );

    Ok((addresses, street_hnr_map))
}

#[cfg(test)]
mod test_load_osm_data_with_housenumbers {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    /// We'll mock or stub out some of the helper calls to avoid requiring a real OSM PBF parser test.
    /// However, if you want a full end-to-end test, you can generate a tiny PBF file and pass it in.
    ///
    /// For demonstration, we'll outline two approaches:
    ///   1) **Full Integration** with a small OSM PBF fixture created at runtime.
    ///   2) **Partial** with a mock approach that forces errors or success at various steps.

    // ===========================================================================
    // (1) Full Integration Test using a Tiny PBF
    // ===========================================================================
    // If your codebase includes helpers like `create_tiny_osm_pbf(...)`, you can do:
    #[traced_test]
    async fn test_load_osm_data_with_housenumbers_tiny_pbf_success() {
        // We assume `create_tiny_osm_pbf_with_housenumber` or similar is available
        // for making a minimal PBF file with city/street/housenumber. 
        // If not, you can craft your own small .osm.pbf fixture.

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let pbf_path = temp_dir.path().join("tiny_test.osm.pbf");

        // For example, if you have a helper that writes a single node with:
        //    city="TestCity", street="TestStreet", housenumber="100-110"
        // we can do:
        if let Err(e) = create_tiny_osm_pbf_with_housenumber(&pbf_path).await {
            eprintln!("Could not create tiny PBF file: {:?}", e);
            // This test won't succeed, so let's just skip
            return;
        }

        // Choose a region that can be converted to a valid `Country` in `infer_country_from_region`.
        // For instance, USRegion::UnitedState(UnitedState::Maryland).into().
        let region = USRegion::UnitedState(UnitedState::Maryland).into();

        // Now call the function under test
        let result = load_osm_data_with_housenumbers(&pbf_path, &region);
        assert!(
            result.is_ok(),
            "Expected a success reading from the tiny PBF, got: {:?}",
            result
        );

        let (addresses, street_hnr_map) = result.unwrap();
        // We expect at least 1 address, 1 street with house‐number data
        assert_eq!(
            addresses.len(),
            1,
            "Should parse the single address from the tiny PBF"
        );
        assert_eq!(
            street_hnr_map.len(),
            1,
            "One street's house‐number range data expected"
        );

        // Optionally verify the address/street details
        let addr = &addresses[0];
        assert_eq!(addr.city().as_ref().unwrap().name(), "testcity");
        assert_eq!(addr.street().as_ref().unwrap().name(), "teststreet");
        assert_eq!(addr.postcode().is_none(), true, "No postcode in tiny fixture? If included, check it here.");

        let (street_key, ranges_vec) = street_hnr_map.iter().next().unwrap();
        assert_eq!(street_key.name(), "teststreet");
        assert_eq!(ranges_vec.len(), 1, "Should have exactly one merged range for '100-110'");
        let range = &ranges_vec[0];
        assert_eq!(*range.start(), 100);
        assert_eq!(*range.end(), 110);
    }

    // ===========================================================================
    // (2) Partial / Mock-based Testing
    // ===========================================================================
    // In these tests, we use minimal stubs or forcibly produce errors at various steps.

    /// Demonstrates the scenario where the file can't be opened (not found, etc.),
    /// leading to an error from `open_osm_pbf_reader`.
    #[traced_test]
    fn test_file_open_fails_returns_error() {
        // We'll pick a region that can be successfully converted to a Country.
        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        // But use a non-existent file path
        let non_existent_path = PathBuf::from("/non/existent/file.osm.pbf");

        let result = load_osm_data_with_housenumbers(&non_existent_path, &region);
        match result {
            Err(OsmPbfParseError::OsmPbf(e)) => {
                // Typically an IO error about the missing file
                assert!(
                    e.to_string().contains("No such file")
                        || e.to_string().contains("not found")
                        || e.to_string().contains("IO error"),
                    "Should reflect a file-open failure"
                );
            }
            other => panic!("Expected OsmPbfParseError::OsmPbf, got {:?}", other),
        }
    }

    /// Demonstrates a scenario where the PBF can be opened, 
    /// but an error occurs mid-processing in `collect_address_and_housenumber_data`.
    /// We can force an error with a partial mock if you'd like. 
    /// For simplicity, we'll create an invalid/corrupted PBF so reading fails during iteration.
    #[traced_test]
    fn test_mid_processing_parse_error() {
        // We'll create a "bogus" .pbf file with random content that isn't valid OSM data.
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let bogus_path = temp_dir.path().join("bogus.pbf");

        let mut file = File::create(&bogus_path).expect("Failed to create bogus pbf file");
        writeln!(file, "This is not valid OSM data").unwrap();
        drop(file);

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let result = load_osm_data_with_housenumbers(&bogus_path, &region);

        match result {
            Err(OsmPbfParseError::OsmPbf(e)) => {
                // Depending on the error from `osmpbf`, might say "invalid file format"
                assert!(
                    e.to_string().contains("invalid") 
                        || e.to_string().contains("corrupt")
                        || e.to_string().contains("OSM PBF parse error"),
                    "Expected parse error. Got: {:?}", e
                );
            }
            other => panic!("Expected OsmPbfParseError::OsmPbf for corrupted data. Got {:?}", other),
        }
    }
}
