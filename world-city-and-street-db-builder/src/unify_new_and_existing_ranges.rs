// ---------------- [ File: src/unify_new_and_existing_ranges.rs ]
crate::ix!();

/// Merges newly provided house‐number ranges with the existing set.
/// Uses `merge_house_number_range` for each new range.
pub fn unify_new_and_existing_ranges(
    mut current: Vec<HouseNumberRange>,
    new_ranges: &[HouseNumberRange],
) -> Vec<HouseNumberRange> {
    trace!(
        "unify_new_and_existing_ranges: merging {} new ranges into {} existing",
        new_ranges.len(),
        current.len()
    );

    for rng in new_ranges {
        current = merge_house_number_range(current, rng);
    }
    current
}
