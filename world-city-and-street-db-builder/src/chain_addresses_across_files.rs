// ---------------- [ File: src/chain_addresses_across_files.rs ]
// ---------------- [ File: src/chain_addresses_across_files.rs ]
crate::ix!();

/// Builds a single chained iterator of [`WorldAddress`] across all `.pbf` files.
/// For each file, we attempt to determine the region and parse the file, skipping
/// those with unknown regions.
///
/// # Returns
///
/// * `Ok(Box<dyn Iterator<Item = Result<WorldAddress, OsmPbfParseError>>>)` on success.
/// * `Err(OsmPbfParseError)` if an error arises parsing a given file.
pub fn chain_addresses_across_files<I:StorageInterface + 'static>(
    pbf_files:     Vec<PathBuf>,
    known_regions: &[WorldRegion],
    db:            Arc<Mutex<I>>,
    pbf_dir:       &Path,
) -> Result<Box<dyn Iterator<Item = Result<WorldAddress, OsmPbfParseError>>>, OsmPbfParseError> {
    trace!("chain_addresses_across_files: building iterator for {} files", pbf_files.len());

    // Start with an empty iterator
    let mut chained_iter = Box::new(iter::empty()) as Box<dyn Iterator<Item = _>>;

    for file_path in pbf_files {
        match find_region_for_file(&file_path, known_regions, pbf_dir) {
            Some(region) => {
                debug!(
                    "chain_addresses_across_files: associating file {:?} with region={:?}",
                    file_path, region
                );
                let file_iter = addresses_from_pbf_file_with_house_numbers(
                    file_path,
                    region,
                    db.clone()
                )?;
                // Chain it to the existing iterator
                chained_iter = Box::new(chained_iter.chain(file_iter));
            }
            None => {
                warn!("chain_addresses_across_files: could not determine region for file {:?}; skipping", file_path);
            }
        }
    }

    Ok(chained_iter)
}

