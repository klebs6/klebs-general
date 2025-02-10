// ---------------- [ File: src/street_names_for_postal_code_in_region.rs ]
// ---------------- [ File: src/street_names_for_postal_code_in_region.rs ]
crate::ix!();

pub trait StreetNamesForPostalCodeInRegion {

    fn street_names_for_postal_code_in_region(
        &self, 
        region_name: &WorldRegion, 
        postal_code: &PostalCode
    ) -> Option<BTreeSet<StreetName>>;
}


impl<I:StorageInterface> StreetNamesForPostalCodeInRegion for DataAccess<I> {

    fn street_names_for_postal_code_in_region(
        &self, 
        region: &WorldRegion, 
        postal_code:    &PostalCode

    ) -> Option<BTreeSet<StreetName>> {

        let key = s_key(region,postal_code);
        self.get_street_set(&key)
    }
}

#[cfg(test)]
#[disable]
mod test_street_names_for_postal_code_in_region {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Creates a temporary database and returns `(Arc<Mutex<Database>>, TempDir)`.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Builds a `DataAccess` reference that points to the same Database.
    fn create_data_access<I:StorageInterface>(db_arc: Arc<Mutex<I>>) -> DataAccess {
        DataAccess::with_db(db_arc)
    }

    /// Helper that stores a set of streets under the key that `s_key(region, postal_code)` generates.
    /// This simulates what the code normally does at ingest time.
    fn put_s_data<I:StorageInterface>(
        db: &mut I,
        region: &WorldRegion,
        postal_code: &PostalCode,
        streets: &BTreeSet<StreetName>,
    ) {
        let key = s_key(region, postal_code);
        let val = compress_set_to_cbor(streets);
        db.put(key, val).unwrap();
    }

    #[traced_test]
    fn test_no_data_returns_none() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "00000").unwrap();

        // No data inserted => should be None
        let result = data_access.street_names_for_postal_code_in_region(&region, &postal_code);
        assert!(
            result.is_none(),
            "Expected None when no data is stored for the region/postal_code"
        );
    }

    #[traced_test]
    fn test_existing_data_returns_btreeset() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "21201").unwrap();

        // Insert multiple streets
        let mut streets = BTreeSet::new();
        streets.insert(StreetName::new("North Ave").unwrap());
        streets.insert(StreetName::new("Howard St").unwrap());
        streets.insert(StreetName::new("Eutaw St").unwrap());

        // Store
        put_s_data(&mut db_guard, &region, &postal_code, &streets);

        // Now query
        let result = data_access.street_names_for_postal_code_in_region(&region, &postal_code);
        assert!(result.is_some(), "Should return Some(...) for stored data");
        let retrieved = result.unwrap();
        assert_eq!(
            retrieved, streets,
            "Retrieved set should match the stored street names"
        );
    }

    #[traced_test]
    fn test_corrupted_cbor_returns_none() {
        // If the underlying CBOR is invalid, we get None.
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "21201").unwrap();
        let key = s_key(&region, &postal_code);

        // Insert invalid data
        db_guard.put(key, b"invalid cbor").unwrap();

        let result = data_access.street_names_for_postal_code_in_region(&region, &postal_code);
        assert!(result.is_none(), "Corrupted data => None");
    }

    #[traced_test]
    fn test_different_region_or_postal_code_returns_none() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region_md = WorldRegion::try_from_abbreviation("MD").unwrap();
        let region_va = WorldRegion::try_from_abbreviation("VA").unwrap();

        let pc_21201 = PostalCode::new(Country::USA, "21201").unwrap();
        let pc_99999 = PostalCode::new(Country::USA, "99999").unwrap();

        let mut streets_md = BTreeSet::new();
        streets_md.insert(StreetName::new("North Ave").unwrap());
        put_s_data(&mut db_guard, &region_md, &pc_21201, &streets_md);

        // If we query region_va instead => no data
        let result_va = data_access.street_names_for_postal_code_in_region(&region_va, &pc_21201);
        assert!(result_va.is_none(), "Different region => no data => None");

        // If we query pc_99999 => no data
        let result_99999 = data_access.street_names_for_postal_code_in_region(&region_md, &pc_99999);
        assert!(result_99999.is_none(), "Different postal code => no data => None");
    }

    #[traced_test]
    fn test_duplicate_street_names_stored_once() {
        // If the DB store has duplicates in the BTreeSet, they unify. 
        // We'll store them and confirm only one instance persists.
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "21000").unwrap();

        let street_1 = StreetName::new("DuplicateStreet").unwrap();
        let street_2 = StreetName::new("DuplicateStreet").unwrap();

        let mut streets = BTreeSet::new();
        streets.insert(street_1);
        streets.insert(street_2); // same name => no effect
        put_s_data(&mut db_guard, &region, &postal_code, &streets);

        let retrieved_opt = data_access.street_names_for_postal_code_in_region(&region, &postal_code);
        assert!(retrieved_opt.is_some(), "We inserted data => Some(...)");
        let retrieved = retrieved_opt.unwrap();
        assert_eq!(retrieved.len(), 1, "Duplicate streets => single entry in BTreeSet");
        assert_eq!(retrieved.iter().next().unwrap().name(), "duplicatestreet");
    }
}
