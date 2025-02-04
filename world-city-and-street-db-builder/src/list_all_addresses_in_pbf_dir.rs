// ---------------- [ File: src/list_all_addresses_in_pbf_dir.rs ]
crate::ix!();

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
pub fn list_all_addresses_in_pbf_dir(
    pbf_dir: impl AsRef<Path>,
    db: Arc<Mutex<Database>>,
) -> Result<impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>, OsmPbfParseError> {
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

/// Reads the specified directory, returning a `Vec<PathBuf>` of all `.pbf` files found.
///
/// # Returns
///
/// * `Ok(Vec<PathBuf>)` if the directory is accessible.
/// * `Err(OsmPbfParseError)` if reading the directory fails.
fn gather_pbf_files(pbf_dir: &Path) -> Result<Vec<PathBuf>, OsmPbfParseError> {
    trace!("gather_pbf_files: scanning directory {:?}", pbf_dir);
    let entries = fs::read_dir(pbf_dir)
        .map_err(|io_err| OsmPbfParseError::IoError(io_err))?;

    let mut pbf_files = Vec::new();
    for entry_result in entries {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                error!("gather_pbf_files: error reading entry in {:?}: {}", pbf_dir, e);
                return Err(OsmPbfParseError::IoError(e));
            }
        };
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("pbf") {
            debug!("gather_pbf_files: found PBF file {:?}", path);
            pbf_files.push(path);
        }
    }

    Ok(pbf_files)
}

/// Builds a single chained iterator of [`WorldAddress`] across all `.pbf` files.
/// For each file, we attempt to determine the region and parse the file, skipping
/// those with unknown regions.
///
/// # Returns
///
/// * `Ok(Box<dyn Iterator<Item = Result<WorldAddress, OsmPbfParseError>>>)` on success.
/// * `Err(OsmPbfParseError)` if an error arises parsing a given file.
fn chain_addresses_across_files(
    pbf_files: Vec<PathBuf>,
    known_regions: &[WorldRegion],
    db: Arc<Mutex<Database>>,
    pbf_dir: &Path,
) -> Result<Box<dyn Iterator<Item = Result<WorldAddress, OsmPbfParseError>>>, OsmPbfParseError> {
    trace!("chain_addresses_across_files: building iterator for {} files", pbf_files.len());

    // Start with an empty iterator
    let mut chained_iter = Box::new(iter::empty()) as Box<dyn Iterator<Item = _>>;

    for file_path in pbf_files {
        match select_region_for_file(&file_path, known_regions, pbf_dir) {
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

/// Attempts to determine which region a given file belongs to. Returns `Some` if found,
/// or `None` if the file does not match any known region heuristics.
fn select_region_for_file(
    file_path: &Path,
    known_regions: &[WorldRegion],
    base_dir: &Path,
) -> Option<WorldRegion> {
    trace!(
        "select_region_for_file: file_path={:?}, base_dir={:?}",
        file_path,
        base_dir
    );
    find_region_for_file(file_path, known_regions, base_dir)
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

    #[test]
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

    #[test]
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

    #[test]
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
