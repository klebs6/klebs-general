// ---------------- [ File: src/process_single_osm_element.rs ]
// ---------------- [ File: src/process_single_osm_element.rs ]
crate::ix!();

/// For one OSM element, we:
///   1. Attempt to parse an [`AddressRecord`] via `AddressRecord::try_from(...)`.
///   2. Extract a [`HouseNumberRange`] if present.
///   3. If both a street name and house‐number range exist, store them in `street_hnr_map`.
pub fn process_single_osm_element(
    element:        &osmpbf::Element,
    country:        &Country,
    addresses:      &mut Vec<AddressRecord>,
    street_hnr_map: &mut HashMap<StreetName, Vec<HouseNumberRange>>,

) -> Result<(), OsmPbfParseError> {

    trace!("process_single_osm_element: analyzing an OSM element");

    // Attempt an AddressRecord
    let record_result = AddressRecord::try_from((element, country));
    if let Ok(addr) = &record_result {
        debug!("process_single_osm_element: got AddressRecord => pushing to addresses");
        addresses.push(addr.clone());
    }

    // Attempt a HouseNumberRange
    let hnr_result = extract_house_number_range_from_element(element);

    // If we found a HNR and we have a valid street, record it
    if let Ok(Some(hnr)) = hnr_result {
        let maybe_street = match &record_result {
            Ok(addr) => addr.street().clone(),  // If we already have an AddressRecord
            Err(_) => {
                // If the AddressRecord parse failed, we can try again or skip
                // Try again just to see if there's a street
                if let Ok(addr2) = AddressRecord::try_from((element, country)) {
                    addr2.street().clone()
                } else {
                    None
                }
            }
        };

        if let Some(street) = maybe_street {
            debug!(
                "process_single_osm_element: found HNR={:?} => adding to street='{}'",
                hnr,
                street
            );
            street_hnr_map.entry(street.clone()).or_default().push(hnr);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test_process_single_osm_element {
    use super::*;
    use std::collections::HashMap;

    /// Returns a sample country. In real code, you might pick `Country::USA` or another.
    fn test_country() -> Country {
        Country::USA
    }

    #[traced_test]
    fn test_valid_address_record_and_hnr() {
        // A node with tags for city, street, postcode, plus housenumber => 1) parse address, 2) parse HNR
        let tags = &[
            ("addr:city", "Baltimore"),
            ("addr:street", "North Avenue"),
            ("addr:postcode", "21201"),
            ("addr:housenumber", "10-20"),
        ];
        let node = MockNode::new(111, tags);

        let mut addresses = Vec::new();
        let mut street_hnr_map = HashMap::new();
        let result = process_single_osm_element(
            &node.as_element(),
            &test_country(),
            &mut addresses,
            &mut street_hnr_map,
        );
        assert!(
            result.is_ok(),
            "Should succeed processing a valid address & housenumber"
        );

        // 1) Check that an AddressRecord was created
        assert_eq!(addresses.len(), 1, "Should push exactly one address record");
        let record = &addresses[0];
        assert_eq!(record.city().as_ref().unwrap().name(), "baltimore");
        assert_eq!(record.street().as_ref().unwrap().name(), "north avenue");
        assert_eq!(record.postcode().as_ref().unwrap().code(), "21201");

        // 2) Check that the HNR was extracted and assigned under street "north avenue"
        assert_eq!(street_hnr_map.len(), 1, "Should have exactly one street in the map");
        assert_street_house_number_map_contains(
            &street_hnr_map,
            "north avenue",
            &[[10, 20]],
        );
    }

    #[traced_test]
    fn test_no_hnr_but_valid_address_record() {
        // Node has city/street/postcode but no housenumber => addresses gets appended, 
        // but no entry in street_hnr_map
        let tags = &[
            ("addr:city", "Test City"),
            ("addr:street", "Test Street"),
            ("addr:postcode", "99999"),
        ];
        let node = MockNode::new(222, tags);

        let mut addresses = Vec::new();
        let mut street_hnr_map = HashMap::new();
        let result = process_single_osm_element(
            &node.as_element(),
            &test_country(),
            &mut addresses,
            &mut street_hnr_map,
        );
        assert!(
            result.is_ok(),
            "Should succeed (valid address, no housenumber is fine)"
        );

        // Should have 1 address
        assert_eq!(addresses.len(), 1);
        let record = &addresses[0];
        assert_eq!(record.city().as_ref().unwrap().name(), "test city");
        assert_eq!(record.street().as_ref().unwrap().name(), "test street");
        assert_eq!(record.postcode().as_ref().unwrap().code(), "99999");

        // But no HNR => street_hnr_map should remain empty
        assert!(street_hnr_map.is_empty(), "No housenumber => no street entry");
    }

    #[traced_test]
    fn test_hnr_but_no_valid_address_record() {
        // Node has housenumber, but city/street is missing or invalid => parse fails => no addresses.
        // We attempt to parse address again if the first time fails, just to see if there's a street name.
        // If there's no valid street => no entry in street_hnr_map.
        let tags = &[
            ("addr:housenumber", "50-60"),
            // No city or street => can't form an AddressRecord
        ];
        let node = MockNode::new(333, tags);

        let mut addresses = Vec::new();
        let mut street_hnr_map = HashMap::new();
        let result = process_single_osm_element(
            &node.as_element(),
            &test_country(),
            &mut addresses,
            &mut street_hnr_map,
        );
        assert!(result.is_ok(), "Should not produce a parse error at the function level");

        // AddressRecord parse fails => addresses is empty
        assert!(addresses.is_empty(), "No valid address => none pushed");

        // We re-try for a street if the first parse fails, but still no street => no map entry
        assert!(street_hnr_map.is_empty(), "No street => no HNR insertion");
    }

    #[traced_test]
    fn test_extract_hnr_and_street_even_if_addressrecord_failed_initially() {
        // Suppose the first parse fails for some reason (maybe an invalid postcode),
        // but we do have a city/street => so we re-try AddressRecord::try_from => 
        // if the second parse also fails, we skip. 
        // But let's see a scenario where the second parse might succeed if the first parse 
        // was aborted for a different reason. (If your code or conditions differ, adjust the test.)
        //
        // We'll emulate a scenario:
        //   - The first parse fails on e.g. invalid postcode
        //   - But the "second parse" is forcibly the same logic, so it likely fails again.
        //   If your code had a partial parse that sometimes might succeed, you'd test that scenario.

        let tags = &[
            ("addr:city", "TroubleTown"),
            ("addr:street", "Broken Boulevard"),
            ("addr:postcode", ""), // if your code fails on empty or invalid
            ("addr:housenumber", "99"),
        ];
        let node = MockNode::new(444, tags);

        let mut addresses = Vec::new();
        let mut street_hnr_map = HashMap::new();
        let result = process_single_osm_element(&node.as_element(), &test_country(), &mut addresses, &mut street_hnr_map);
        assert!(result.is_ok());

        // Because the postcode is empty, the AddressRecord parse might fail => addresses = []
        assert!(addresses.is_empty(), "No valid AddressRecord => none added");

        // The function tries again to parse address if the first parse failed,
        // but it uses the same data => likely fails again => no street => no HNR insertion
        assert!(street_hnr_map.is_empty(), "No valid street => no insertion");
    }

    #[traced_test]
    fn test_parse_error_in_hnr_not_fatal_for_address() {
        // If housenumber parse fails, it returns an Err => we skip adding to street map
        // but the address record could still succeed if city/street/postcode is valid.
        let tags = &[
            ("addr:city", "City"),
            ("addr:street", "Street"),
            ("addr:postcode", "22222"),
            ("addr:housenumber", "invalidRange"), // parse fails
        ];
        let node = MockNode::new(555, tags);

        let mut addresses = Vec::new();
        let mut street_hnr_map = HashMap::new();

        let result = process_single_osm_element(
            &node.as_element(), 
            &test_country(), 
            &mut addresses, 
            &mut street_hnr_map
        );

        assert!(result.is_ok());

        // We get the address, but no HNR
        assert_eq!(addresses.len(), 1, "AddressRecord parse ok => one address");
        assert!(street_hnr_map.is_empty(), "HNR parse fails => no entry in street map");
    }

    #[traced_test]
    fn test_no_parse_error_spread() {
        // The function returns Ok(()) regardless of parse failures internally, 
        // we rely on the warnings/logs. We'll ensure no panic or error is returned 
        // if both address parse and housenumber parse fail.
        let tags = &[
            ("foo", "bar"),
            ("another", "tag")
        ];
        let node = MockNode::new(666, tags);

        let mut addresses = Vec::new();
        let mut street_hnr_map = HashMap::new();

        let result = process_single_osm_element(
            &node.as_element(), 
            &test_country(), 
            &mut addresses, 
            &mut street_hnr_map
        );

        assert!(result.is_ok());
        // addresses empty, street_hnr_map empty
        assert!(addresses.is_empty());
        assert!(street_hnr_map.is_empty());
    }
}
