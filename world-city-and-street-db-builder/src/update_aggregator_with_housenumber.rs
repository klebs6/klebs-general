// ---------------- [ File: src/update_aggregator_with_housenumber.rs ]
crate::ix!();

/// Extracts a [`HouseNumberRange`] (if any) from the element and, if found,
/// updates the aggregator entry for the element's street (taken from `record`).
pub fn update_aggregator_with_housenumber(
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

#[cfg(test)]
#[disable]
mod test_update_aggregator_with_housenumber {
    use super::*;
    use std::collections::HashMap;
    use osmpbf::{Node, Element};
    use tracing::{trace, debug};

    /// A small helper to create a mock OSM `Node` with specific tags for testing.
    /// We'll store tags in `node.tags_mut()` so `Element::Node(node).tags()` yields them.
    fn make_node_with_tags(id: i64, tags: &[(&str, &str)]) -> Element<'static> {
        let mut node = Node::default();
        node.set_id(id);
        for (k, v) in tags {
            node.tags_mut().insert(k.to_string(), v.to_string());
        }
        Element::Node(node)
    }

    /// A minimal helper for building an `AddressRecord` with a street,
    /// ignoring city/postcode if not relevant for the aggregator usage.
    fn make_address_record_with_street(street_name: &str) -> AddressRecord {
        AddressRecordBuilder::default()
            .street(Some(StreetName::new(street_name).unwrap()))
            .build()
            .expect("Should build a minimal AddressRecord with just a street")
    }

    /// A convenience for comparing the aggregator's contents against expected ranges for a street.
    fn assert_street_ranges(
        aggregator: &HashMap<StreetName, Vec<HouseNumberRange>>,
        street_name: &str,
        expected: &[HouseNumberRange]
    ) {
        // Locate the aggregator entry for the specified street name.
        let street_key_opt = aggregator.keys().find(|k| k.name() == street_name);
        assert!(
            street_key_opt.is_some(),
            "No entry found for street: {}",
            street_name
        );
        let street_key = street_key_opt.unwrap();
        let stored_ranges = aggregator.get(street_key).unwrap();

        assert_eq!(
            stored_ranges, expected,
            "HouseNumber ranges mismatch for street '{}'.\nGot:  {:?}\nWant: {:?}",
            street_name, stored_ranges, expected
        );
    }

    /// A helper to build a `HouseNumberRange`.
    fn hnr(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    #[test]
    fn test_no_housenumber_in_element_no_aggregator_update() {
        // The OSM element lacks "addr:housenumber" => Ok(None) => aggregator not updated
        let element = make_node_with_tags(1, &[
            ("addr:city", "IgnoreCity"),
            ("highway", "residential"),
        ]);
        let record = make_address_record_with_street("SomeStreet");
        let mut aggregator: HashMap<StreetName, Vec<HouseNumberRange>> = HashMap::new();

        update_aggregator_with_housenumber(&element, &record, &mut aggregator);
        assert!(aggregator.is_empty(), "No housenumber => aggregator remains empty");
    }

    #[test]
    fn test_valid_housenumber_updates_aggregator() {
        // "addr:housenumber" => "10-20" => extracted => aggregator updated with [10..20].
        let element = make_node_with_tags(2, &[
            ("addr:housenumber", "10-20"),
            ("some", "tag"),
        ]);
        let record = make_address_record_with_street("TestStreet");
        let mut aggregator = HashMap::new();

        update_aggregator_with_housenumber(&element, &record, &mut aggregator);
        assert_eq!(aggregator.len(), 1, "One street entry expected");
        assert_street_ranges(&aggregator, "teststreet", &[hnr(10,20)]);
    }

    #[test]
    fn test_parse_error_in_housenumber_no_aggregator_update() {
        // If `extract_house_number_range_from_element` fails, aggregator not updated
        // We'll pass a "nonsense" housenumber so the parse fails.
        let element = make_node_with_tags(3, &[
            ("addr:housenumber", "invalid??"),
        ]);
        let record = make_address_record_with_street("FailStreet");
        let mut aggregator = HashMap::new();

        // The extraction function likely returns an error => aggregator not updated
        update_aggregator_with_housenumber(&element, &record, &mut aggregator);
        assert!(aggregator.is_empty(), "Parse error => no aggregator update");
    }

    #[test]
    fn test_found_housenumber_but_no_street_in_record() {
        // If record.street() is None => aggregator can't be updated with no street key.
        let element = make_node_with_tags(4, &[
            ("addr:housenumber", "50-60"),
        ]);
        let record = AddressRecordBuilder::default()
            // city or postcode might be present, but no street
            .city(Some(CityName::new("CityOnly").unwrap()))
            .build()
            .expect("Record with city but no street");
        let mut aggregator = HashMap::new();

        update_aggregator_with_housenumber(&element, &record, &mut aggregator);
        assert!(aggregator.is_empty(), "No street => aggregator not updated, even though housenumber found");
    }

    #[test]
    fn test_aggregator_appends_new_range_for_same_street() {
        // aggregator already has [10..20] for "MainSt". The new housenumber => "25" => appended => [10..20, 25..25]
        // We do not unify them (the aggregator is just storing subranges).
        // If you needed to unify, you'd do so separately in e.g. unify_new_and_existing_ranges approach.
        let element = make_node_with_tags(5, &[
            ("addr:housenumber", "25"),
        ]);
        let record = make_address_record_with_street("MainSt");

        let mut aggregator: HashMap<StreetName, Vec<HouseNumberRange>> = HashMap::new();
        let street_key = StreetName::new("MainSt").unwrap();
        aggregator.insert(street_key.clone(), vec![hnr(10,20)]);

        update_aggregator_with_housenumber(&element, &record, &mut aggregator);

        // Now aggregator[MainSt] => [10..20, 25..25]
        let expected = vec![hnr(10,20), hnr(25,25)];
        let ranges = aggregator.get(&street_key).unwrap();
        assert_eq!(ranges, &expected);
    }
}
