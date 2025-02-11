// ---------------- [ File: src/list_all_addresses_in_pbf_dir.rs ]
crate::ix!();

pub type WorldAddressIterator = impl Iterator<Item=Result<WorldAddress,OsmPbfParseError>>;

/// Produces an iterator of [`WorldAddress`] items by scanning a directory for
/// `.pbf` files and attempting to parse each one. Files are associated with
/// known regions and then processed in sequence.
///
/// # Arguments
///
/// * `pbf_dir` - A filesystem path to a directory containing `.pbf` files.
/// * `db`      - Shared database reference for storing or retrieving house number data.
///
/// # Returns
///
/// * `Ok(impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>)` on success.
/// * `Err(OsmPbfParseError)` if reading the directory or chaining file iterators fails.
pub fn list_all_addresses_in_pbf_dir<I:StorageInterface + 'static>(
    pbf_dir: impl AsRef<Path>,
    db: Arc<Mutex<I>>,
) -> Result<WorldAddressIterator, OsmPbfParseError> {
    trace!("list_all_addresses_in_pbf_dir: start for pbf_dir={:?}", pbf_dir.as_ref());

    // 1) Collect all `.pbf` files from the directory.
    let pbf_files = gather_pbf_files(pbf_dir.as_ref())?;
    debug!("list_all_addresses_in_pbf_dir: found {} PBF files", pbf_files.len());

    // 2) Identify known regions.
    let known_regions = dmv_regions();
    info!("list_all_addresses_in_pbf_dir: known regions: {:#?}", known_regions);

    // 3) Build a chained iterator of addresses from all recognized PBF files.
    let chained = chain_addresses_across_files(pbf_files, &known_regions, db, pbf_dir.as_ref())?;
    Ok(chained)
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
        let db = Database::open(&temp_dir).expect("DB should open");
        // No .pbf files => the returned iterator is empty.
        let result = list_all_addresses_in_pbf_dir(temp_dir.path(),db);
        assert!(result.is_ok(), "Should not fail scanning an empty dir");
        let iter = result.unwrap();
        assert_eq!(iter.count(), 0, "No pbf => no addresses");
    }

    #[traced_test]
    async fn test_list_all_addresses_no_matching_filenames() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(&temp_dir).expect("DB should open");

        // Create a file not recognized by find_region_for_file, e.g. "unknown-latest.osm.pbf".
        let unknown_path = temp_dir.path().join("unknown-latest.osm.pbf");
        {
            let mut f = tokio::fs::File::create(&unknown_path).await.unwrap();
            f.write_all(b"fake").await.unwrap();
        }

        // The file is recognized as .pbf, but region determination fails => skip.
        let result = list_all_addresses_in_pbf_dir(temp_dir.path(),db).unwrap();
        let items: Vec<_> = result.collect();
        assert_eq!(items.len(), 0, "No recognized region => skip => zero addresses");
    }

    #[traced_test]
    fn test_list_all_addresses_corrupted_pbf() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(&temp_dir).expect("DB should open");
        let pbf_path = temp_dir.path().join("maryland-latest.osm.pbf");
        {
            let rt = Runtime::new().unwrap();
            rt.block_on(create_corrupt_pbf(&pbf_path));
        }

        let result_iter = list_all_addresses_in_pbf_dir(temp_dir.path(),db).unwrap();

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

    #[traced_test]
    fn test_list_all_addresses_zero_length_pbf() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(&temp_dir).expect("DB should open");
        let pbf_path = temp_dir.path().join("virginia-latest.osm.pbf");
        {
            let rt = Runtime::new().unwrap();
            rt.block_on(create_empty_pbf(&pbf_path));
        }
        let result_iter = list_all_addresses_in_pbf_dir(temp_dir.path(),db).unwrap();
        let collected: Vec<_> = result_iter.collect();
        if collected.is_empty() {
            debug!("Zero-length file => no addresses, acceptable");
        } else {
            assert_eq!(collected.len(), 1, "Probably one parse error from zero-length file");
            let first = &collected[0];
            assert!(matches!(first, Err(OsmPbfParseError::OsmPbf(_))), "Likely parse error");
        }
    }

    #[traced_test]
    fn test_list_all_addresses_multiple_files_mixture() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(&temp_dir).expect("DB should open");
        let md_path = temp_dir.path().join("maryland-latest.osm.pbf");
        let va_path = temp_dir.path().join("virginia-latest.osm.pbf");
        {
            let rt = Runtime::new().unwrap();
            rt.block_on(create_corrupt_pbf(&md_path));
            rt.block_on(create_corrupt_pbf(&va_path));
        }
        let iter = list_all_addresses_in_pbf_dir(temp_dir.path(),db).unwrap();
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
