crate::ix!();

/// Loads existing house‐number ranges for a street, merges new data, and stores the result.
pub fn process_street_house_numbers(
    db: &mut Database,
    region: &WorldRegion,
    street: &StreetName,
    new_ranges: &[HouseNumberRange],
) -> Result<(), DatabaseConstructionError> {
    trace!(
        "process_street_house_numbers: street='{}', merging {} new ranges",
        street,
        new_ranges.len()
    );

    let existing = load_existing_house_number_ranges(db, region, street)?;
    let merged = unify_new_and_existing_ranges(existing, new_ranges);
    store_merged_house_number_ranges(db, region, street, &merged)?;

    Ok(())
}
