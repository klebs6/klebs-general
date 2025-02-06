// ---------------- [ File: src/load_house_number_ranges.rs ]
crate::ix!();

pub trait LoadHouseNumberRanges {
    fn load_house_number_ranges(
        &self, 
        region: &WorldRegion, 
        street_obj: &StreetName
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError>;
}

impl LoadHouseNumberRanges for Database {

    // ----------------------------------------------------------------------
    // (C) Example method to load house-number ranges from DB or a stub
    // ----------------------------------------------------------------------
    //
    // This is purely illustrative. Adjust the signature or error handling 
    // as needed in your codebase.
    //
    fn load_house_number_ranges(
        &self, 
        region: &WorldRegion, 
        street_obj: &StreetName
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> 
    {
        let key = house_number_ranges_key(region, street_obj);
        let raw_opt = self.get(&key)?;

        match raw_opt {
            None => Ok(None), // no key => no data
            Some(bytes) => {
                // Attempt to decode from CBOR -> CompressedList<HouseNumberRange>
                let clist_result: Result<crate::compressed_list::CompressedList<HouseNumberRange>, _> =
                    serde_cbor::from_slice(&bytes);

                match clist_result {
                    Ok(clist) => {
                        let items = clist.items().clone();
                        Ok(Some(items))
                    }
                    Err(e) => {
                        let msg = format!(
                            "Failed to deserialize HouseNumberRanges for '{}': {}",
                            key, e
                        );
                        // Convert to OsmPbfParseError
                        Err(OsmPbfParseError::HouseNumberRangeSerdeError { msg }.into())
                    }
                }
            }
        }
    }
}
