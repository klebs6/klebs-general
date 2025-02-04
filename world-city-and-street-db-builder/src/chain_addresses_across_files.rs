crate::ix!();

/// Builds a single chained iterator of [`WorldAddress`] across all `.pbf` files.
/// For each file, we attempt to determine the region and parse the file, skipping
/// those with unknown regions.
///
/// # Returns
///
/// * `Ok(Box<dyn Iterator<Item = Result<WorldAddress, OsmPbfParseError>>>)` on success.
/// * `Err(OsmPbfParseError)` if an error arises parsing a given file.
pub fn chain_addresses_across_files(
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
