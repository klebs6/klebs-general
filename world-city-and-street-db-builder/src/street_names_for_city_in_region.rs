// ---------------- [ File: src/street_names_for_city_in_region.rs ]
crate::ix!();

pub trait StreetNamesForCityInRegion {

    fn street_names_for_city_in_region(
        &self, 
        region_name: &WorldRegion, 
        city:        &CityName

    ) -> Option<BTreeSet<StreetName>>;
}

impl<I:StorageInterface> StreetNamesForCityInRegion for DataAccess<I> {

    // Similarly for other queries:
    fn street_names_for_city_in_region(&self, region: &WorldRegion, city: &CityName) -> Option<BTreeSet<StreetName>> {
        let key = c2s_key(region,city);
        self.get_street_set(&key)
    }
}

#[cfg(test)]
#[disable]
mod test_street_names_for_city_in_region {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Creates a temporary database for testing, returning `(Arc<Mutex<Database>>, TempDir)`.
    /// The `TempDir` ensures that the directory remains valid for the duration of the tests.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Creates a [`DataAccess`] referencing the same `Database` so we can call the method under test.
    fn create_data_access<I:StorageInterface>(db_arc: Arc<Mutex<I>>) -> DataAccess {
        DataAccess::with_db(db_arc)
    }

    /// Inserts a set of streets under the RocksDB key `C2S:{region_abbr}:{city}`.
    /// This helps simulate the stored data that `street_names_for_city_in_region` should retrieve.
    fn put_c2s_data<I:StorageInterface>(
        db:      &mut I,
        region:  &WorldRegion,
        city:    &CityName,
        streets: &BTreeSet<StreetName>
    ) {
        let key = c2s_key(region, city);
        let val = compress_set_to_cbor(streets);
        db.put(key, val).unwrap();
    }

    #[traced_test]
    fn test_no_data_returns_none() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("NoDataCity").unwrap();

        // We haven't inserted anything for this city => should return None
        let result = data_access.street_names_for_city_in_region(&region, &city);
        assert!(result.is_none(), "No data => None");
    }

    #[traced_test]
    fn test_existing_data_returns_btreeset() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("Baltimore").unwrap();

        // We'll store multiple streets in a BTreeSet
        let mut streets = BTreeSet::new();
        streets.insert(StreetName::new("North Avenue").unwrap());
        streets.insert(StreetName::new("Howard Street").unwrap());
        streets.insert(StreetName::new("Pratt Street").unwrap());

        put_c2s_data(&mut db_guard, &region, &city, &streets);

        // Now retrieve
        let result = data_access.street_names_for_city_in_region(&region, &city);
        assert!(result.is_some(), "We have data => Some(...)");
        let retrieved = result.unwrap();
        assert_eq!(retrieved, streets, "Should match the stored set exactly");
    }

    #[traced_test]
    fn test_corrupted_cbor_returns_none() {
        // If the CBOR is corrupted, the code returns None instead of Some(...)
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("Glitchville").unwrap();

        // Write invalid bytes
        let key = c2s_key(&region, &city);
        db_guard.put(key, b"not valid cbor").unwrap();

        // Attempt to read => should fail decode => None
        let result = data_access.street_names_for_city_in_region(&region, &city);
        assert!(result.is_none(), "Corrupted data => None");
    }

    #[traced_test]
    fn test_different_city_returns_none() {
        // If we only store data for city1 but query city2, we get None
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city_stored = CityName::new("Frederick").unwrap();
        let city_missing = CityName::new("Hagerstown").unwrap();

        // Insert for Frederick
        let mut streets = BTreeSet::new();
        streets.insert(StreetName::new("Market Street").unwrap());
        put_c2s_data(&mut db_guard, &region, &city_stored, &streets);

        // Query for Hagerstown => None
        let result = data_access.street_names_for_city_in_region(&region, &city_missing);
        assert!(result.is_none(), "Should not find data for a different city");
    }

    #[traced_test]
    fn test_different_region_returns_none() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region_md = WorldRegion::try_from_abbreviation("MD").unwrap();
        let region_va = WorldRegion::try_from_abbreviation("VA").unwrap();

        let city = CityName::new("BorderTown").unwrap();

        // Store for region_md
        let mut streets = BTreeSet::new();
        streets.insert(StreetName::new("CrossLine").unwrap());
        put_c2s_data(&mut db_guard, &region_md, &city, &streets);

        // Query for region_va => should be none
        let result = data_access.street_names_for_city_in_region(&region_va, &city);
        assert!(result.is_none(), "No data for a different region => None");
    }

    #[traced_test]
    fn test_duplicate_streets_still_in_btreeset() {
        // If we stored duplicate streets somehow, the BTreeSet should unify them,
        // but let's confirm the final result is still correct.
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("DupCity").unwrap();

        // Insert duplicates
        let street_dup1 = StreetName::new("SameStreet").unwrap();
        let street_dup2 = StreetName::new("SameStreet").unwrap();

        let mut streets = BTreeSet::new();
        streets.insert(street_dup1);
        streets.insert(street_dup2); // won't have any effect beyond the first
        put_c2s_data(&mut db_guard, &region, &city, &streets);

        // We get the set with one item
        let result = data_access.street_names_for_city_in_region(&region, &city).unwrap();
        assert_eq!(result.len(), 1, "Duplicates unify in a single BTreeSet item");
        assert_eq!(result.iter().next().unwrap().name(), "samestreet");
    }
}
