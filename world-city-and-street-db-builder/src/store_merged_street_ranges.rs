crate::ix!();

/// Stores the merged house‐number ranges back into the DB for the given street.
pub fn store_merged_street_ranges(
    db: &mut Database,
    world_region: &WorldRegion,
    street: &StreetName,
    merged: &[HouseNumberRange],
) -> Result<(), OsmPbfParseError> {
    trace!(
        "store_merged_street_ranges: storing {} merged ranges for street='{}'",
        merged.len(),
        street
    );

    store_house_number_ranges(db, world_region, street, merged)?;
    Ok(())
}
