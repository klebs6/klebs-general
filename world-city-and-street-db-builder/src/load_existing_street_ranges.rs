// ---------------- [ File: src/load_existing_street_ranges.rs ]
crate::ix!();

/// Loads existing house‐number ranges for the specified street from the DB.
pub fn load_existing_street_ranges(
    db: &mut Database,
    world_region: &WorldRegion,
    street: &StreetName,
) -> Result<Option<Vec<HouseNumberRange>>, DatabaseConstructionError> {
    trace!(
        "load_existing_street_ranges: loading for street='{}' in region={:?}",
        street,
        world_region
    );
    let existing = load_house_number_ranges(db, world_region, street)?;
    Ok(existing)
}
