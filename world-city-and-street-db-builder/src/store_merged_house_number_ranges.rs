crate::ix!();

/// Stores the merged list of house‐number ranges back into the database.
pub fn store_merged_house_number_ranges(
    db: &mut Database,
    region: &WorldRegion,
    street: &StreetName,
    merged: &[HouseNumberRange],
) -> Result<(), DatabaseConstructionError> {
    trace!(
        "store_merged_house_number_ranges: storing {} ranges for street='{}' in region={:?}",
        merged.len(),
        street,
        region
    );

    store_house_number_ranges(db, region, street, merged)?;
    debug!(
        "store_merged_house_number_ranges: successfully stored ranges for street='{}'",
        street
    );
    Ok(())
}
