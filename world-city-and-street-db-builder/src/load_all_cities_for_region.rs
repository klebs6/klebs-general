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
pub fn load_all_cities_for_region<I:StorageInterface>(db: &I, region: &WorldRegion) -> Vec<String> {
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

#[cfg(test)]
#[disable]
mod test_load_all_cities_for_region {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc,Mutex};

    /// Creates a temporary database for testing.
    /// Returns `(Arc<Mutex<Database>>, TempDir)` so that the temp directory
    /// remains valid for the lifetime of the tests.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(tmp.path()).expect("Failed to open database in temp dir");
        (db, tmp)
    }

    /// Stores CBOR-encoded data under the `C2Z:{region_abbr}:{city}` key.
    /// This helps simulate city->postal-code data in RocksDB, though we only
    /// need to confirm the city extraction logic for the key.
    /// The actual value can be valid or invalid CBOR for negative testing.
    fn put_c2z_data<I:StorageInterface>(
        db:        &mut I,
        region:    &WorldRegion,
        city_str:  &str,
        cbor_data: &[u8],
    ) {
        let key = format!("C2Z:{}:{}", region.abbreviation(), city_str);
        db.put(key, cbor_data).expect("Storing test data should succeed");
    }

    /// A helper to generate minimal (valid) CBOR for the sake of tests.
    /// We'll store some dummy "postal codes" or anything that decodes properly.
    /// The function returns a serialized `CompressedList<PostalCode>` with a single item.
    fn make_valid_cbor_for_test_postal() -> Vec<u8> {
        let pc = PostalCode::new(Country::USA, "00000").unwrap();
        let clist = CompressedList::from(vec![pc]);
        serde_cbor::to_vec(&clist).expect("Serialization should succeed")
    }

    /// A helper that returns corrupted bytes for negative test cases.
    fn make_corrupted_cbor() -> Vec<u8> {
        b"not valid cbor data".to_vec()
    }

    #[test]
    fn test_no_keys_in_db_returns_empty() {
        let (db_arc, _td) = create_temp_db();
        let db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let result = load_all_cities_for_region(&db_guard, &region);
        assert!(result.is_empty(), "Expected an empty vector if no C2Z keys exist");
    }

    #[test]
    fn test_no_c2z_keys_for_this_region_returns_empty() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        // We'll store something unrelated, e.g. C2S or S2C, not matching the prefix "C2Z:<abbr>:"
        db_guard.put("S2C:US:main st", b"dummy cbor").unwrap();
        db_guard.put("C2Z:CA:toronto", b"dummy cbor").unwrap(); // different region abbr

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let result = load_all_cities_for_region(&db_guard, &region);
        assert!(result.is_empty(), "No keys matching C2Z:MD: => should return empty");
    }

    #[test]
    fn test_single_city_extracted() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let valid_data = make_valid_cbor_for_test_postal();

        put_c2z_data(&mut db_guard, &region, "baltimore", &valid_data);

        let cities = load_all_cities_for_region(&db_guard, &region);
        assert_eq!(cities.len(), 1, "Should find exactly one city");
        assert_eq!(cities[0], "baltimore", "City name should match the key substring");
    }

    #[test]
    fn test_multiple_cities_extracted() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let valid_data = make_valid_cbor_for_test_postal();

        put_c2z_data(&mut db_guard, &region, "baltimore", &valid_data);
        put_c2z_data(&mut db_guard, &region, "frederick", &valid_data);
        put_c2z_data(&mut db_guard, &region, "annapolis", &valid_data);

        let mut cities = load_all_cities_for_region(&db_guard, &region);
        cities.sort();

        assert_eq!(cities.len(), 3, "Expected three distinct city extractions");
        assert_eq!(cities, vec!["annapolis", "baltimore", "frederick"]);
    }

    #[test]
    fn test_malformed_key_skips_extraction() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let valid_data = make_valid_cbor_for_test_postal();

        // This key lacks a second colon, so it's not valid "C2Z:MD:city"
        db_guard.put("C2Z:MD", &valid_data).unwrap();

        // This key has only 2 parts but not 3, e.g. "C2Z:MD:"
        db_guard.put("C2Z:MD:", &valid_data).unwrap();

        // We'll put one valid key so there's at least one city in DB.
        put_c2z_data(&mut db_guard, &region, "rockville", &valid_data);

        let cities = load_all_cities_for_region(&db_guard, &region);
        assert_eq!(
            cities,
            vec!["rockville"],
            "Only the properly formed key should be extracted"
        );
    }

    #[test]
    fn test_corrupted_cbor_still_extracts_city_name_but_logs_warning() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let corrupted_data = make_corrupted_cbor();

        put_c2z_data(&mut db_guard, &region, "baltimore", &corrupted_data);

        // The city name "baltimore" will appear from the key, 
        // but decoding the CBOR is expected to fail. We only discard that data, 
        // but we still capture the city name from the key substring.
        let cities = load_all_cities_for_region(&db_guard, &region);
        assert_eq!(cities, vec!["baltimore"]);
    }

    #[test]
    fn test_value_is_empty_bytes_is_still_accepted_but_ignored_for_postal_data() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        // Storing an empty value
        put_c2z_data(&mut db_guard, &region, "silver_spring", &[]);

        let cities = load_all_cities_for_region(&db_guard, &region);
        assert_eq!(cities, vec!["silver_spring"], "City name is extracted from key, ignoring empty value");
    }
}
