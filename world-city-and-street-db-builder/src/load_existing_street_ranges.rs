// ---------------- [ File: src/load_existing_street_ranges.rs ]
crate::ix!();

pub trait LoadExistingStreetRanges {

    fn load_existing_street_ranges(
        &self,
        world_region: &WorldRegion,
        street:       &StreetName,
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError>;
}

impl LoadExistingStreetRanges for Database {

    /// Loads existing house‐number ranges for the specified street from the DB.
    fn load_existing_street_ranges(
        &self,
        world_region: &WorldRegion,
        street:       &StreetName,
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
        trace!(
            "load_existing_street_ranges: loading for street='{}' in region={:?}",
            street,
            world_region
        );
        let existing = self.load_house_number_ranges(world_region, street)?;
        Ok(existing)
    }
}
