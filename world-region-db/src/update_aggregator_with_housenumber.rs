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

    fn assert_street_ranges(
        aggregator: &HouseNumberAggregator,
        street_name: &str,
        expected: &[HouseNumberRange]
    ) {
        let street_key_opt = aggregator
            .keys()
            .find(|k| k.name() == street_name);
        assert!(
            street_key_opt.is_some(),
            "No aggregator entry for street: {}",
            street_name
        );
        let street_key = street_key_opt.unwrap();
        let actual = aggregator.get(street_key).unwrap();

        assert_eq!(actual, expected,
            "HouseNumberRange mismatch for street '{}'.\nWanted: {:?}\nGot:    {:?}",
            street_name,
            expected,
            actual
        );
    }

    fn hnr(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    #[traced_test]
    fn test_no_housenumber_in_element_no_aggregator_update() {
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
        let node = MockNode::new(2, &[
            ("addr:housenumber", "10-20"),
            ("some", "tag"),
        ]);
        let record = make_address_record_with_street("TestStreet");
        let mut aggregator = HouseNumberAggregator::new(&example_region());

        update_aggregator_with_housenumber(&node.as_element(), &record, &mut aggregator);
        assert_eq!(aggregator.len(), 1, "One street entry expected");
        assert_street_ranges(&aggregator, "teststreet", &[hnr(10,20)]);
    }

    #[traced_test]
    fn test_parse_error_in_housenumber_no_aggregator_update() {
        let node = MockNode::new(3, &[
            ("addr:housenumber", "invalid??"),
        ]);
        let record = make_address_record_with_street("FailStreet");
        let mut aggregator = HouseNumberAggregator::new(&example_region());

        update_aggregator_with_housenumber(&node.as_element(), &record, &mut aggregator);
        assert!(aggregator.is_empty(), "Parse error => no aggregator update");
    }

    #[traced_test]
    fn test_found_housenumber_but_no_street_in_record() {
        let node = MockNode::new(4, &[
            ("addr:housenumber", "50-60"),
        ]);
        // city but no street => record.street() is None
        let record = AddressRecordBuilder::default()
            .city(Some(CityName::new("CityOnly").unwrap()))
            .build()
            .unwrap();

        let mut aggregator = HouseNumberAggregator::new(&example_region());
        update_aggregator_with_housenumber(&node.as_element(), &record, &mut aggregator);
        assert!(aggregator.is_empty(), "No street => aggregator not updated");
    }

    #[traced_test]
    fn test_aggregator_appends_new_range_for_same_street() {
        let node = MockNode::new(5, &[
            ("addr:housenumber", "25"),
        ]);
        let record = make_address_record_with_street("MainSt");

        let mut aggregator = HouseNumberAggregator::new(&example_region());
        // Insert existing [10..20] for MainSt
        let key = StreetName::new("MainSt").unwrap();
        aggregator.insert(key.clone(), vec![hnr(10,20)]);

        update_aggregator_with_housenumber(&node.as_element(), &record, &mut aggregator);

        // aggregator[MainSt] => [10..20, 25..25]
        let expected = vec![hnr(10,20), hnr(25,25)];
        let actual = aggregator.get(&key).unwrap();
        assert_eq!(actual, &expected, "Should append new subrange for the same street");
    }
}
