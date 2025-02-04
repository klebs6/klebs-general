crate::ix!();

/// Loads existing house‐number ranges from the database for the specified street in the given region.
pub fn load_existing_house_number_ranges(
    db: &mut Database,
    region: &WorldRegion,
    street: &StreetName,
) -> Result<Vec<HouseNumberRange>, DatabaseConstructionError> {
    trace!(
        "load_existing_house_number_ranges: street='{}' in region={:?}",
        street,
        region
    );

    let existing_opt = load_house_number_ranges(db, region, street)?;
    let existing = existing_opt.unwrap_or_default();

    debug!(
        "load_existing_house_number_ranges: found {} existing ranges for street='{}'",
        existing.len(),
        street
    );
    Ok(existing)
}
