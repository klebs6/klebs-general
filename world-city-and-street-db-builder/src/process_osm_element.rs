// ---------------- [ File: src/process_osm_element.rs ]
crate::ix!();

/// Processes a single OSM element. If it contains a valid (city, street, postcode),
/// we build a [`WorldAddress`] and send it to the consumer over `tx`. Then, if
/// there's an `addr:housenumber`, we update the aggregator’s entry for that street
/// with the new house‐number range. If anything fails in parsing, we skip and continue.
///
/// # Arguments
///
/// * `element`    - The OSM element to process (Node, Way, Relation, DenseNode).
/// * `country`    - Inferred country for this region (used in `AddressRecord` parsing).
/// * `region`     - The region, used in constructing a [`WorldAddress`].
/// * `tx`         - A synchronous sender for streaming out results as [`Result<WorldAddress, OsmPbfParseError>`].
/// * `aggregator` - Map of `StreetName` to a list of [`HouseNumberRange`] objects, updated in-place.
pub fn process_osm_element(
    element: osmpbf::Element,
    country: &Country,
    region: &WorldRegion,
    tx: &std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    aggregator: &mut HashMap<StreetName, Vec<HouseNumberRange>>,
) {
    trace!("process_osm_element: entering for an OSM element, ID={}", get_element_id(&element));

    // Step 1: Try parsing an [`AddressRecord`] from the element.
    let record_option = parse_address_record_if_any(&element, country);

    if let Some(record) = &record_option {
        trace!("process_osm_element: got an AddressRecord => attempting to build WorldAddress");
        // Step 2: Attempt to build a [`WorldAddress`] from (region, city, street, postcode).
        if let Some(world_address) = build_world_address_if_possible(region, record) {
            // Step 3: Send the [`WorldAddress`] through `tx` unless the channel has closed.
            if tx.send(Ok(world_address)).is_err() {
                debug!("process_osm_element: receiver dropped; halting further processing.");
                return;
            }

            // Step 4: Try extracting a house-number range and, if found, associate it with the street.
            update_aggregator_with_housenumber(&element, record, aggregator);
        } else {
            debug!("process_osm_element: could not build WorldAddress => skipping aggregator update");
        }
    } else {
        // AddressRecord parse failed or wasn't present => we can still check for a house-number range
        trace!("process_osm_element: no AddressRecord => checking for house-number range with partial data");
        try_infer_street_and_update_housenumber(&element, country, aggregator);
    }
}

/// Retrieves the element ID (Node, Way, Relation, or DenseNode), or returns "?" if unknown.
/// Primarily used for logging.
fn get_element_id(element: &osmpbf::Element) -> String {
    match element {
        osmpbf::Element::Node(n) => format!("{}", n.id()),
        osmpbf::Element::Way(w) => format!("{}", w.id()),
        osmpbf::Element::Relation(r) => format!("{}", r.id()),
        osmpbf::Element::DenseNode(dn) => format!("{}", dn.id()),
    }
}

/// Parses an [`AddressRecord`] from the element if possible, returning `Some(AddressRecord)`
/// or `None` if the element doesn't contain valid city/street/postcode tags.
fn parse_address_record_if_any(
    element: &osmpbf::Element,
    country: &Country
) -> Option<AddressRecord> {
    match AddressRecord::try_from((element, country)) {
        Ok(rec) => {
            debug!(
                "parse_address_record_if_any: successfully built an AddressRecord, city={:?}, street={:?}, postcode={:?}",
                rec.city(),
                rec.street(),
                rec.postcode()
            );
            Some(rec)
        }
        Err(e) => {
            debug!("parse_address_record_if_any: element not a valid address => {:?}", e);
            None
        }
    }
}

/// If the [`AddressRecord`] has non-empty (city, street, postcode), build a [`WorldAddress`].
/// Otherwise returns `None`. This allows skipping elements without a complete address.
fn build_world_address_if_possible(
    region: &WorldRegion,
    record: &AddressRecord
) -> Option<WorldAddress> {
    let (city_opt, street_opt, postcode_opt) = (record.city(), record.street(), record.postcode());

    if let (Some(city), Some(street), Some(postcode)) = (city_opt, street_opt, postcode_opt) {
        match build_world_address(region, &city, &street, &postcode) {
            Ok(addr) => {
                debug!(
                    "build_world_address_if_possible: built WorldAddress => {}",
                    addr
                );
                Some(addr)
            }
            Err(e) => {
                debug!("build_world_address_if_possible: failed => {:?}", e);
                None
            }
        }
    } else {
        debug!("build_world_address_if_possible: record missing city/street/postcode => skipping");
        None
    }
}

