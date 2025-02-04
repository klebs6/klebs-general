crate::ix!();

/// Integrates a list of new house‐number ranges into existing DB data for the given street,
/// then writes the merged result back to the DB. Logs warnings if load/store operations fail.
///
/// # Arguments
///
/// * `db`           - Database to update.
/// * `world_region` - Region context (used in key derivation).
/// * `street`       - The street whose house‐number ranges are being updated.
/// * `new_ranges`   - A list of new [`HouseNumberRange`] values to be merged.
///
/// # Returns
///
/// * `Ok(())` on success, or if partial failures occurred but we can continue.
/// * `Err(OsmPbfParseError)` if a critical error prevents further processing.
pub fn integrate_house_number_subranges_for_street(
    db: &mut Database,
    world_region: &WorldRegion,
    street: &StreetName,
    new_ranges: Vec<HouseNumberRange>,
) -> Result<(), OsmPbfParseError> {
    trace!(
        "integrate_house_number_subranges_for_street: street='{}', merging {} new ranges",
        street,
        new_ranges.len()
    );

    // Step 1) Load existing ranges (if any)
    let existing_opt = match load_existing_street_ranges(db, world_region, street) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "integrate_house_number_subranges_for_street: could not load existing ranges for street='{}': {:?}",
                street,
                e
            );
            None
        }
    };

    // Step 2) Merge
    let merged = merge_new_subranges(existing_opt.unwrap_or_default(), new_ranges);

    // Step 3) Store
    match store_merged_street_ranges(db, world_region, street, &merged) {
        Ok(_) => {
            debug!(
                "integrate_house_number_subranges_for_street: successfully stored merged ranges for street='{}'",
                street
            );
        }
        Err(e) => {
            warn!(
                "integrate_house_number_subranges_for_street: could not store updated ranges for street='{}': {:?}",
                street, e
            );
        }
    }

    Ok(())
}
