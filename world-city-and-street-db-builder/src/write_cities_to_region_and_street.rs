// ---------------- [ File: src/write_cities_to_region_and_street.rs ]
// ---------------- [ File: src/write_cities_to_region_and_street.rs ]
crate::ix!();

pub trait WriteCitiesToRegionAndStreet {
    fn write_cities_to_region_and_street(
        &mut self, 
        region: &WorldRegion, 
        street: &StreetName, 
        cities: &BTreeSet<CityName>
    ) -> Result<(),DatabaseConstructionError>;
}

impl WriteCitiesToRegionAndStreet for Database {

    fn write_cities_to_region_and_street(
        &mut self, 
        region: &WorldRegion, 
        street: &StreetName, 
        cities: &BTreeSet<CityName>
    ) -> Result<(),DatabaseConstructionError> {
        let key = s2c_key(region,street);
        self.put(&key, compress_set_to_cbor(cities))?;
        Ok(())
    }
}

#[cfg(test)]
mod test_write_cities_to_region_and_street {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Utility for building a `CityName`. Adjust if your constructor differs.
    fn city(name: &str) -> CityName {
        CityName::new(name).unwrap()
    }

    /// Helper to retrieve the stored data from RocksDB under the `s2c_key`.
    /// Returns `None` if key is absent or decoding fails.
    fn load_cities_from_s2c<I:StorageInterface>(
        db:     &I,
        region: &WorldRegion,
        street: &StreetName
    ) -> Option<BTreeSet<CityName>> {
        let key = s2c_key(region, street);
        match db.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let clist_result: Result<crate::CompressedList<CityName>, _> =
                    serde_cbor::from_slice(&bytes);
                match clist_result {
                    Ok(clist) => Some(clist.items().iter().cloned().collect()),
                    Err(_) => None,
                }
            }
            _ => None,
        }
    }

    #[traced_test]
    fn test_write_cities_first_time() {
        // No prior data => store new set => verify it’s correct
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("TestStreet").unwrap();

        // A set of two cities
        let mut cities = BTreeSet::new();
        cities.insert(city("Baltimore"));
        cities.insert(city("Laurel"));

        let result = db_guard
            .write_cities_to_region_and_street(&region, &street, &cities);
        assert!(result.is_ok(), "Writing city set should succeed");

        // Now read back from DB
        let stored_opt = load_cities_from_s2c(&*db_guard, &region, &street);
        assert!(stored_opt.is_some(), "Should have data");
        let stored = stored_opt.unwrap();
        assert_eq!(stored, cities, "Retrieved set should match what we stored");
    }

    #[traced_test]
    fn test_overwrite_existing_cities() {
        // If there's already data, writing a new set overwrites it.
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let street = StreetName::new("Main Ave").unwrap();

        // First store some initial data
        let mut initial_cities = BTreeSet::new();
        initial_cities.insert(city("Arlington"));
        db_guard
            .write_cities_to_region_and_street(&region, &street, &initial_cities)
            .expect("write initial");

        // Then store new data
        let mut new_cities = BTreeSet::new();
        new_cities.insert(city("Alexandria"));
        new_cities.insert(city("Falls Church"));
        db_guard
            .write_cities_to_region_and_street(&region, &street, &new_cities)
            .expect("write new data");

        // Confirm only the new data remains
        let stored_opt = load_cities_from_s2c(&*db_guard, &region, &street);
        let stored = stored_opt.unwrap();
        assert_eq!(stored, new_cities, "Should reflect the newly stored data only");
    }

    #[traced_test]
    fn test_empty_set_is_stored() {
        // It's valid to store an empty set. We'll confirm it decodes as an empty set later.
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let street = StreetName::new("Empty Blvd").unwrap();

        let cities = BTreeSet::new(); // empty

        db_guard
            .write_cities_to_region_and_street(&region, &street, &cities)
            .expect("Should store empty set successfully");

        let stored_opt = load_cities_from_s2c(&*db_guard, &region, &street);
        assert!(stored_opt.is_some(), "Key should exist even if empty");
        let stored = stored_opt.unwrap();
        assert!(stored.is_empty(), "Should decode as empty set");
    }

    #[traced_test]
    fn test_rocksdb_error_on_put() {
        // If put fails => returns DatabaseConstructionError. We'll define a minimal failing stub.

        let mut failing_stub = FailingDbStub;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("FailStreet").unwrap();
        let mut city_set = BTreeSet::new();
        city_set.insert(city("FailureCity"));

        let result = failing_stub.write_cities_to_region_and_street(&region, &street, &city_set);
        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated put error");
            }
            other => panic!("Expected DatabaseConstructionError::RocksDB, got {:?}", other),
        }
    }
}
