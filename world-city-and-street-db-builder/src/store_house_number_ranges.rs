// ---------------- [ File: src/store_house_number_ranges.rs ]
crate::ix!();

pub trait StoreHouseNumberRanges {
    fn store_house_number_ranges(
        &mut self,
        region: &WorldRegion,
        street: &StreetName,
        ranges: &[HouseNumberRange],
    ) -> Result<(), DatabaseConstructionError>;
}

impl StoreHouseNumberRanges for Database {

    /// Stores a set of house number sub-ranges for a given region/street into RocksDB.
    ///
    /// This overwrites any existing data for that region+street. 
    /// If you want to merge or append, load first, modify, then store again.
    fn store_house_number_ranges(
        &mut self,
        region: &WorldRegion,
        street: &StreetName,
        ranges: &[HouseNumberRange],
    ) -> Result<(), DatabaseConstructionError> {
        // 1) Key = "HNR:REGION_ABBR:street"
        let key = house_number_ranges_key(region, street);

        // 2) We'll store in CBOR. We can store it as a vector of HouseNumberRange 
        //    inside the standard CompressedList container or just directly. 
        //    For consistency with the rest of the code, let's store it in a CompressedList.
        let clist = crate::compressed_list::CompressedList::from(ranges.to_vec());
        let serialized = match serde_cbor::to_vec(&clist) {
            Ok(bytes) => bytes,
            Err(e) => {
                // Convert to OsmPbfParseError
                let msg = format!("Failed to serialize HouseNumberRanges for street '{}': {}", street.name(), e);
                return Err(OsmPbfParseError::HouseNumberRangeSerdeError { msg }.into());
            }
        };

        // 3) Put into RocksDB
        self.put(key.as_bytes(), serialized)?;
        Ok(())
    }
}
