// ---------------- [ File: src/house_number_in_any_range.rs ]
crate::ix!();

pub trait HouseNumberInAnyRange {

    fn house_number_in_any_range(
        &self,
        region:      &WorldRegion,
        street:      &StreetName,
        house_num:   u32,
    ) -> Result<bool, DataAccessError>;
}

impl HouseNumberInAnyRange for Database {

    /// Utility function to check if a given house number is contained
    /// in any of the sub-ranges for a region+street.
    fn house_number_in_any_range(
        &self,
        region:      &WorldRegion,
        street:      &StreetName,
        house_num:   u32,
    ) -> Result<bool, DataAccessError> {

        if let Some(ranges) = self.load_house_number_ranges(region, street)? {
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
}
