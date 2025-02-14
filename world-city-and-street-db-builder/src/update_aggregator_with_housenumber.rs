// ---------------- [ File: src/update_aggregator_with_housenumber.rs ]
crate::ix!();

/// Extracts a [`HouseNumberRange`] (if any) from the element and, if found,
/// updates the aggregator entry for the element's street (taken from `record`).
pub fn update_aggregator_with_housenumber(
    element:    &osmpbf::Element,
    record:     &AddressRecord,
    aggregator: &mut HouseNumberAggregator,
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
mod test_update_aggregator_with_housenumber {
    use super::*;
    use std::collections::HashMap;
    use osmpbf::{Node, Element};
    use tracing::{trace, debug};

    /// A convenience for comparing the aggregator's contents against expected ranges for a street.
    fn assert_street_ranges(
        aggregator: &HouseNumberAggregator,
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

    #[traced_test]
    fn test_no_housenumber_in_element_no_aggregator_update() {
        // The OSM node lacks "addr:housenumber" => Ok(None) => aggregator not updated
        let node = MockNode::new(1, &[
            ("addr:city", "IgnoreCity"),
            ("highway", "residential"),
        ]);
        let record = make_address_record_with_street("SomeStreet");
        let mut aggregator = HouseNumberAggregator::new(&example_region());

        update_aggregator_with_housenumber(&node.as_element(), &record, &mut aggregator);
        assert!(aggregator.is_empty(), "No housenumber => aggregator remains empty");
    }

    #[traced_test]
    fn test_valid_housenumber_updates_aggregator() {
        // "addr:housenumber" => "10-20" => extracted => aggregator updated with [10..20].
        let node = MockNode::new(2, &[
            ("addr:housenumber", "10-20"),
            ("some", "tag"),
        ]);
        let record = make_address_record_with_street("TestStreet");
        let region = example_region();
        let mut aggregator = HouseNumberAggregator::new(&region);

        update_aggregator_with_housenumber(&node.as_element(), &record, &mut aggregator);
        assert_eq!(aggregator.len(), 1, "One street entry expected");
        assert_street_ranges(&aggregator, "teststreet", &[hnr(10,20)]);
    }

    #[traced_test]
    fn test_parse_error_in_housenumber_no_aggregator_update() {
        // If `extract_house_number_range_from_element` fails, aggregator not updated
        // We'll pass a "nonsense" housenumber so the parse fails.
        let node = MockNode::new(3, &[
            ("addr:housenumber", "invalid??"),
        ]);
        let record = make_address_record_with_street("FailStreet");
        let region = example_region();
        let mut aggregator = HouseNumberAggregator::new(&region);

        // The extraction function likely returns an error => aggregator not updated
        update_aggregator_with_housenumber(&node.as_element(), &record, &mut aggregator);
        assert!(aggregator.is_empty(), "Parse error => no aggregator update");
    }

    #[traced_test]
    fn test_found_housenumber_but_no_street_in_record() {
        // If record.street() is None => aggregator can't be updated with no street key.
        let node = MockNode::new(4, &[
            ("addr:housenumber", "50-60"),
        ]);
        let record = AddressRecordBuilder::default()
            // city or postcode might be present, but no street
            .city(Some(CityName::new("CityOnly").unwrap()))
            .build()
            .expect("Record with city but no street");
        let region = example_region();
        let mut aggregator = HouseNumberAggregator::new(&region);

        update_aggregator_with_housenumber(&node.as_element(), &record, &mut aggregator);
        assert!(aggregator.is_empty(), "No street => aggregator not updated, even though housenumber found");
    }

    #[traced_test]
    fn test_aggregator_appends_new_range_for_same_street() {
        // aggregator already has [10..20] for "MainSt". The new housenumber => "25" => appended => [10..20, 25..25]
        // We do not unify them (the aggregator is just storing subranges).
        // If you needed to unify, you'd do so separately in e.g. unify_new_and_existing_ranges approach.
        let node = MockNode::new(5, &[
            ("addr:housenumber", "25"),
        ]);
        let record = make_address_record_with_street("MainSt");

        let mut aggregator = HouseNumberAggregator::new(&example_region());
        let street_key = StreetName::new("MainSt").unwrap();
        aggregator.insert(street_key.clone(), vec![hnr(10,20)]);

        update_aggregator_with_housenumber(&node.as_element(), &record, &mut aggregator);

        // Now aggregator[MainSt] => [10..20, 25..25]
        let expected = vec![hnr(10,20), hnr(25,25)];
        let ranges = aggregator.get(&street_key).unwrap();
        assert_eq!(ranges, &expected);
    }
}
