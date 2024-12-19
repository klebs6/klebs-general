crate::ix!();

/// Collect tags into a HashMap for easier access
pub fn collect_tags(tags: TagIter<'_>) -> HashMap<String,String> {
    tags.map(|(k,v)| (k.to_string(), v.to_string())).collect()
}

/// Parse OSM PBF and return a list of address records
pub fn parse_osm_pbf(path: impl AsRef<Path>, country: &Country) 
    -> Result<Vec<AddressRecord>,OsmPbfParseError> 
{
    let reader = ElementReader::from_path(path)?;
    let mut records = Vec::new();
    let mut count = 0;

    reader.for_each(|element| {

        if let Ok(record) = AddressRecord::try_from((element,country)) {

            if count % 1000 == 0 { 
                info!("record for osm element, {:?}", record); 
            }

            records.push(record);
        }

        count += 1;

    })?;

    Ok(records)
}
