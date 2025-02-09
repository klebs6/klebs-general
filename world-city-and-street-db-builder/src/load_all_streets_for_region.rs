// ---------------- [ File: src/load_all_streets_for_region.rs ]
crate::ix!();

/// Similarly for street autocomplete. We gather all known street names for region
/// by scanning prefix "S2C:<abbr>:", parse out the “street” portion from the key.
pub fn load_all_streets_for_region(
    db: &Database,
    region: &WorldRegion,
) -> Vec<String> {
    let mut all_streets = Vec::new();
    let prefix = format!("S2C:{}:", region.abbreviation());

    let iter = db.db().prefix_iterator(prefix.as_bytes());
    for item in iter {
        if let Ok((key_bytes, val_bytes)) = item {
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            // e.g. "S2C:US:north avenue"
            // splitn(3, ':') => ["S2C", "US", "north avenue"]
            let parts: Vec<&str> = key_str.splitn(3, ':').collect();
            if parts.len() < 3 {
                continue;
            }
            let raw_street = parts[2].to_owned();
            // Optionally parse or ignore val_bytes
            all_streets.push(raw_street);
        }
    }

    all_streets
}

#[cfg(test)]
mod test_load_all_streets_for_region {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};

    /// Creates a temporary database for testing and returns `(db, temp_dir)`.
    /// The temp directory ensures the DB files exist only for the test duration.
    fn create_temp_db() -> (Arc<Mutex<Database>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db = Database::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Places data in the DB under the `S2C:{region_abbr}:{street}` key.
    /// We don't care about the exact value stored, since `load_all_streets_for_region`
    /// only extracts the street substring from the key.
    fn put_s2c_data(db: &mut Database, region: &WorldRegion, street: &str, value: &[u8]) {
        let key = format!("S2C:{}:{}", region.abbreviation(), street);
        db.put(&key, value).expect("Failed to insert test data");
    }

    #[test]
    fn test_empty_db_returns_empty_vec() {
        let (db_arc, _temp_dir) = create_temp_db();
        let db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let streets = load_all_streets_for_region(&db_guard, &region);
        assert!(streets.is_empty(), "Expected no streets in an empty DB");
    }

    #[test]
    fn test_no_prefix_matches_returns_empty_vec() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        // Insert keys that do not match the "S2C:MD:" prefix
        db_guard.put("C2S:US:some-city", b"dummy").unwrap();
        db_guard.put("S2C:CA:some-street", b"dummy").unwrap(); // different region abbreviation

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let streets = load_all_streets_for_region(&db_guard, &region);
        assert!(streets.is_empty(), "No keys have prefix S2C:MD:");
    }

    #[test]
    fn test_well_formed_keys_return_street_names() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        put_s2c_data(&mut db_guard, &region, "main st", b"val1");
        put_s2c_data(&mut db_guard, &region, "north ave", b"val2");
        put_s2c_data(&mut db_guard, &region, "charles street", b"val3");

        let mut streets = load_all_streets_for_region(&db_guard, &region);
        streets.sort();

        assert_eq!(streets.len(), 3, "Should have found three street entries");
        assert_eq!(
            streets,
            vec!["charles street", "main st", "north ave"],
            "Returned street names do not match expected"
        );
    }

    #[test]
    fn test_malformed_keys_are_skipped() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();

        // Missing the second colon, e.g. "S2C:MD" not "S2C:MD:main st"
        db_guard.put("S2C:MD", b"bad-data").unwrap();
        // We'll also have a correct key
        put_s2c_data(&mut db_guard, &region, "fairview road", b"val");

        let streets = load_all_streets_for_region(&db_guard, &region);
        assert_eq!(streets, vec!["fairview road"],
                   "Only the properly formed key should be returned");
    }

    #[test]
    fn test_duplicate_keys_return_all_streets() {
        // The function does not deduplicate results; it just accumulates them in a vector.
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        put_s2c_data(&mut db_guard, &region, "elm street", b"some-val");
        put_s2c_data(&mut db_guard, &region, "elm street", b"another-val");

        let streets = load_all_streets_for_region(&db_guard, &region);
        assert_eq!(streets.len(), 2, "Should list the street twice if keyed twice");
        assert_eq!(&streets[0], "elm street");
        assert_eq!(&streets[1], "elm street");
    }

    #[test]
    fn test_value_contents_ignored() {
        // The function doesn't parse the value at all, only the key. 
        // We'll confirm that an invalid or empty value does not impact the street output.
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        put_s2c_data(&mut db_guard, &region, "ignore-value", b"not-cbor-at-all");

        let streets = load_all_streets_for_region(&db_guard, &region);
        assert_eq!(streets, vec!["ignore-value"]);
    }
}
