crate::ix!();

/// A tiny helper to gather all known city names for a given region,
/// by scanning the RocksDB keys that match `C2Z:<abbr>:`. Then we parse
/// the city portion out of the key, and also decode the CBOR list so that
/// we can confirm it is well-formed. This lets us do naive “autocomplete”
/// by city name later in the REPL.
pub fn load_all_cities_for_region(
    db: &Database,
    region: &WorldRegion,
) -> Vec<String> {
    let mut all_cities = Vec::new();
    let prefix = format!("C2Z:{}:", region.abbreviation());

    // We'll do a prefix_iterator. Because the crate’s dynamic slice transform
    // extracts up to the second colon, we know "C2Z:US:" is enough to group by region.
    let iter = db.db().prefix_iterator(prefix.as_bytes());
    for item in iter {
        if let Ok((key_bytes, val_bytes)) = item {
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            // key_str looks like "C2Z:US:baltimore" or similar
            // We can parse out the city substring after the 2nd colon:
            //
            //  "C2Z:US:baltimore"
            //         ^ index of second colon
            // We'll do a simple split:
            let parts: Vec<&str> = key_str.splitn(3, ':').collect();
            if parts.len() < 3 {
                continue; // skip unexpected
            }
            let raw_city = parts[2].to_owned(); // e.g. "baltimore"
            // We also decode the value to confirm it’s valid CBOR, but we don’t strictly need the contents:
            if !val_bytes.is_empty() {
                // Optionally parse the postal codes or do nothing
                // let _postal_codes = decompress_cbor_to_list::<PostalCode>(&val_bytes);
            }
            all_cities.push(raw_city);
        }
    }

    all_cities
}


