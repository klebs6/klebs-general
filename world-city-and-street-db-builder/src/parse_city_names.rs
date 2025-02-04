crate::ix!();

/// Parses city names from the `(key, value)` pairs. Extracts the city substring
/// after the second colon (`C2Z:US:baltimore => "baltimore"`). Additionally,
/// this step can decode the CBOR values for sanity checks, if desired.
pub fn parse_city_names(kv_pairs: Vec<(String, Vec<u8>)>) -> Vec<String> {
    trace!(
        "parse_city_names: parsing city names from {} key-value pairs",
        kv_pairs.len()
    );

    let mut cities = Vec::new();

    for (key_str, val_bytes) in kv_pairs {
        match extract_city_from_key(&key_str) {
            Some(city) => {
                debug!("parse_city_names: extracted city='{}' from key='{}'", city, key_str);
                // Optionally decode the CBOR to confirm validity:
                if let Err(e) = try_decode_postal_codes(&val_bytes) {
                    // We ignore the contents, but we can log an error if decoding fails
                    warn!(
                        "parse_city_names: postal code decoding failed for city='{}': {}",
                        city, e
                    );
                }
                cities.push(city);
            }
            None => {
                debug!(
                    "parse_city_names: skipping unexpected key='{}' (cannot parse city)",
                    key_str
                );
            }
        }
    }

    cities
}