#[cfg(test)]
mod chain_addresses_across_files_tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// A small helper to create an empty `.pbf` file at the given path.
    fn create_empty_pbf(path: &std::path::Path) -> std::io::Result<()> {
        File::create(path).map(|_f| ())
    }

    /// A helper that writes some minimal or invalid data to create a "corrupted" .pbf
    fn create_corrupted_pbf(path: &std::path::Path) -> std::io::Result<()> {
        let mut f = File::create(path)?;
        f.write_all(b"not real pbf data")?;
        Ok(())
    }

    /// For region detection, ensure your `find_region_for_file(...)` actually matches
    /// the filename pattern for known regions. E.g. if a file is "maryland-latest.osm.pbf"
    /// and `known_regions` has MD, it returns Some(Maryland). Otherwise returns None.

    // 1) No `.pbf` => empty => returns an empty chained iterator
    #[traced_test]
    fn test_chain_addresses_across_files_empty_list() {
        let db = Database::open(std::env::temp_dir().join("dummy_chain_db1")).unwrap();
        let known_regions = vec![];
        let pbf_dir = std::env::temp_dir();

        let result = chain_addresses_across_files(vec![], &known_regions, db, &pbf_dir);
        assert!(result.is_ok(), "No files => Ok");
        let chained_iter = result.unwrap();
        assert_eq!(chained_iter.count(), 0, "No files => empty iterator");
    }

    // 2) Single file recognized => yields addresses (assuming the file is valid)
    #[traced_test]
    fn test_chain_addresses_across_files_single_recognized() {
        // We'll define "maryland-latest.osm.pbf" as recognized => MD region
        // We'll create a minimal or empty .pbf that yields zero addresses or triggers parse logic.
        // In a real test, you'd create a small valid fixture that yields exactly 1 address.

        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("chain_db2")).unwrap();
        let known_region = USRegion::UnitedState(UnitedState::Maryland).into();
        let known_regions = vec![known_region];
        let pbf_dir = tmp.path();

        let pbf_path = pbf_dir.join("maryland-latest.osm.pbf");
        create_empty_pbf(&pbf_path).unwrap();

        // Now call chain_addresses_across_files with that single file
        let result = chain_addresses_across_files(vec![pbf_path.clone()], &known_regions, db.clone(), pbf_dir);
        assert!(result.is_ok(), "Should succeed building chain with recognized region");
        let chained_iter = result.unwrap();

        // The addresses_from_pbf_file_with_house_numbers might yield zero addresses if file is empty,
        // or 1 address if your code handles empty gracefully. We'll do an example check:
        let items: Vec<_> = chained_iter.collect();
        // If empty file => typically 0 addresses or 1 parse error. We'll check for 0 addresses as the "happy path".
        assert_eq!(
            items.len(),
            0,
            "Empty .pbf => we expect no addresses or possibly an error item; adjust if needed"
        );
    }

    // 3) Single file unrecognized => skip => yields no addresses
    #[traced_test]
    fn test_chain_addresses_across_files_single_unrecognized() {
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("chain_db3")).unwrap();
        // known_regions => "virginia", but the file is "maryland-latest..."
        let known_region = USRegion::UnitedState(UnitedState::Virginia).into();
        let known_regions = vec![known_region];
        let pbf_dir = tmp.path();

        let pbf_path = pbf_dir.join("maryland-latest.osm.pbf");
        create_empty_pbf(&pbf_path).unwrap();

        let result = chain_addresses_across_files(vec![pbf_path], &known_regions, db, pbf_dir);
        assert!(result.is_ok());
        let chained_iter = result.unwrap();
        let items: Vec<_> = chained_iter.collect();
        assert_eq!(
            items.len(),
            0,
            "File => region not recognized => skip => yields zero addresses"
        );
    }

    // 4) Multiple files => some recognized, some not => chain them
    #[traced_test]
    fn test_chain_addresses_across_files_multiple() {
        // We'll define 2 recognized regions => MD, VA
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("chain_db4")).unwrap();
        let known_md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let known_va: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let known_regions = vec![known_md, known_va];
        let pbf_dir = tmp.path();

        // recognized => "maryland-latest.osm.pbf"
        let md_path = pbf_dir.join("maryland-latest.osm.pbf");
        create_empty_pbf(&md_path).unwrap();

        // recognized => "virginia-latest.osm.pbf"
        let va_path = pbf_dir.join("virginia-latest.osm.pbf");
        create_empty_pbf(&va_path).unwrap();

        // unrecognized => "unknown-latest.osm.pbf"
        let unknown_path = pbf_dir.join("unknown-latest.osm.pbf");
        create_empty_pbf(&unknown_path).unwrap();

        let all_files = vec![md_path, unknown_path, va_path];
        let result = chain_addresses_across_files(all_files, &known_regions, db.clone(), pbf_dir);
        assert!(result.is_ok());
        let chained_iter = result.unwrap();
        let items: Vec<_> = chained_iter.collect();
        // Because each recognized file is empty => yields 0 addresses. So total => 0
        assert_eq!(
            items.len(),
            0,
            "2 recognized empty => 0 addresses, unknown => skip => total 0"
        );

        // If you had partial data that yields addresses, you'd see them. 
        // The key check is we didn't blow up. We skip the unknown file and 
        // yield the chain from the 2 recognized ones.
    }

    // 5) Error from addresses_from_pbf_file_with_house_numbers => returns Err
    // e.g. if one recognized file is corrupted => that function returns an error => short-circuit
    #[traced_test]
    fn test_chain_addresses_across_files_error_in_mid_loop() {
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("chain_db5")).unwrap();
        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let known_regions = vec![region];
        let pbf_dir = tmp.path();

        // File #1 => recognized => but it's "maryland-latest.osm.pbf" and let's corrupt it => parse error
        let md_path = pbf_dir.join("maryland-latest.osm.pbf");
        create_corrupted_pbf(&md_path).unwrap();

        // File #2 => recognized => "maryland-latest2.osm.pbf" => skip or recognized? 
        // If your code specifically checks "maryland-latest.osm.pbf" ignoring ASCII case, 
        // then "maryland-latest2" might not match. We'll do a second recognized name:
        let md2_path = pbf_dir.join("maryland-latest.osm.pbf.md5");
        // or "maryland-latest.abcdef.osm.pbf" if your code allows appended .abcdef before .osm.pbf
        create_empty_pbf(&md2_path).unwrap();

        // The first recognized file => parse error => chain_addresses_across_files returns an Err immediately (since the function calls addresses_from_pbf_file_with_house_numbers and returns its error).
        let result = chain_addresses_across_files(
            vec![md_path.clone(), md2_path.clone()],
            &known_regions,
            db.clone(),
            pbf_dir
        );
        // We expect an immediate error from the first file's parse:
        assert!(result.is_err(), "Corrupted PBF => parse error => Err(...) from chain fn");

        match result.err().unwrap() {
            OsmPbfParseError::OsmPbf(_) => {
                // good => parse error
            }
            other => panic!("Expected parse error from corrupted pbf, got: {:?}", other),
        }
    }
}
