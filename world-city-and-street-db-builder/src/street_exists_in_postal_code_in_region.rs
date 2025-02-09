// ---------------- [ File: src/street_exists_in_postal_code_in_region.rs ]
crate::ix!();

pub trait StreetExistsInPostalCodeInRegion {

    fn street_exists_in_postal_code(
        &self, 
        region_name: &WorldRegion, 
        postal_code: &PostalCode, 
        street:      &StreetName
    ) -> bool;
}

impl<I:StorageInterface> StreetExistsInPostalCodeInRegion for DataAccess<I> {

    fn street_exists_in_postal_code(&self, region: &WorldRegion, postal_code: &PostalCode, street: &StreetName) -> bool {
        if let Some(sts) = self.street_names_for_postal_code_in_region(region, postal_code) {
            sts.contains(street)
        } else {
            false
        }
    }
}

#[cfg(test)]
#[disable]
mod test_street_exists_in_postal_code_in_region {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Creates a temporary database for testing, returning `(Arc<Mutex<Database>>, TempDir)`.
    /// The `TempDir` ensures that the directory remains valid for the duration of the tests.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Creates a [`DataAccess`] referencing the same database.
    fn create_data_access<I:StorageInterface>(db_arc: Arc<Mutex<I>>) -> DataAccess<I> {
        DataAccess::with_db(db_arc)
    }

    /// Helper to store a set of street names under the key returned by `s_key(region, postal_code)`.
    fn put_s_data<I: StorageInterface>(
        db:          &mut I,
        region:      &WorldRegion,
        postal_code: &PostalCode,
        streets:     &BTreeSet<StreetName>,
    ) {
        let key = s_key(region, postal_code);
        let val = compress_set_to_cbor(streets);
        db.put(&key, val).expect("Storing postalcode->streets data should succeed");
    }

    #[test]
    fn test_no_data_returns_false() {
        let (db_arc, _tmp) = create_temp_db();
        let db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let pc = PostalCode::new(Country::USA, "99999").unwrap();
        let street = StreetName::new("MissingStreet").unwrap();

        let result = data_access.street_exists_in_postal_code(&region, &pc, &street);
        assert!(!result, "No data => false");
    }

    #[test]
    fn test_street_in_set_returns_true() {
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let pc = PostalCode::new(Country::USA, "21201").unwrap();
        let street = StreetName::new("North Avenue").unwrap();

        // Insert the street in the set
        let mut streets = BTreeSet::new();
        streets.insert(street.clone());
        put_s_data(&mut db_guard, &region, &pc, &streets);

        // Query
        let result = data_access.street_exists_in_postal_code(&region, &pc, &street);
        assert!(result, "Street is in the stored set => true");
    }

    #[test]
    fn test_street_not_in_set_returns_false() {
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let pc = PostalCode::new(Country::USA, "21230").unwrap();

        let known = StreetName::new("KnownStreet").unwrap();
        let missing = StreetName::new("MissingStreet").unwrap();

        let mut streets = BTreeSet::new();
        streets.insert(known.clone());
        put_s_data(&mut db_guard, &region, &pc, &streets);

        // The missing street isn't in the set
        let result = data_access.street_exists_in_postal_code(&region, &pc, &missing);
        assert!(!result, "Should be false if street not in the set");
    }

    #[test]
    fn test_different_postal_code_returns_false() {
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();

        let pc1 = PostalCode::new(Country::USA, "21201").unwrap();
        let pc2 = PostalCode::new(Country::USA, "99999").unwrap();
        let street = StreetName::new("SharedStreet").unwrap();

        // Insert for pc1
        let mut streets = BTreeSet::new();
        streets.insert(street.clone());
        put_s_data(&mut db_guard, &region, &pc1, &streets);

        // Query for pc2 => no match
        let result = data_access.street_exists_in_postal_code(&region, &pc2, &street);
        assert!(!result, "Stored under pc1 => pc2 lookup => false");
    }

    #[test]
    fn test_different_region_returns_false() {
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region_md = WorldRegion::try_from_abbreviation("MD").unwrap();
        let region_va = WorldRegion::try_from_abbreviation("VA").unwrap();

        let pc = PostalCode::new(Country::USA, "21201").unwrap();
        let street = StreetName::new("RegionBoundStreet").unwrap();

        let mut streets = BTreeSet::new();
        streets.insert(street.clone());
        put_s_data(&mut db_guard, &region_md, &pc, &streets);

        // Query with region_va => not found
        let result = data_access.street_exists_in_postal_code(&region_va, &pc, &street);
        assert!(!result, "No data for region_va => false");
    }

    #[test]
    fn test_corrupted_cbor_returns_false() {
        // If the underlying data is invalid CBOR, 
        // `street_names_for_postal_code_in_region` returns None => false
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let pc = PostalCode::new(Country::USA, "21201").unwrap();
        let street = StreetName::new("BadStreet").unwrap();

        let key = s_key(&region, &pc);
        db_guard.put(key, b"not valid cbor").unwrap();

        let result = data_access.street_exists_in_postal_code(&region, &pc, &street);
        assert!(!result, "Corrupted data => decode fails => false");
    }
}
