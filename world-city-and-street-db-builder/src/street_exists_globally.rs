// ---------------- [ File: src/street_exists_globally.rs ]
// ---------------- [ File: src/street_exists_globally.rs ]
crate::ix!();

pub trait StreetExistsGlobally {

    fn street_exists_globally(
        &self, 
        region_name: &WorldRegion, 
        street:      &StreetName
    ) -> bool;
}

impl<I:StorageInterface> StreetExistsGlobally for DataAccess<I> {

    // street_exists_globally in a region:
    fn street_exists_globally(&self, region: &WorldRegion, street: &StreetName) -> bool {

        // If S2C or S2Z keys exist for this street, it's known:
        let key_cities       = s2c_key(region,street);
        let key_postal_codes = s2z_key(region,street);

        self.get_city_set(&key_cities).is_some() || self.get_postal_code_set(&key_postal_codes).is_some()
    }
}

#[cfg(test)]
#[disable]
mod test_street_exists_globally {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::collections::BTreeSet;
    use tempfile::TempDir;

    /// Creates a temporary database for testing, returning `(Arc<Mutex<Database>>, TempDir)`.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Creates a [`DataAccess`] from the same database reference.
    fn create_data_access<I:StorageInterface>(db_arc: Arc<Mutex<I>>) -> DataAccess {
        DataAccess::with_db(db_arc)
    }

    /// Inserts a set of cities under the `S2C:{region_abbr}:{street}` key.
    fn put_s2c_data<I:StorageInterface>(
        db:     &mut I, 
        region: &WorldRegion, 
        street: &StreetName, 
        cities: &BTreeSet<CityName>
    ) {
        let key = s2c_key(region, street);
        let val = compress_set_to_cbor(cities);
        db.put(&key, val).unwrap();
    }

    /// Inserts a set of postal codes under the `S2Z:{region_abbr}:{street}` key.
    fn put_s2z_data<I:StorageInterface>(
        db:           &mut I, 
        region:       &WorldRegion, 
        street:       &StreetName, 
        postal_codes: &BTreeSet<PostalCode>
    ) {
        let key = s2z_key(region, street);
        let val = compress_set_to_cbor(postal_codes);
        db.put(&key, val).unwrap();
    }

    #[traced_test]
    fn test_no_data_returns_false() {
        let (db_arc, _td) = create_temp_db();
        let db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Unknown Street").unwrap();

        let result = data_access.street_exists_globally(&region, &street);
        assert!(!result, "No data => street should not exist");
    }

    #[traced_test]
    fn test_s2c_key_exists_returns_true() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Main St").unwrap();

        // Insert some city set for S2C
        let mut city_set = BTreeSet::new();
        city_set.insert(CityName::new("Baltimore").unwrap());
        put_s2c_data(&mut db_guard, &region, &street, &city_set);

        let result = data_access.street_exists_globally(&region, &street);
        assert!(result, "S2C key is present => street should exist");
    }

    #[traced_test]
    fn test_s2z_key_exists_returns_true() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let street = StreetName::new("Another St").unwrap();

        // Insert some postal code set for S2Z
        let mut postal_codes = BTreeSet::new();
        postal_codes.insert(PostalCode::new(Country::USA, "20190").unwrap());
        put_s2z_data(&mut db_guard, &region, &street, &postal_codes);

        let result = data_access.street_exists_globally(&region, &street);
        assert!(result, "S2Z key is present => street should exist");
    }

    #[traced_test]
    fn test_both_keys_exist_also_returns_true() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let street = StreetName::new("Pennsylvania Ave").unwrap();

        // Insert city under S2C
        let mut city_set = BTreeSet::new();
        city_set.insert(CityName::new("Washington").unwrap());
        put_s2c_data(&mut db_guard, &region, &street, &city_set);

        // Insert postal under S2Z
        let mut postal_codes = BTreeSet::new();
        postal_codes.insert(PostalCode::new(Country::USA, "20001").unwrap());
        put_s2z_data(&mut db_guard, &region, &street, &postal_codes);

        let result = data_access.street_exists_globally(&region, &street);
        assert!(result, "Having either key is enough => definitely true if both exist");
    }

    #[traced_test]
    fn test_corrupted_cbor_treated_as_none() {
        // If the CBOR is invalid, `get_city_set` or `get_postal_code_set` returns None => street_exists_globally => false
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Corrupted St").unwrap();

        // Write invalid bytes to s2c key
        let s2c = s2c_key(&region, &street);
        db_guard.put(&s2c, b"not valid cbor").unwrap();

        let result = data_access.street_exists_globally(&region, &street);
        assert!(!result, "Corrupted cbor => decode fails => None => false");
    }

    #[traced_test]
    fn test_region_is_mismatched() {
        // If we store S2C for region=MD but query region=VA, it returns false
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region_md = WorldRegion::try_from_abbreviation("MD").unwrap();
        let region_va = WorldRegion::try_from_abbreviation("VA").unwrap();
        let street = StreetName::new("Boundary Street").unwrap();

        let mut city_set = BTreeSet::new();
        city_set.insert(CityName::new("SomeCity").unwrap());
        put_s2c_data(&mut db_guard, &region_md, &street, &city_set);

        // Query with region=VA => no match
        let result = data_access.street_exists_globally(&region_va, &street);
        assert!(!result, "Wrong region => no key => false");
    }
}
