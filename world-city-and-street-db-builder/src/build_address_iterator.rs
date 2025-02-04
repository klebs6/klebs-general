crate::ix!();

/// Builds an iterator of addresses from the specified `.pbf` directory.
pub fn build_address_iterator(
    db: Arc<Mutex<Database>>,
    pbf_dir: &Path
) -> Result<impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>, WorldCityAndStreetDbBuilderError> {
    trace!("build_address_iterator: listing addresses in dir={:?}", pbf_dir);
    list_all_addresses_in_pbf_dir(pbf_dir, db)
        .map_err(|e| {
            error!("build_address_iterator: error listing addresses => {:?}", e);
            WorldCityAndStreetDbBuilderError::OsmPbfParseFailure(e)
        })
}
