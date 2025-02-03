// ---------------- [ File: src/house_number_in_any_range.rs ]
crate::ix!();

/// Utility function to check if a given house number is contained
/// in any of the sub-ranges for a region+street.
pub fn house_number_in_any_range(
    db: &Database,
    region: &WorldRegion,
    street: &StreetName,
    house_num: u32,
) -> Result<bool, DatabaseConstructionError> {
    if let Some(ranges) = load_house_number_ranges(db, region, street)? {
        for rng in ranges {
            if rng.contains(house_num) {
                return Ok(true);
            }
        }
        Ok(false)
    } else {
        // No entry found, so presumably no coverage
        Ok(false)
    }
}
