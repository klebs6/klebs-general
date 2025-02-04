// ---------------- [ File: src/write_house_number_ranges_into_storage.rs ]
crate::ix!();

/// Merges and writes all provided house‐number ranges into storage for the given region.
/// For each `(street, ranges)`, we load existing data from the DB, unify it with the new ranges,
/// and store the result. This function logs relevant steps and returns an error if
/// database operations fail.
///
/// # Arguments
///
/// * `house_number_ranges` - Map of `StreetName` to a list of new [`HouseNumberRange`] objects.
/// * `region` - The world region these house‐number ranges belong to.
/// * `db`     - Mutable reference to the `Database` to be updated.
///
/// # Returns
///
/// * `Ok(())` if all street data is successfully written.
/// * `Err(DatabaseConstructionError)` if a load or store operation fails.
pub fn write_house_number_ranges_into_storage(
    house_number_ranges: &HashMap<StreetName, Vec<HouseNumberRange>>,
    region: &WorldRegion,
    db: &mut Database,
) -> Result<(), DatabaseConstructionError> {
    trace!(
        "write_house_number_ranges_into_storage: storing house‐number data for region={:?}, streets_count={}",
        region,
        house_number_ranges.len()
    );

    for (street, new_ranges) in house_number_ranges {
        process_street_house_numbers(db, region, street, new_ranges)?;
    }

    info!("write_house_number_ranges_into_storage: done processing all streets for region={:?}", region);
    Ok(())
}
