// ---------------- [ File: src/list_all_addresses_in_pbf_dir.rs ]
crate::ix!();

pub fn list_all_addresses_in_pbf_dir(
    pbf_dir: impl AsRef<Path>
) -> Result<impl Iterator<Item=Result<WorldAddress, OsmPbfParseError>>, OsmPbfParseError> {

    let entries = std::fs::read_dir(&pbf_dir)?;

    // Gather PBF files
    let mut pbf_files = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("pbf") {
            pbf_files.push(path);
        }
    }

    // Known regions (DMV or more)
    let known_regions = world_regions();

    info!("listing all addresses in the PBF dir for all known regions: {:#?}", known_regions);

    // Create a chained iterator of addresses from all files
    // We'll create one iterator per file and chain them.
    let mut chained = Box::new(std::iter::empty()) as Box<dyn Iterator<Item=Result<WorldAddress, OsmPbfParseError>>>;

    for file_path in pbf_files {

        // pbf_dir is your base directory of PBF files
        let region = match find_region_for_file(&file_path, &known_regions, &pbf_dir) {
            Some(r) => r,
            None => {
                warn!("Could not determine region for file {:?}, skipping", file_path);
                continue;
            }
        };

        // Create an iterator for this file's addresses
        let file_iter = addresses_from_pbf_file(file_path, region)?;
        // Chain it to the existing iterator
        chained = Box::new(chained.chain(file_iter));
    }

    Ok(chained)
}

/// Returns an iterator over addresses from a single PBF file.
fn addresses_from_pbf_file(
    path:         PathBuf,
    world_region: WorldRegion,

) -> Result<impl Iterator<Item=Result<WorldAddress, OsmPbfParseError>>, OsmPbfParseError> {

    info!("parsing addresses for pbf_file {:?} for region {}", path, world_region);

    let country = Country::try_from(world_region)?;

    let (tx, rx) = std::sync::mpsc::sync_channel(1000);

    std::thread::spawn(move || {
        let reader = match osmpbf::ElementReader::from_path(&path) {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(Err(OsmPbfParseError::OsmPbf(e)));
                return;
            }
        };

        let result = reader.for_each(|element| {

            // The closure returns (), not Result.
            match AddressRecord::try_from((element,&country)) {

                Ok(rec) => {

                    if let (Some(city), Some(street), Some(postcode)) = (rec.city(), rec.street(), rec.postcode()) {

                        let addr = match WorldAddressBuilder::default()
                            .region(world_region)
                            .city(city.clone())
                            .street(street.clone())
                            .postal_code(postcode.clone())
                            .build() 
                        {
                            Ok(a) => a,
                            Err(_) => {
                                warn!("Failed to build WorldAddress from record");
                                // Just skip this record
                                return;
                            }
                        };

                        if tx.send(Ok(addr)).is_err() {
                            // Receiver dropped, we can't return an error directly here.
                            // Log and optionally panic or break.
                            warn!("Receiver dropped. Stopping processing.");
                            // If you need to stop processing, just return. `for_each` will continue for all elements,
                            // but since `tx` is closed, no more addresses will be handled upstream.
                            return;
                        }

                        // Successfully processed this element
                    } else {
                        // Incomplete address, skip
                    }
                }
                Err(e) => {
                    // Could not parse this element as address
                    // warn!("Failed to parse address element: {:?}", e);
                    // Just skip this element
                }
            }
            // End of closure for this element - returns ()
        });

        // If reading failed at some point, result would be Err
        if let Err(osmpbf_err) = result {
            let _ = tx.send(Err(OsmPbfParseError::OsmPbf(osmpbf_err)));
        }
    });

    Ok(rx.into_iter())
}

#[cfg(test)]
mod list_all_addresses_tests {
    use super::*;

    /// A local utility: create a `.pbf` file with random/corrupt data for parse error tests.
    async fn create_corrupt_pbf(path: &Path) {
        let mut f = File::create(path).await.expect("create corrupt pbf");
        f.write_all(b"not a valid pbf").await.expect("write bytes");
    }

    /// Another local utility: create an empty `.pbf` file (zero bytes).
    async fn create_empty_pbf(path: &Path) {
        File::create(path).await.expect("create empty pbf");
    }

    /// Because `list_all_addresses_in_pbf_dir` uses the real `world_regions` and real `find_region_for_file`,
    /// you may want to override them for tests. If you want to force a minimal set of known regions,
    /// you’d have to do a local re-implementation or some injection approach.
    /// Here, we demonstrate a real integration test (assuming `world_regions()` returns MD and VA, etc.).
    /// If your real code returns many regions, that’s fine.

