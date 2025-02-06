// ---------------- [ File: src/load_existing_house_number_ranges.rs ]
crate::ix!();

pub trait LoadExistingHouseNumberRanges {

    fn load_existing_house_number_ranges(
        &self,
        region: &WorldRegion,
        street: &StreetName,
    ) -> Result<Vec<HouseNumberRange>, DataAccessError>;
}

impl LoadExistingHouseNumberRanges for Database {

    /// Loads existing house‐number ranges from the database for the specified street in the given region.
    fn load_existing_house_number_ranges(
        &self,
        region: &WorldRegion,
        street: &StreetName,
    ) -> Result<Vec<HouseNumberRange>, DataAccessError> {
        trace!(
            "load_existing_house_number_ranges: street='{}' in region={:?}",
            street,
            region
        );

        let existing_opt = self.load_house_number_ranges(region, street)?;
        let existing = existing_opt.unwrap_or_default();

        debug!(
            "load_existing_house_number_ranges: found {} existing ranges for street='{}'",
            existing.len(),
            street
        );
        Ok(existing)
    }
}
