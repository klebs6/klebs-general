// ---------------- [ File: src/list_all_addresses_in_pbf_dir.rs ]
crate::ix!();

pub fn list_all_addresses_in_pbf_dir(
    pbf_dir: impl AsRef<Path>
) -> Result<impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>, OsmPbfParseError> {
    let entries = std::fs::read_dir(&pbf_dir)?;

    // Gather PBF files.
    let mut pbf_files = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("pbf") {
            pbf_files.push(path);
        }
    }

    // Known regions (DMV or more).
    let known_regions = dmv_regions();
    info!(
        "listing all addresses in the PBF dir for all known regions: {:#?}",
        known_regions
    );

    // Create a chained iterator of addresses from all files.
    let mut chained = Box::new(iter::empty()) as Box<dyn Iterator<Item = Result<WorldAddress, OsmPbfParseError>>>;
    for file_path in pbf_files {
        // pbf_dir is your base directory of PBF files.
        let region = match find_region_for_file(&file_path, &known_regions, &pbf_dir) {
            Some(r) => r,
            None => {
                warn!("Could not determine region for file {:?}, skipping", file_path);
                continue;
            }
        };

        // Create an iterator for this file's addresses.
        let file_iter = addresses_from_pbf_file(file_path, region)?;
        // Chain it to the existing iterator.
        chained = Box::new(chained.chain(file_iter));
    }

    Ok(chained)
}

/// Returns an iterator over addresses from a single PBF file.
fn addresses_from_pbf_file(
    path: PathBuf,
    world_region: impl Into<crate::WorldRegion>,
) -> Result<impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>, OsmPbfParseError> {
    let world_region = world_region.into();
    info!("parsing addresses for pbf_file {:?} for region {}", path, world_region);

    let country = Country::try_from(world_region)?;
    let (tx, rx) = mpsc::sync_channel(1000);

    thread::spawn(move || {
        let reader = match osmpbf::ElementReader::from_path(&path) {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(Err(OsmPbfParseError::OsmPbf(e)));
                return;
            }
        };

        let result = reader.for_each(|element| {
            // The closure returns (), not Result.
            match AddressRecord::try_from((&element, &country)) {
                Ok(rec) => {
                    if let (Some(city), Some(street), Some(postcode)) =
                        (rec.city(), rec.street(), rec.postcode())
                    {
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
                                // Just skip this record.
                                return;
                            }
                        };

                        if tx.send(Ok(addr)).is_err() {
                            warn!("Receiver dropped. Stopping processing.");
                            return;
                        }
                    }
                }
                Err(_) => {
                    // If parsing of the element fails, we simply skip it.
                }
            }
        });

        if let Err(osmpbf_err) = result {
            let _ = tx.send(Err(OsmPbfParseError::OsmPbf(osmpbf_err)));
        }
    });

    Ok(rx.into_iter())
}

#[cfg(test)]
mod list_all_addresses_tests {
    use super::*;

    // Asynchronous utility to create a corrupted .pbf file.
    async fn create_corrupt_pbf(path: &Path) {
        let mut f = tokio::fs::File::create(path).await.expect("create corrupt pbf");
        f.write_all(b"not a valid pbf").await.expect("write bytes");
    }

    // Asynchronous utility to create an empty .pbf file.
    async fn create_empty_pbf(path: &Path) {
        tokio::fs::File::create(path).await.expect("create empty pbf");
    }

    #[traced_test]
    async fn test_list_all_addresses_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        // No .pbf files => the returned iterator is empty.
        let result = list_all_addresses_in_pbf_dir(temp_dir.path());
        assert!(result.is_ok(), "Should not fail scanning an empty dir");
        let iter = result.unwrap();
        assert_eq!(iter.count(), 0, "No pbf => no addresses");
    }

    #[traced_test]
    async fn test_list_all_addresses_no_matching_filenames() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file not recognized by find_region_for_file, e.g. "unknown-latest.osm.pbf".
        let unknown_path = temp_dir.path().join("unknown-latest.osm.pbf");
        {
            let mut f = tokio::fs::File::create(&unknown_path).await.unwrap();
            f.write_all(b"fake").await.unwrap();
        }

        // The file is recognized as .pbf, but region determination fails => skip.
        let result = list_all_addresses_in_pbf_dir(temp_dir.path()).unwrap();
        let items: Vec<_> = result.collect();
        assert_eq!(items.len(), 0, "No recognized region => skip => zero addresses");
    }

    #[test]
    fn test_list_all_addresses_corrupted_pbf() {
        let temp_dir = TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");
        {
            let rt = Runtime::new().unwrap();
            rt.block_on(create_corrupt_pbf(&pbf_path));
        }

        let result_iter = list_all_addresses_in_pbf_dir(temp_dir.path()).unwrap();

        // We expect the iterator to yield at least one error.
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
        // Expect no more items.
        let second = iter.next();
        assert!(second.is_none());
    }

    #[test]
    fn test_list_all_addresses_zero_length_pbf() {
        let temp_dir = TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("virginia-latest.osm.pbf");
        {
            let rt = Runtime::new().unwrap();
            rt.block_on(create_empty_pbf(&pbf_path));
        }
        let result_iter = list_all_addresses_in_pbf_dir(temp_dir.path()).unwrap();
        let collected: Vec<_> = result_iter.collect();
        if collected.is_empty() {
            debug!("Zero-length file => no addresses, acceptable");
        } else {
            assert_eq!(collected.len(), 1, "Probably one parse error from zero-length file");
            let first = &collected[0];
            assert!(matches!(first, Err(OsmPbfParseError::OsmPbf(_))), "Likely parse error");
        }
    }

    #[test]
    fn test_list_all_addresses_multiple_files_mixture() {
        let temp_dir = TempDir::new().unwrap();
        let md_path = temp_dir.path().join("maryland-latest.osm.pbf");
        let va_path = temp_dir.path().join("virginia-latest.osm.pbf");
        {
            let rt = Runtime::new().unwrap();
            rt.block_on(create_corrupt_pbf(&md_path));
            rt.block_on(create_corrupt_pbf(&va_path));
        }
        let iter = list_all_addresses_in_pbf_dir(temp_dir.path()).unwrap();
        let results: Vec<_> = iter.collect();
        assert_eq!(results.len(), 2, "two .pbf files should yield two parse errors");
        for res in results {
            match res {
                Err(OsmPbfParseError::OsmPbf(_)) => {}
                other => panic!("Expected parse error, got: {:?}", other),
            }
        }
    }
}