    #[traced_test]
    async fn test_list_all_addresses_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        // No `.pbf` files => the returned iterator is empty
        let result = list_all_addresses_in_pbf_dir(temp_dir.path());
        assert!(result.is_ok(), "Should not fail scanning an empty dir");
        let iter = result.unwrap();
        assert_eq!(iter.count(), 0, "No pbf => no addresses");
    }

    #[traced_test]
    async fn test_list_all_addresses_no_matching_filenames() {
        let temp_dir = TempDir::new().unwrap();

        // create a file not recognized by find_region_for_file => e.g. "unknown-latest.osm.pbf"
        let unknown_path = temp_dir.path().join("unknown-latest.osm.pbf");
        File::create(&unknown_path).await.unwrap().write_all(b"fake");

        // We'll parse => The file is recognized as `.pbf`, but region is None => skip => no addresses.
        let result = list_all_addresses_in_pbf_dir(temp_dir.path()).unwrap();
        let items: Vec<_> = result.collect();
        assert_eq!(items.len(), 0, "No recognized region => skip => zero addresses");
    }

    #[test]
    fn test_list_all_addresses_corrupted_pbf() {
        let temp_dir = TempDir::new().unwrap();

        // Suppose "maryland-latest.osm.pbf" => recognized => but is corrupted => parse error
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");
        create_corrupt_pbf(&pbf_path);

        let result_iter = list_all_addresses_in_pbf_dir(temp_dir.path()).unwrap();

        // We expect iteration to yield exactly one Err(...) from the parse failure.
        let mut iter = result_iter;
        let first = iter.next();
        assert!(first.is_some(), "We expect at least 1 item, an error, from the parse");
        let first_err = first.unwrap();
        match first_err {
            Err(OsmPbfParseError::OsmPbf(_)) => {
                debug!("Got parse error from corrupted file as expected");
            }
            other => panic!("Expected OsmPbf parse error, got: {:?}", other),
        }
        // No more items afterwards
        let second = iter.next();
        assert!(second.is_none());
    }

    #[test]
    fn test_list_all_addresses_zero_length_pbf() {
        let temp_dir = TempDir::new().unwrap();

        // "virginia-latest.osm.pbf" => recognized => zero-length => parse error or empty
        let pbf_path = temp_dir.path().join("virginia-latest.osm.pbf");
        create_empty_pbf(&pbf_path);

        // Depending on osmpbf’s behavior, zero-length might yield an error or possibly an empty set.
        // Let’s see:
        let result_iter = list_all_addresses_in_pbf_dir(temp_dir.path()).unwrap();
        let collected: Vec<_> = result_iter.collect();
        if collected.is_empty() {
            debug!("Zero-length file => no addresses, no error, that’s acceptable");
        } else {
            // Possibly one error
            assert_eq!(collected.len(), 1, "Probably one parse error from zero-length file");
            let first = &collected[0];
            assert!(matches!(first, Err(OsmPbfParseError::OsmPbf(_))), "Likely parse error");
        }
    }

    #[test]
    fn test_list_all_addresses_multiple_files_mixture() {
        // Scenario: 2 .pbf files => one recognized region (MD) but corrupted,
        // the other recognized region (VA) but also corrupted => we see 2 errors in iteration
        let temp_dir = TempDir::new().unwrap();

        let md_path = temp_dir.path().join("maryland-latest.osm.pbf");
        create_corrupt_pbf(&md_path);

        let va_path = temp_dir.path().join("virginia-latest.osm.pbf");
        create_corrupt_pbf(&va_path);

        let iter = list_all_addresses_in_pbf_dir(temp_dir.path()).unwrap();
        let results: Vec<_> = iter.collect();
        assert_eq!(results.len(), 2, "two .pbf => likely two parse attempts => two errors");
        for res in results {
            match res {
                Err(OsmPbfParseError::OsmPbf(_)) => {}, 
                other => panic!("Expected parse error, got: {:?}", other),
            }
        }
    }

    // If you truly want to test a “valid .osm.pbf” with real addresses, 
    // you’d either place a tiny fixture in your repo or do a more advanced mock 
    // of `osmpbf::ElementReader`. For now, these tests confirm skipping unknown 
    // files, handling parse errors, and a recognized but empty file scenario.
}
