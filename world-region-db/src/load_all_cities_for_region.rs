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
    trace!(
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
mod test_load_all_cities_for_region {
    use super::*;

    /// Stores CBOR-encoded data under the `C2Z:{region_abbr}:{city}` key.
    /// This helps simulate city->postal-code data in RocksDB, though we only
    /// need to confirm the city extraction logic for the key.
    /// The actual value can be valid or invalid CBOR for negative testing.
    fn put_c2z_data<I: StorageInterface>(
        db:        &mut I,
        region:    &WorldRegion,
        city_str:  &str,
        cbor_data: &[u8],
    ) {
        info!("Putting C2Z data for region {:?}, city '{}'", region, city_str);
        let key = format!("C2Z:{}:{}", region.abbreviation(), city_str);
        db.put(key, cbor_data).expect("Storing test data should succeed");
        debug!("Successfully stored test data for '{}'", city_str);
    }

    /// A helper to generate minimal (valid) CBOR for the sake of tests.
    /// We'll store some dummy "postal codes" or anything that decodes properly.
    /// The function returns a serialized `CompressedList<PostalCode>` with a single item.
    fn make_valid_cbor_for_test_postal() -> Vec<u8> {
        info!("Making valid CBOR for test postal data");
        let pc = PostalCode::new(Country::USA, "00000").unwrap();
        let clist = CompressedList::from(vec![pc]);
        serde_cbor::to_vec(&clist).expect("Serialization should succeed")
    }

    /// A helper that returns corrupted bytes for negative test cases.
    fn make_corrupted_cbor() -> Vec<u8> {
        info!("Making corrupted CBOR data for negative tests");
        b"not valid cbor data".to_vec()
    }

    #[traced_test]
    fn test_no_keys_in_db_returns_empty() {
        info!("Testing load_all_cities_for_region with empty DB (California)");
        let (db_arc, _td) = create_temp_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::California).into();
        let result = load_all_cities_for_region(&*db_guard, &region);
        assert!(result.is_empty(), "Expected an empty vector if no C2Z keys exist");
    }

    #[traced_test]
    fn test_no_c2z_keys_for_this_region_returns_empty() {
        info!("Testing load_all_cities_for_region with no matching C2Z prefix for California");
        let (db_arc, _td) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        // We'll store something unrelated, e.g. C2S or S2C, not matching "C2Z:CA:..."
        db_guard.put("S2C:US:main st", b"dummy cbor").unwrap();
        db_guard.put("C2Z:TX:dallas", b"dummy cbor").unwrap(); // different region abbr

        let region = USRegion::UnitedState(UnitedState::California).into();
        let result = load_all_cities_for_region(&*db_guard, &region);
        assert!(
            result.is_empty(),
            "No keys matching C2Z:CA: => should return empty"
        );
    }

    #[traced_test]
    fn test_single_city_extracted() {
        info!("Testing single city extraction for California");
        let (db_arc, _td) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::California).into();
        let valid_data = make_valid_cbor_for_test_postal();

        // e.g. "sunnyvale"
        put_c2z_data(&mut *db_guard, &region, "sunnyvale", &valid_data);

        let cities = load_all_cities_for_region(&*db_guard, &region);
        assert_eq!(cities.len(), 1, "Should find exactly one city");
        assert_eq!(
            cities[0],
            "sunnyvale",
            "City name should match the key substring"
        );
    }

    #[traced_test]
    fn test_multiple_cities_extracted() {
        info!("Testing multiple city extractions for California");
        let (db_arc, _td) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::California).into();
        let valid_data = make_valid_cbor_for_test_postal();

        put_c2z_data(&mut *db_guard, &region, "sunnyvale",     &valid_data);
        put_c2z_data(&mut *db_guard, &region, "santa_clara",   &valid_data);
        put_c2z_data(&mut *db_guard, &region, "palo_alto",     &valid_data);

        let mut cities = load_all_cities_for_region(&*db_guard, &region);
        cities.sort();

        assert_eq!(cities.len(), 3, "Expected three distinct city extractions");
        assert_eq!(cities, vec!["palo_alto", "santa_clara", "sunnyvale"]);
    }

    #[traced_test]
    fn test_malformed_key_skips_extraction() {
        info!("Testing malformed key scenario for California");
        let (db_arc, _td) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::California).into();
        let valid_data = make_valid_cbor_for_test_postal();

        // This key lacks a second colon, so it's not valid "C2Z:CA:city"
        db_guard.put("C2Z:CA", &valid_data).unwrap();

        // This key has only 2 parts but not 3, e.g. "C2Z:CA:"
        db_guard.put("C2Z:CA:", &valid_data).unwrap();

        // We'll put one valid key so there's at least one city in DB.
        put_c2z_data(&mut *db_guard, &region, "milpitas", &valid_data);

        let cities = load_all_cities_for_region(&*db_guard, &region);
        assert_eq!(
            cities,
            vec!["milpitas"],
            "Only the properly formed key should be extracted"
        );
    }

    #[traced_test]
    fn test_corrupted_cbor_still_extracts_city_name_but_logs_warning() {
        info!("Testing corrupted CBOR scenario for California");
        let (db_arc, _td) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::California).into();
        let corrupted_data = make_corrupted_cbor();

        // e.g. "sunnyvale"
        put_c2z_data(&mut *db_guard, &region, "sunnyvale", &corrupted_data);

        // We still see "sunnyvale" from the key, but the underlying CBOR decode fails.
        let cities = load_all_cities_for_region(&*db_guard, &region);
        assert_eq!(cities, vec!["sunnyvale"]);
    }

    #[traced_test]
    fn test_value_is_empty_bytes_is_still_accepted_but_ignored_for_postal_data() {
        info!("Testing empty value scenario for California");
        let (db_arc, _td) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::California).into();

        // e.g. "san_jose"
        put_c2z_data(&mut *db_guard, &region, "san_jose", &[]);

        let cities = load_all_cities_for_region(&*db_guard, &region);
        assert_eq!(
            cities,
            vec!["san_jose"],
            "City name is extracted from key, ignoring empty value"
        );
    }
}
