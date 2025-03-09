crate::ix!();

/// A helper function to load streets for a region from region->postal_code->streets keys.
/// This ensures we pick up “street+zip but no city” cases.
pub fn load_extra_streets_from_zip_prefix<I: StorageInterface>(
    db: &I,
    region: &WorldRegion
) -> Vec<String> {
    let mut results = Vec::new();

    // The prefix we use to store zip→streets is typically `S:{region}:{zip}` 
    // or region_postal_code_streets => "S2Z:REGION_ABBR:..."
    let prefix = format!("S:{}:", region.abbreviation());
    trace!("load_extra_streets_from_zip_prefix: prefix='{}'", prefix);

    let iter = db.prefix_iterator(prefix.as_bytes());
    for item_res in iter {
        match item_res {
            Ok((key_bytes, val_bytes)) => {
                let key_str = String::from_utf8_lossy(&key_bytes).to_string();
                // Example key => "S:VA:20124"
                let parts: Vec<&str> = key_str.splitn(3, ':').collect();
                if parts.len() < 3 {
                    continue;
                }
                // val_bytes => CBOR of StreetName set
                let street_list = crate::compressed_list::decompress_cbor_to_list::<StreetName>(&val_bytes);
                // We convert each StreetName into a string
                for sname in street_list {
                    results.push(sname.name().to_string());
                }
            }
            Err(e) => {
                error!("load_extra_streets_from_zip_prefix: error reading DB: {}", e);
            }
        }
    }

    results
}
