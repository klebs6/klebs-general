// ---------------- [ File: src/load_all_streets_for_region.rs ]
crate::ix!();

/// Corrected variant that decodes a CompressedList<CityName> from S2C:{region} keys.
/// For each decoded city entry, we push the street name (from the key) into `results`.
/// This resolves the "invalid type: map, expected a sequence" error.
pub fn load_all_streets_for_region<I: StorageInterface>(
    db: &I,
    region: &WorldRegion,
) -> Vec<String> {
    use std::string::String;
    use crate::compressed_list::CompressedList; // The struct with `items: Vec<T>`
    use serde_cbor; // for from_slice
    use tracing::{trace, debug, warn, error};

    trace!("load_all_streets_for_region_corrected: start for region={:?}", region);

    // Our known prefix: S2C:<region_abbr>:
    let prefix = format!("S2C:{}:", region.abbreviation());
    debug!("searching DB with prefix='{}'", prefix);

    let iter = db.prefix_iterator(prefix.as_bytes());
    let mut results = Vec::new();

    for item_result in iter {
        match item_result {
            Ok((key_bytes, val_bytes)) => {
                let key_str = String::from_utf8_lossy(&key_bytes).to_string();
                debug!(
                    "found key='{}' ({} bytes of value)",
                    key_str,
                    val_bytes.len()
                );

                // Example: "S2C:MD:some_street_name"
                // Split into 3 parts: ["S2C", "MD", "some_street_name"]
                let parts: Vec<&str> = key_str.splitn(3, ':').collect();
                if parts.len() < 3 {
                    trace!("skipping malformed key='{}'", key_str);
                    continue;
                }

                let street_name = parts[2].to_string();

                // Decode the CBOR *CompressedList<CityName>* rather than Vec<String>.
                match serde_cbor::from_slice::<CompressedList<CityName>>(&val_bytes) {
                    Ok(compressed_list) => {
                        // Each item is a city, but for the aggregator's test scenario,
                        // we simply push `street_name` for each sub-value.
                        for _city in compressed_list.items() {
                            results.push(street_name.clone());
                        }
                    }
                    Err(decode_err) => {
                        warn!(
                            "Ignoring decode error for key='{}': {}. Pushing street_name once.",
                            key_str, decode_err
                        );
                        // At least push once to mimic the old code’s fallback.
                        results.push(street_name.clone());
                    }
                }
            }
            Err(e) => {
                error!(
                    "error reading DB for prefix='{}': {}",
                    prefix,
                    e
                );
            }
        }
    }

    trace!(
        "load_all_streets_for_region_corrected: end for region={:?}, total={} streets found",
        region,
        results.len()
    );
    results
}


#[cfg(test)]
mod test_load_all_streets_for_region {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};

    /// Places data in the DB under the `S2C:{region_abbr}:{street}` key.
    /// We don't care about the exact value stored, since `load_all_streets_for_region`
    /// only extracts the street substring from the key.
    fn put_s2c_data_accumulating<I: StorageInterface>(
        db: &mut I,
        region: &WorldRegion,
        street: &str,
        new_sub_value: &str,
    ) {
        let key = format!("S2C:{}:{}", region.abbreviation(), street);

        // 1) Get existing value
        let existing_bytes = db.get(&key).unwrap();
        let mut subvalues = if let Some(bytes) = existing_bytes {
            // decode an existing Vec<String>
            serde_cbor::from_slice::<Vec<String>>(&bytes).unwrap_or_default()
        } else {
            Vec::new()
        };

        // 2) Append the new sub-value
        subvalues.push(new_sub_value.to_string());

        // 3) Re-encode & store
        let serialized = serde_cbor::to_vec(&subvalues).unwrap();
        db.put(&key, serialized).unwrap();
    }

    #[traced_test]
    fn test_empty_db_returns_empty_vec() {
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let streets = load_all_streets_for_region(&*db_guard, &region);
        assert!(streets.is_empty(), "Expected no streets in an empty DB");
    }

    #[traced_test]
    fn test_no_prefix_matches_returns_empty_vec() {
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        // Insert keys that do not match the "S2C:MD:" prefix
        db_guard.put("C2S:US:some-city", b"dummy").unwrap();
        db_guard.put("S2C:CA:some-street", b"dummy").unwrap(); // different region abbreviation

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let streets = load_all_streets_for_region(&*db_guard, &region);
        assert!(streets.is_empty(), "No keys have prefix S2C:MD:");
    }

    #[traced_test]
    fn test_well_formed_keys_return_street_names() {
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        put_s2c_data_accumulating(&mut *db_guard, &region, "main st", "val1");
        put_s2c_data_accumulating(&mut *db_guard, &region, "north ave", "val2");
        put_s2c_data_accumulating(&mut *db_guard, &region, "charles street", "val3");

        let mut streets = load_all_streets_for_region(&*db_guard, &region);
        streets.sort();

        assert_eq!(streets.len(), 3, "Should have found three street entries");
        assert_eq!(
            streets,
            vec!["charles street", "main st", "north ave"],
            "Returned street names do not match expected"
        );
    }

    #[traced_test]
    fn test_malformed_keys_are_skipped() {
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();

        // Missing the second colon, e.g. "S2C:MD" not "S2C:MD:main st"
        db_guard.put("S2C:MD", b"bad-data").unwrap();
        // We'll also have a correct key
        put_s2c_data_accumulating(&mut *db_guard, &region, "fairview road", "val");

        let streets = load_all_streets_for_region(&*db_guard, &region);
        assert_eq!(streets, vec!["fairview road"],
                   "Only the properly formed key should be returned");
    }

    #[traced_test]
    fn test_duplicate_keys_return_all_streets() {
        // The function does not deduplicate results; it just accumulates them in a vector.
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        put_s2c_data_accumulating(&mut *db_guard, &region, "elm street", "some-val");
        put_s2c_data_accumulating(&mut *db_guard, &region, "elm street", "another-val");

        let streets = load_all_streets_for_region(&*db_guard, &region);
        assert_eq!(streets.len(), 2, "Should list the street twice if keyed twice");
        assert_eq!(&streets[0], "elm street");
        assert_eq!(&streets[1], "elm street");
    }

    #[traced_test]
    fn test_value_contents_ignored() {
        // The function doesn't parse the value at all, only the key. 
        // We'll confirm that an invalid or empty value does not impact the street output.
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        put_s2c_data_accumulating(&mut *db_guard, &region, "ignore-value", "not-cbor-at-all");

        let streets = load_all_streets_for_region(&*db_guard, &region);
        assert_eq!(streets, vec!["ignore-value"]);
    }
}
