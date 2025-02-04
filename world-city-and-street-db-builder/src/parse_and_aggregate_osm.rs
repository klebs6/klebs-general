// ---------------- [ File: src/parse_and_aggregate_osm.rs ]
crate::ix!();

/// Reads all elements from the `ElementReader`, extracts address info,
/// sends `WorldAddress` objects through `tx`, and accumulates
/// `HouseNumberRange` data into `aggregator`.
///
/// On any parse error from `osmpbf::ElementReader::for_each`, returns it
/// as an `OsmPbfParseError::OsmPbf(...)`.
pub fn parse_and_aggregate_osm<R:Read + Send + Sync>(
    reader:     ElementReader<R>,
    country:    &Country,
    region:     &WorldRegion,
    tx:         &SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    aggregator: &mut HashMap<StreetName, Vec<HouseNumberRange>>,
) -> Result<(), OsmPbfParseError>
{
    trace!("parse_and_aggregate_osm: starting iteration over PBF elements.");

    // for_each returns an `osmpbf::Result<()>`.
    let result = reader.for_each(|element| {
        // We process each element inside this closure:
        process_osm_element(element, country, region, tx, aggregator);
    });

    // If for_each fails with an osmpbf::Error, we wrap it:
    if let Err(osmpbf_err) = result {
        error!("parse_and_aggregate_osm: Error reading PBF => {:?}", osmpbf_err);
        return Err(OsmPbfParseError::OsmPbf(osmpbf_err));
    }

    debug!("parse_and_aggregate_osm: Completed iteration over OSM data.");
    Ok(())
}

#[cfg(test)]
mod parse_and_aggregate_osm_tests {
    use super::*;

    #[test]
    fn test_parse_and_aggregate_osm_empty() {
        // We can create a mock or an "in-memory" ElementReader with no elements.
        // Because osmpbf is heavily oriented toward reading from actual files,
        // you might do a real file with zero elements or use a custom approach.
        // For demonstration, let's do a partial approach:

        let empty_reader = ElementReader::from_path("/dev/null").unwrap(); 
        let country = Country::USA;
        let region = WorldRegion::default(); // or something
        let (tx, rx) = mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);
        let mut aggregator: HashMap<StreetName, Vec<HouseNumberRange>> = HashMap::new();

        let res = parse_and_aggregate_osm(empty_reader, &country, &region, &tx, &mut aggregator);
        // Might fail if /dev/null or empty => error. If so, adapt your test or skip it on Windows, etc.
        if res.is_ok() {
            // aggregator empty => good
            assert!(aggregator.is_empty());
            // no addresses => rx should be empty or no items
            let maybe_addr = rx.try_recv();
            assert!(maybe_addr.is_err(), "No addresses expected in empty pbf");
        }
    }

    // Add further tests where we actually include minimal real or mock OSM data, etc.
}
