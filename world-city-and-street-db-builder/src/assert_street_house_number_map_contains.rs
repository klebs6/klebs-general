crate::ix!();

/// Asserts that `street_hnr_map` has exactly one entry under the given street name,
/// and that the list of ranges matches `expected_ranges`.
pub fn assert_street_house_number_map_contains(
    street_hnr_map: &HashMap<StreetName, Vec<HouseNumberRange>>,
    street_name: &str,
    expected_ranges: &[[u32; 2]],
) {
    let found = street_hnr_map.iter().find(|(st, _)| st.name() == street_name);
    assert!(found.is_some(), "Expected a street name '{}' in the map", street_name);
    let (street_key, ranges_vec) = found.unwrap();

    // Convert expected_ranges into HouseNumberRange for easy comparison
    let expected_vec: Vec<HouseNumberRange> = expected_ranges
        .iter()
        .map(|[start, end]| HouseNumberRange::new(*start, *end))
        .collect();

    assert_eq!(
        ranges_vec, &expected_vec,
        "Mismatch in house number subranges for street '{}'",
        street_key.name()
    );
}
