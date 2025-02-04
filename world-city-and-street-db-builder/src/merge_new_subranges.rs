crate::ix!();

/// Merges newly extracted subranges into the existing list, returning a consolidated list.
/// This example calls an existing helper (like `merge_house_number_range`) for each range.
///
/// # Returns
///
/// * A new `Vec<HouseNumberRange>` containing the merged results.
pub fn merge_new_subranges(
    mut current: Vec<HouseNumberRange>,
    new_ranges: Vec<HouseNumberRange>,
) -> Vec<HouseNumberRange> {
    trace!(
        "merge_new_subranges: current={} existing ranges, new={} ranges",
        current.len(),
        new_ranges.len()
    );

    for rng in new_ranges {
        current = merge_house_number_range(current, rng);
    }
    current
}
