// ---------------- [ File: src/write_cities_to_region_and_postal_code.rs ]
// ---------------- [ File: src/write_cities_to_region_and_postal_code.rs ]
crate::ix!();

pub trait WriteCitiesToRegionAndPostalCode {
    fn write_cities_to_region_and_postal_code(
        &mut self, 
        region:      &WorldRegion, 
        postal_code: &PostalCode, 
        cities:      &BTreeSet<CityName>
    ) -> Result<(),DatabaseConstructionError> ;
}

impl WriteCitiesToRegionAndPostalCode for Database {

    fn write_cities_to_region_and_postal_code(
        &mut self, 
        region:      &WorldRegion, 
        postal_code: &PostalCode, 
        cities:      &BTreeSet<CityName>

    ) -> Result<(),DatabaseConstructionError> {

        let key = z2c_key(region,postal_code);
        let val = compress_set_to_cbor(cities);
        self.put(&key, val)?;
        Ok(())
    }
}

#[cfg(test)]
mod test_write_cities_to_region_and_postal_code {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// A small helper for building a CityName.
    fn city(name: &str) -> CityName {
        CityName::new(name).unwrap()
    }

    /// Reads back what's stored under `z2c_key(region, postal_code)` to confirm it matches.
    /// Returns `None` if the key is missing or decoding fails.
    fn load_cities_from_db<I:StorageInterface>(
        db:          &I,
        region:      &WorldRegion,
        postal_code: &PostalCode

    ) -> Option<BTreeSet<CityName>> {

        let key     = z2c_key(region, postal_code);
        let val_opt = db.get(&key).ok()??;

        let clist_result: Result<crate::CompressedList<CityName>, _> 
            = serde_cbor::from_slice(&val_opt);

        clist_result.ok().map(|cl| cl.items().clone().into_iter().collect())
    }

    #[traced_test]
    fn test_write_cities_ok() {
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "21201").unwrap();

        // We'll define a set of cities
        let mut city_set = BTreeSet::new();
        city_set.insert(city("Baltimore"));
        city_set.insert(city("Highlandtown"));

        // Write
        db_guard
            .write_cities_to_region_and_postal_code(&region, &postal_code, &city_set)
            .expect("Should store successfully");

        // Now read back
        let loaded = load_cities_from_db(&*db_guard, &region, &postal_code)
            .expect("Should find stored data");
        assert_eq!(loaded, city_set, "Data read back should match stored set");
    }

    #[traced_test]
    fn test_overwrite_existing_cities() {
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let postal_code = PostalCode::new(Country::USA, "20190").unwrap();

        // First store an initial set
        let mut city_set = BTreeSet::new();
        city_set.insert(city("Arlington"));
        db_guard
            .write_cities_to_region_and_postal_code(&region, &postal_code, &city_set)
            .expect("write initial");

        // Overwrite with new set
        let mut new_city_set = BTreeSet::new();
        new_city_set.insert(city("Reston"));
        new_city_set.insert(city("Herndon"));
        db_guard
            .write_cities_to_region_and_postal_code(&region, &postal_code, &new_city_set)
            .expect("write updated");

        // Confirm old data is replaced
        let loaded = load_cities_from_db(&*db_guard, &region, &postal_code)
            .expect("Should exist after overwrite");
        assert_eq!(loaded, new_city_set, "Should reflect the new data only");
    }

    #[traced_test]
    fn test_rocksdb_error_on_put() {
        // If a RocksDB error occurs on put, we return DatabaseConstructionError. 
        // We'll define a minimal failing stub.

        let mut failing_db = FailingDbStub;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "99999").unwrap();

        let mut city_set = BTreeSet::new();
        city_set.insert(city("FailTown"));

        let result = failing_db
            .write_cities_to_region_and_postal_code(&region, &postal_code, &city_set);

        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated put error");
            }
            other => panic!("Expected RocksDB error, got {:?}", other),
        }
    }
}
