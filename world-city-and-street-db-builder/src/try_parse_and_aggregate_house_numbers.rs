crate::ix!();

/// Attempts to parse the OSM PBF data, populate `aggregator`, and stream out addresses.
/// Returns an error if parsing fails or an error is encountered mid-processing.
pub fn try_parse_and_aggregate_house_numbers<R:Read+Send+Sync>(
    reader: osmpbf::ElementReader<R>,
    country: &Country,
    world_region: &WorldRegion,
    tx: &std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    aggregator: &mut HashMap<StreetName, Vec<HouseNumberRange>>
) -> Result<(), OsmPbfParseError> {
    trace!(
        "try_parse_and_aggregate_house_numbers: Parsing OSM from reader with region={:?}, country={:?}",
        world_region, country
    );

    parse_and_aggregate_osm(
        reader,
        country,
        world_region,
        tx,
        aggregator
    )
}
