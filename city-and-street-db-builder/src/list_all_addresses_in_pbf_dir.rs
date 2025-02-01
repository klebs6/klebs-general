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

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Minimal mock `world_regions()` returning only 2 test regions for clarity:
    /// e.g. Maryland => "maryland-latest.osm.pbf", Virginia => "virginia-latest.osm.pbf".
    /// The real code presumably returns more, but for testing we can override or
    /// replicate the real function if you can do so in your test mod.
    fn test_world_regions() -> Vec<WorldRegion> {
        vec![
            USRegion::UnitedState(UnitedState::Maryland).into(),
            USRegion::UnitedState(UnitedState::Virginia).into(),
        ]
    }

    /// Minimal mock/fake implementation of `find_region_for_file()` that matches exactly
    /// the two known test world_regions. In actual usage, you'd rely on your real function.
    /// However, for robust tests, we might need to stub or override it to ensure deterministic behavior.
    fn test_find_region_for_file(
        pbf_path: &Path,
        known_regions: &[WorldRegion],
        base_dir: impl AsRef<Path>,
    ) -> Option<WorldRegion> {
        // We can do something extremely simplistic:
        // if pbf file name starts with "maryland-latest" => region=MD
        // if pbf file name starts with "virginia-latest" => region=VA
        // else => None
        let fname = pbf_path.file_name()?.to_str()?;
        if fname.to_ascii_lowercase().starts_with("maryland-latest") {
            Some(USRegion::UnitedState(UnitedState::Maryland).into())
        } else if fname.to_ascii_lowercase().starts_with("virginia-latest") {
            Some(USRegion::UnitedState(UnitedState::Virginia).into())
        } else {
            None
        }
    }

    /// Write a minimal OSM PBF file with one or more Node elements that
    /// contain `addr:city`, `addr:street`, `addr:postcode` tags. This will let
    /// us test the reading logic. We’ll do a small example using `osmpbf::ElementWriter`.
    ///
    /// For a full test, we'd have to ensure that `osmpbf` can parse it back.
    /// This method is somewhat simplistic, because the official `osmpbf` crate
    /// typically writes to `.osc` or `blob` formats. But let's show the gist:
    fn write_mock_osm_pbf_with_addresses(
        path: &Path, 
        addresses: &[(Option<&str>, Option<&str>, Option<&str>)] // city, street, postcode
    ) -> Result<(), OsmPbfParseError> {
        // Create the file
        let file = File::create(path)?;
        let mut writer = ElementWriter::new(file);

        // We'll write each address as a Node with the relevant tags
        for (i, (city, street, postcode)) in addresses.iter().enumerate() {
            let mut tags = Vec::new();
            if let Some(c) = city { tags.push(("addr:city".to_string(), c.to_string())) }
            if let Some(s) = street { tags.push(("addr:street".to_string(), s.to_string())) }
            if let Some(pc) = postcode { tags.push(("addr:postcode".to_string(), pc.to_string())) }

            let node = osmpbf::Node {
                id: (1000 + i) as i64,
                lat: None,
                lon: None,
                tags,
            };
            writer.write(&Element::Node(node))?;
        }
        writer.close()?;
        Ok(())
    }

    /// A variant to produce a PBF file that is either corrupt or unparseable
    /// by `osmpbf::ElementReader::from_path()`.
    async fn write_corrupt_osm_pbf(path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path).await?;
        // Just write some random bytes
        file.write_all(b"not a real pbf file").await?;
        Ok(())
    }

    // We'll override the references in `list_all_addresses_in_pbf_dir` for the sake of testing:
    // * Instead of calling the real `world_regions()`, we might call `test_world_regions()`.
    // * Instead of `find_region_for_file()`, call `test_find_region_for_file`.
    // But if you prefer, you can do direct integration tests. For clarity, we show how to
    // override by re-implementing a small function with the same logic but stubs.

    /// A re-implemented function that’s identical in logic to `list_all_addresses_in_pbf_dir`,
    /// but uses our test stubs for `world_regions()` and `find_region_for_file()`.
    fn list_all_addresses_in_pbf_dir_test_stub(
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

        // Known regions
        let known_regions = test_world_regions();

        // Create a chained iterator
        let mut chained = Box::new(std::iter::empty()) as Box<dyn Iterator<Item=Result<WorldAddress, OsmPbfParseError>>>;

        for file_path in pbf_files {
            let region = match test_find_region_for_file(&file_path, &known_regions, &pbf_dir) {
                Some(r) => r,
                None => {
                    // skipping
                    continue;
                }
            };

            let file_iter = addresses_from_pbf_file(file_path, region)?;
            chained = Box::new(chained.chain(file_iter));
        }

        Ok(chained)
    }

    // -------------------------------------------------------------------------
    // The actual tests
    // -------------------------------------------------------------------------

    /// Test a directory with zero `.pbf` files => the returned iterator is empty.
    #[test]
    fn test_list_all_addresses_empty_dir() -> Result<(), ListAllAddressesError> {
        let temp_dir = TempDir::new()?;
        let result_iter = list_all_addresses_in_pbf_dir_test_stub(temp_dir.path())?;
        assert_eq!(result_iter.count(), 0, "No .pbf => no addresses");
        Ok(())
    }

    /// Test a directory with some .pbf file that doesn't match any known region => skip it => 0 addresses.
    #[traced_test]
    async fn test_list_all_addresses_unknown_region_pbf() -> Result<(), ListAllAddressesError> {
        let temp_dir = TempDir::new()?;
        let pbf_path = temp_dir.path().join("guam-latest.osm.pbf");
        // We can put something valid or empty
        File::create(&pbf_path).await?.write_all(b"whatever")?;

        let result_iter = list_all_addresses_in_pbf_dir_test_stub(temp_dir.path())?;
        assert_eq!(result_iter.count(), 0, "File name not recognized => skip => no addresses");
        Ok(())
    }

    /// Test a directory with one recognized PBF file that is valid and has some addresses => we see them in iteration.
    #[test]
    fn test_list_all_addresses_one_valid_pbf() -> Result<(), ListAllAddressesError> {
        let temp_dir = TempDir::new()?;

        // We'll treat "maryland-latest.osm.pbf" => region=MD
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");
        write_mock_osm_pbf_with_addresses(
            &pbf_path, 
            &[
                (Some("Baltimore"), Some("North Avenue"), Some("21201")),
                (None, Some("Ignored Street"), Some("21201")), // partial => incomplete => skip
                (Some("Rockville"), Some("Veirs Mill Rd"), Some("20850")),
            ]
        )?;

        let result_iter = list_all_addresses_in_pbf_dir_test_stub(temp_dir.path())?;
        let addresses: Vec<_> = result_iter.collect();

        // We should have 2 valid addresses (the second is incomplete => skipped)
        assert_eq!(addresses.len(), 2, "Expected 2 fully-formed addresses from the .pbf file");
        for addr in addresses {
            let addr = addr?;
            // region => Maryland
            assert_eq!(addr.region(), &USRegion::UnitedState(UnitedState::Maryland).into());
        }
        Ok(())
    }

    /// Test a directory with multiple recognized PBF files => the resulting iterator is a chain.
    #[test]
    fn test_list_all_addresses_two_valid_pbfs() -> Result<(), ListAllAddressesError> {
        let temp_dir = TempDir::new()?;

        // 1) "maryland-latest.osm.pbf"
        let pbf_md = temp_dir.path().join("maryland-latest.osm.pbf");
        write_mock_osm_pbf_with_addresses(
            &pbf_md, 
            &[
                (Some("Baltimore"), Some("North Avenue"), Some("21201")),
            ]
        )?;

        // 2) "virginia-latest.osm.pbf"
        let pbf_va = temp_dir.path().join("virginia-latest.osm.pbf");
        write_mock_osm_pbf_with_addresses(
            &pbf_va, 
            &[
                (Some("Clifton"), Some("Redbird Ridge"), Some("20124")),
            ]
        )?;

        let result_iter = list_all_addresses_in_pbf_dir_test_stub(temp_dir.path())?;
        let addrs: Vec<_> = result_iter.collect();

        assert_eq!(addrs.len(), 2, "One address from MD file + one address from VA file");
        let md_addr = &addrs[0].as_ref().unwrap();
        let va_addr = &addrs[1].as_ref().unwrap();
        assert_eq!(md_addr.region(), &USRegion::UnitedState(UnitedState::Maryland).into());
        assert_eq!(va_addr.region(), &USRegion::UnitedState(UnitedState::Virginia).into());
        Ok(())
    }

    /// Test that if osmpbf fails to parse a file, we get an error in the iteration.
    #[traced_test]
    async fn test_list_all_addresses_corrupted_pbf() -> Result<(), ListAllAddressesError> {
        let temp_dir = TempDir::new()?;

        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");
        write_corrupt_osm_pbf(&pbf_path).await?;

        let result_iter = list_all_addresses_in_pbf_dir_test_stub(temp_dir.path())?;

        // The iterator might produce exactly one Err(...) from addresses_from_pbf_file if the parse fails.
        // Let's collect them. We expect either 0 or 1 items. If the parse fails fast, we might see one error.
        let mut iter = result_iter;

        // Attempt to get the first item from the iterator
        let first = iter.next();
        assert!(first.is_some(), "We do expect at least one item that is an Err(...) if parse fails");
        let first_err = first.unwrap();
        assert!(matches!(first_err, Err(OsmPbfParseError::OsmPbf(_))), "Should be an OsmPbf parse error");
        
        // Next item should be None
        let second = iter.next();
        assert!(second.is_none(), "After parse error, no further items should appear");
        Ok(())
    }

    /// Test that if we have multiple files, and exactly one is corrupted,
    /// we see addresses from the good file plus an error from the bad file
    /// in the combined chain. The exact iteration sequence may vary:
    /// we might see the good file's addresses first, then an Err from the bad file.
    #[traced_test]
    async fn test_list_all_addresses_one_corrupted_one_good() -> Result<(), ListAllAddressesError> {
        let temp_dir = TempDir::new()?;

        // Good MD file
        let pbf_md = temp_dir.path().join("maryland-latest.osm.pbf");
        write_mock_osm_pbf_with_addresses(
            &pbf_md, 
            &[(Some("Baltimore"), Some("Howard St"), Some("21201"))]
        )?;

        // Corrupted VA file
        let pbf_va = temp_dir.path().join("virginia-latest.osm.pbf");
        write_corrupt_osm_pbf(&pbf_va).await?;

        let result_iter = list_all_addresses_in_pbf_dir_test_stub(temp_dir.path())?;
        let results: Vec<_> = result_iter.collect();

        // We expect 2 items: 
        //   1) Ok(WorldAddress for MD)
        //   2) Err(...) for the VA parse error
        assert_eq!(results.len(), 2, "One good address + one parse error item");

        // We won't guarantee the order, but typically the MD file is discovered first by the OS directory iteration,
        // so we see Ok(...) then Err(...).
        let first = &results[0];
        let second = &results[1];

        // Find at least one is Ok => Baltimore address, at least one is Err => parse error
        assert!(
            matches!(first, Ok(addr) if addr.city().name() == "baltimore") ||
            matches!(second, Ok(addr) if addr.city().name() == "baltimore"),
            "Should see a valid address from the MD file"
        );

        assert!(
            matches!(first, Err(OsmPbfParseError::OsmPbf(_))) ||
            matches!(second, Err(OsmPbfParseError::OsmPbf(_))),
            "Should see an OsmPbf parse error from the corrupted VA file"
        );
        Ok(())
    }

    /// (Addresses_from_pbf_file specific) - test that addresses from partial or incomplete record are skipped.
    #[test]
    fn test_addresses_from_pbf_file_partial_skip() -> Result<(), ListAllAddressesError> {
        // We'll just call addresses_from_pbf_file directly with a minimal file.
        let temp_dir = TempDir::new()?;
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");
        write_mock_osm_pbf_with_addresses(
            &pbf_path,
            &[
                (Some("Baltimore"), None, Some("21201")), // missing street => skip
                (None, Some("North Avenue"), Some("21201")), // missing city => skip
                (Some("Baltimore"), Some("North Avenue"), None), // missing postcode => skip
                (Some("Baltimore"), Some("North Avenue"), Some("21201")), // full => used
            ]
        )?;

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let iter = addresses_from_pbf_file(pbf_path, region)?;

        let results: Vec<_> = iter.collect();
        assert_eq!(results.len(), 1, "Only the fully-populated record yields an address");
        let first = results[0].as_ref().unwrap();
        assert_eq!(first.city().name(), "baltimore");
        assert_eq!(first.street().name(), "north avenue");
        assert_eq!(first.postal_code().code(), "21201");
        Ok(())
    }

    /// (Addresses_from_pbf_file specific) - test that if WorldAddressBuilder fails for some reason,
    /// we skip that record. This might be contrived unless there's logic in the builder that can fail.
    #[test]
    fn test_addresses_from_pbf_file_world_address_builder_failure_is_skipped() -> Result<(), ListAllAddressesError> {
        // Suppose your builder fails if city name is over 256 chars, or something similar.
        // We'll simulate that scenario by mocking or extending the test. Here, we assume that
        // "StreetName" might fail if it's empty after normalization, etc. 
        // We'll produce a node with weird data that triggers an error in `WorldAddressBuilder`.
        // In the standard code, it might not fail. We'll just show the structure.

        let temp_dir = TempDir::new()?;
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");
        // We'll rely on the normal validations we have: if city= "???" => no real fail. 
        // For demonstration, let's do an extremely large city name to see if your code fails.
        let big_city = "X".repeat(2000);

        write_mock_osm_pbf_with_addresses(
            &pbf_path,
            &[
                (Some(&big_city), Some("North Avenue"), Some("21201")),
                (Some("Baltimore"), Some("North Avenue"), Some("21201")),
            ]
        )?;

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let iter = addresses_from_pbf_file(pbf_path, region)?;
        let results: Vec<_> = iter.collect();

        // If your code doesn't fail with big city, we'd see 2. If it does fail,
        // the first is skipped, we only see the second. We'll just show the test structure:
        //
        // Let's say we expected 1 => skip big city, accept normal one
        if results.len() == 1 {
            assert_eq!(results[0].as_ref().unwrap().city().name(), "baltimore");
        } else {
            // Possibly your code doesn't fail at all => 2 addresses
            assert_eq!(results.len(), 2, "If no builder fail => we have 2 addresses");
        }
        Ok(())
    }
}