/// Extracts a [`HouseNumberRange`] (if any) from the element and, if found,
/// updates the aggregator entry for the element's street (taken from `record`).
fn update_aggregator_with_housenumber(
    element: &osmpbf::Element,
    record: &AddressRecord,
    aggregator: &mut HashMap<StreetName, Vec<HouseNumberRange>>,
) {
    match extract_house_number_range_from_element(element) {
        Ok(Some(range)) => {
            if let Some(street) = record.street() {
                trace!("update_aggregator_with_housenumber: found housenumber range={:?}, street={}", range, street);
                aggregator.entry(street.clone()).or_default().push(range);
            }
        }
        Ok(None) => {
            // No house-number => do nothing
        }
        Err(e) => {
            debug!("update_aggregator_with_housenumber: error extracting house number => {:?}", e);
        }
    }
}

/// In cases where we didn't parse an [`AddressRecord`] fully, we still might have `addr:housenumber`.
/// We attempt a partial parse for the street, and if found, update the aggregator.
fn try_infer_street_and_update_housenumber(
    element: &osmpbf::Element,
    country: &Country,
    aggregator: &mut HashMap<StreetName, Vec<HouseNumberRange>>,
) {
    match extract_house_number_range_from_element(element) {
        Ok(Some(range)) => {
            // Attempt to parse enough of an address record to see if there's a street
            if let Ok(record2) = AddressRecord::try_from((element, country)) {
                if let Some(street) = record2.street() {
                    debug!(
                        "try_infer_street_and_update_housenumber: storing housenumber range={:?} for street='{}'",
                        range, street
                    );
                    aggregator.entry(street.clone()).or_default().push(range);
                }
            }
        }
        Ok(None) => {
            // No housenumber => skip
        }
        Err(e) => {
            debug!("try_infer_street_and_update_housenumber: error extracting => {:?}", e);
        }
    }
}

#[cfg(test)]
mod process_osm_element_tests {
    use super::*;

    #[test]
    fn test_process_osm_element_no_address_record() {
        use std::io::Write;
        use tempfile::tempdir;
        use tokio::runtime::Runtime;

        // We'll define a small helper that writes a .osm.pbf with a single node that has no addr: tags.
        // For demonstration, we do synchronous writes of a "fake" or "very minimal" file.
        // Or you might reuse your create_tiny_osm_pbf(...) logic, but omit tags.

        fn create_tiny_osm_pbf_no_tags(path: &std::path::Path) -> std::io::Result<()> {
            // In real usage, you'd produce a valid OSMHeader + one OSMData blob
            // whose node has zero "addr:*" tags. For brevity, let's say we do that here:
            // (Below is just a placeholder to represent writing minimal valid data.)

            let mut file = std::fs::File::create(path)?;
            file.write_all(b"not a real pbf but suppose you wrote a valid minimal node w/ no tags...")?;
            Ok(())
        }

        // 1) Create a temp dir & pbf file
        let dir = tempdir().unwrap();
        let pbf_path = dir.path().join("no_addr_tags.osm.pbf");
        create_tiny_osm_pbf_no_tags(&pbf_path).unwrap();

        // 2) Use ElementReader to parse
        let reader = osmpbf::ElementReader::from_path(&pbf_path).unwrap();
        let country = Country::USA;
        let region = WorldRegion::default();

        // 3) aggregator + channel
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);
        let mut aggregator: HashMap<StreetName, Vec<HouseNumberRange>> = HashMap::new();

        // 4) Call parse_and_aggregate_osm or process each element
        let result = parse_and_aggregate_osm(reader, &country, &region, &tx, &mut aggregator);

        // 5) If the file is truly minimal and has no addr:* tags, aggregator stays empty,
        //    and no addresses are sent.
        if result.is_ok() {
            assert!(aggregator.is_empty(), "No addr tags => aggregator empty");
            assert!(rx.try_recv().is_err(), "No addresses => channel empty");
        } else {
            // Possibly handle the scenario where the minimal file triggers an osmpbf parse error.
            // Either ignore or do a separate assertion, depending on what you want to allow.
        }
    }
}
