// ---------------- [ File: src/load_all_cities_for_region.rs ]
crate::ix!();

/// A tiny helper to gather all known city names for a given region.
/// Internally, it searches RocksDB for keys with the prefix `C2Z:<abbr>:`
/// and extracts the city substring after the second colon. It also decodes
/// CBOR values to confirm they're valid, though we discard the parsed data
/// by default.
///
/// # Arguments
///
/// * `db`     - The database reference used for iteration.
/// * `region` - The region whose city names we want to gather.
///
/// # Returns
///
/// * A vector of city names (e.g., `["baltimore", "frederick", ...]`).
pub fn load_all_cities_for_region(db: &Database, region: &WorldRegion) -> Vec<String> {
    trace!("load_all_cities_for_region: start for region={:?}", region);

    let prefix = build_city_search_prefix(region);
    debug!(
        "load_all_cities_for_region: searching DB with prefix='{}'",
        prefix
    );

    // 1) Collect all (key, value) pairs matching "C2Z:<abbr>:".
    let kv_pairs = gather_city_key_value_pairs(db, &prefix);

    // 2) Parse city names from these pairs, optionally decoding CBOR to confirm validity.
    let all_cities = parse_city_names(kv_pairs);

    debug!(
        "load_all_cities_for_region: found {} cities for region={:?}",
        all_cities.len(),
        region
    );
    all_cities
}

/// Constructs the RocksDB key prefix for city => postal code data.
fn build_city_search_prefix(region: &WorldRegion) -> String {
    trace!("build_city_search_prefix: building prefix for region={:?}", region);
    format!("C2Z:{}:", region.abbreviation())
}

/// Performs a prefix-based iteration in RocksDB to find all city keys matching the prefix.
/// Returns a vector of `(key_string, value_bytes)` tuples for further processing.
fn gather_city_key_value_pairs(db: &Database, prefix: &str) -> Vec<(String, Vec<u8>)> {
    trace!(
        "gather_city_key_value_pairs: prefix='{}' => running prefix_iterator",
        prefix
    );

    let iter = db.db().prefix_iterator(prefix.as_bytes());
    let mut results = Vec::new();

    for item_result in iter {
        match item_result {
            Ok((key_bytes, val_bytes)) => {
                let key_str = String::from_utf8_lossy(&key_bytes).to_string();
                debug!(
                    "gather_city_key_value_pairs: found key='{}' (value: {} bytes)",
                    key_str,
                    val_bytes.len()
                );
                results.push((key_str, val_bytes.to_vec()));
            }
            Err(e) => {
                error!(
                    "gather_city_key_value_pairs: error reading from DB for prefix='{}': {}",
                    prefix, e
                );
            }
        }
    }

    results
}

/// Parses city names from the `(key, value)` pairs. Extracts the city substring
/// after the second colon (`C2Z:US:baltimore => "baltimore"`). Additionally,
/// this step can decode the CBOR values for sanity checks, if desired.
fn parse_city_names(kv_pairs: Vec<(String, Vec<u8>)>) -> Vec<String> {
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

/// Attempts to extract the city portion from a RocksDB key of the form:
/// `C2Z:<region_abbreviation>:<city>`.
///
/// Returns `Some(city_string)` if successful, or `None` if the key is malformed.
fn extract_city_from_key(key_str: &str) -> Option<String> {
    trace!("extract_city_from_key: analyzing key='{}'", key_str);

    // `splitn(3, ':')` -> e.g. ["C2Z", "US", "baltimore"]
    let parts: Vec<&str> = key_str.splitn(3, ':').collect();
    if parts.len() < 3 {
        warn!(
            "extract_city_from_key: key='{}' does not contain 3 parts; ignoring",
            key_str
        );
        return None;
    }
    Some(parts[2].to_owned())
}

/// Illustrates a hypothetical decode of postal codes from the RocksDB value bytes.
/// Currently, this just checks if the value is valid CBOR without storing or returning
/// the data. This can be extended to parse a `CompressedList<PostalCode>` if needed.
fn try_decode_postal_codes(val_bytes: &[u8]) -> Result<(), String> {
    trace!("try_decode_postal_codes: attempting decode of {} bytes", val_bytes.len());
    if val_bytes.is_empty() {
        debug!("try_decode_postal_codes: empty value => ignoring");
        return Ok(());
    }

    // Example: We'll pretend to decode, ignoring the actual type for demonstration.
    match serde_cbor::from_slice::<serde_cbor::Value>(val_bytes) {
        Ok(_) => {
            debug!("try_decode_postal_codes: successfully decoded CBOR data");
            Ok(())
        }
        Err(e) => {
            // Return an error string, or a specialized error type in real code
            Err(format!("CBOR decode error: {:?}", e))
        }
    }
}
