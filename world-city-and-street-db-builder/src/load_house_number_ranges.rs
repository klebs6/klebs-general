// ---------------- [ File: src/load_house_number_ranges.rs ]
crate::ix!();

/// Loads the house number ranges (if any) for the specified region+street.
/// Returns `Ok(None)` if no data was stored for that key.
/// Returns `Ok(Some(vec![]))` if the key exists but is empty or if it 
/// legitimately has zero sub-ranges.
pub fn load_house_number_ranges(
    db:     &Database,
    region: &WorldRegion,
    street: &StreetName,

) -> Result<Option<Vec<HouseNumberRange>>, DatabaseConstructionError> {

    let key = house_number_ranges_key(region, street);
    let raw_opt = db.get(&key)?;

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
