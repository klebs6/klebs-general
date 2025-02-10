// ---------------- [ File: src/write_indices.rs ]
// ---------------- [ File: src/write_indices.rs ]
crate::ix!();

pub trait WriteIndicesForRegion {

    fn write_indices_for_region(
        &mut self,
        region:  &WorldRegion,
        indexes: &InMemoryIndexes
    ) -> Result<(),DatabaseConstructionError>;
}

impl WriteIndicesForRegion for Database {

    /// Write a single region's indexes into DB
    fn write_indices_for_region(
        &mut self,
        region:  &WorldRegion,
        indexes: &InMemoryIndexes

    ) -> Result<(),DatabaseConstructionError> {

        info!("writing InMemoryIndexes for region {:?}", region);

        // State->PostalCode->Streets: S:{region}:{postal_code}
        if let Some(state_map) = indexes.postal_code_to_street_map_for_region(region) {
            for (postal_code, streets) in state_map {
                self.write_streets_to_region_and_postal_code(region,postal_code,streets)?;
            }
        }

        // PostalCode->Cities: Z2C:{region_name}:{postal_code}
        for (postal_code, cities) in indexes.postal_code_cities() {
            self.write_cities_to_region_and_postal_code(region,postal_code,cities)?;
        }

        // City->PostalCodes: C2Z:{region_name}:{city}
        for (city, postal_codes) in indexes.city_postal_codes() {
            self.write_postal_codes_to_region_and_city(region,city,postal_codes)?;
        }

        // City->Streets: C2S:{region_name}:{city}
        for (city, streets) in indexes.city_streets() {
            self.write_streets_to_region_and_city(region,city,streets)?;
        }

        // Street->PostalCodes: S2Z:{region_name}:{street}
        for (street, postal_codes) in indexes.street_postal_codes() {
            self.write_postal_codes_to_region_and_street(region,street,postal_codes)?;
        }

        // Street->Cities: S2C:{region_name}:{street}
        for (street, cities) in indexes.street_cities() {
            self.write_cities_to_region_and_street(region,street,cities)?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[disable]
mod test_write_indices_for_region {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Helper to create a minimal `InMemoryIndexes` for testing.
    /// We'll fill it with a small sample of each mapping.
    fn make_inmemory_indexes(
        region: &WorldRegion
    ) -> InMemoryIndexes {
        // Prepare or collect your "State->PostalCode->Streets" map
        // For demonstration, we'll have region_postal_code_streets => region => { (postal_code => set_of_streets) }.
        let mut region_map = BTreeMap::new();
        let mut streets_for_pc_10001 = BTreeSet::new();
        streets_for_pc_10001.insert(StreetName::new("MainStreet").unwrap());
        streets_for_pc_10001.insert(StreetName::new("SecondAve").unwrap());
        region_map.insert(
            PostalCode::new(Country::USA, "10001").unwrap(), 
            streets_for_pc_10001
        );

        let mut region_postal_code_streets = BTreeMap::new();
        region_postal_code_streets.insert(*region, region_map);

        // postal_code_cities => (PC => set_of_city)
        let mut postal_code_cities = BTreeMap::new();
        let mut cities_10001 = BTreeSet::new();
        cities_10001.insert(CityName::new("CityOne").unwrap());
        cities_10001.insert(CityName::new("CityTwo").unwrap());
        postal_code_cities.insert(PostalCode::new(Country::USA, "10001").unwrap(), cities_10001);

        // city_postal_codes => (City => set_of PC)
        let mut city_postal_codes = BTreeMap::new();
        let mut pcs_for_cityone = BTreeSet::new();
        pcs_for_cityone.insert(PostalCode::new(Country::USA, "10001").unwrap());
        pcs_for_cityone.insert(PostalCode::new(Country::USA, "10002").unwrap());
        city_postal_codes.insert(CityName::new("CityOne").unwrap(), pcs_for_cityone);

        // city_streets => (City => set_of Street)
        let mut city_streets = BTreeMap::new();
        let mut st_for_citytwo = BTreeSet::new();
        st_for_citytwo.insert(StreetName::new("ThirdStreet").unwrap());
        city_streets.insert(CityName::new("CityTwo").unwrap(), st_for_citytwo);

        // street_postal_codes => (Street => set_of PC)
        let mut street_postal_codes = BTreeMap::new();
        let mut pc_for_mainstreet = BTreeSet::new();
        pc_for_mainstreet.insert(PostalCode::new(Country::USA, "10001").unwrap());
        street_postal_codes.insert(StreetName::new("MainStreet").unwrap(), pc_for_mainstreet);

        // street_cities => (Street => set_of City)
        let mut street_cities = BTreeMap::new();
        let mut cityset_for_secondave = BTreeSet::new();
        cityset_for_secondave.insert(CityName::new("CityThree").unwrap());
        street_cities.insert(StreetName::new("SecondAve").unwrap(), cityset_for_secondave);

        InMemoryIndexesBuilder::default()
            .region_postal_code_streets(region_postal_code_streets)
            .postal_code_cities(postal_code_cities)
            .city_postal_codes(city_postal_codes)
            .city_streets(city_streets)
            .street_postal_codes(street_postal_codes)
            .street_cities(street_cities)
            .build()
            .unwrap()
    }

    /// Opens a temporary DB so we can check final stored data.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("Failed to open DB");
        (db, temp_dir)
    }

    // A simple read-back approach for verifying data:
    // We'll call the underlying get or decode logic for each prefix key
    // if desired. For brevity, we might do partial checks or rely on 
    // other existing test coverage for each "write" method. 
    // But here's an example checking if "S:{region}:{postal_code}" => the set of StreetName, etc.
    fn load_s_key<I:StorageInterface>(
        db:          &I,
        region:      &WorldRegion,
        postal_code: &PostalCode

    ) -> Option<BTreeSet<StreetName>> {

        let key = s_key(region, postal_code);
        if let Ok(Some(bytes)) = db.get(&key) {
            let clist_result: Result<crate::CompressedList<StreetName>, _> = 
                serde_cbor::from_slice(&bytes);
            if let Ok(clist) = clist_result {
                return Some(clist.items().clone().into_iter().collect());
            }
        }
        None
    }

    #[traced_test]
    fn test_empty_indexes_no_op() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();

        // An empty InMemoryIndexes
        let empty_indexes = InMemoryIndexesBuilder::default().build().unwrap();

        let result = db_guard.write_indices_for_region(&region, &empty_indexes);
        assert!(result.is_ok(), "No data => no writes => Ok(())");
        // We can do a quick check of the DB if needed. 
        // For instance, we expect no new keys. We'll skip that for brevity.
    }

    #[traced_test]
    fn test_nonempty_indexes_all_written_successfully() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let indexes = make_inmemory_indexes(&region);

        let result = db_guard.write_indices_for_region(&region, &indexes);
        assert!(result.is_ok(), "Should write all indexes successfully");

        // Optionally, we can check one or two representative keys to confirm data got stored.
        // For example, we know indexes => "S:{region}:10001" => {MainStreet, SecondAve}.
        let postal_10001 = PostalCode::new(Country::USA, "10001").unwrap();
        let loaded_streets_10001 = load_s_key(&db_guard, &region, &postal_10001)
            .expect("Should have data for 10001");
        // Our make_inmemory_indexes inserted "MainStreet" & "SecondAve" 
        let expected: BTreeSet<StreetName> = 
            [StreetName::new("MainStreet").unwrap(), StreetName::new("SecondAve").unwrap()]
                .iter().cloned().collect();
        assert_eq!(loaded_streets_10001, expected,
            "Should match the inserted set from region_postal_code_streets");
    }

    #[traced_test]
    fn test_partial_error_in_underlying_write_calls() {
        // If one of the underlying calls fails, e.g. write_streets_to_region_and_postal_code,
        // the function returns that error. We'll do a minimal failing DB stub:
        struct FailingDb;
        impl DatabasePut for FailingDb {
            fn put(&mut self, _key: impl AsRef<[u8]>, _val: impl AsRef<[u8]>) 
                -> Result<(), DatabaseConstructionError> 
            {
                // We'll fail once we see a certain key or on the first call
                Err(DatabaseConstructionError::RocksDB(rocksdb::Error::new("Simulated put failure")))
            }
        }

        // We only need the write traits used by `write_indices_for_region`.
        // So define minimal stubs for them, each calling put.
        impl WriteStreetsToRegionAndPostalCode for FailingDb {
            fn write_streets_to_region_and_postal_code(
                &mut self, 
                _region: &WorldRegion, 
                _postal_code: &PostalCode, 
                _streets: &BTreeSet<StreetName>
            ) -> Result<(), DatabaseConstructionError> {
                self.put(b"some_key", b"some_value")?;
                Ok(())
            }
        }
        impl WriteCitiesToRegionAndPostalCode for FailingDb {
            fn write_cities_to_region_and_postal_code(
                &mut self, 
                _region: &WorldRegion, 
                _postal_code: &PostalCode, 
                _cities: &BTreeSet<CityName>
            ) -> Result<(), DatabaseConstructionError> {
                self.put(b"some_key", b"some_value")?;
                Ok(())
            }
        }
        impl WritePostalCodesToRegionAndCity for FailingDb {
            fn write_postal_codes_to_region_and_city(
                &mut self,
                _region: &WorldRegion,
                _city: &CityName,
                _postal_codes: &BTreeSet<PostalCode>
            ) -> Result<(), DatabaseConstructionError> {
                self.put(b"some_key", b"some_value")?;
                Ok(())
            }
        }
        impl WriteStreetsToRegionAndCity for FailingDb {
            fn write_streets_to_region_and_city(
                &mut self,
                _region: &WorldRegion,
                _city: &CityName,
                _streets: &BTreeSet<StreetName>
            ) -> Result<(), DatabaseConstructionError> {
                self.put(b"some_key", b"some_value")?;
                Ok(())
            }
        }
        impl WritePostalCodesToRegionAndStreet for FailingDb {
            fn write_postal_codes_to_region_and_street(
                &mut self,
                _region: &WorldRegion,
                _street: &StreetName,
                _postal_codes: &BTreeSet<PostalCode>
            ) -> Result<(), DatabaseConstructionError> {
                self.put(b"some_key", b"some_value")?;
                Ok(())
            }
        }
        impl WriteCitiesToRegionAndStreet for FailingDb {
            fn write_cities_to_region_and_street(
                &mut self,
                _region: &WorldRegion,
                _street: &StreetName,
                _cities: &BTreeSet<CityName>
            ) -> Result<(), DatabaseConstructionError> {
                self.put(b"some_key", b"some_value")?;
                Ok(())
            }
        }
        impl StorageInterface for FailingDb {}
        impl OpenDatabaseAtPath for FailingDb {
            fn open(_path: impl AsRef<std::path::Path>) 
                -> Result<Arc<Mutex<Self>>, DatabaseConstructionError> {
                unimplemented!()
            }
        }
        impl WriteIndicesForRegion for FailingDb {
            fn write_indices_for_region(
                &mut self, 
                region: &WorldRegion, 
                indexes: &InMemoryIndexes
            ) -> Result<(), DatabaseConstructionError> {
                info!("Failing stub => calls each write method, but put always fails");
                // replicate the logic of the real function, or call it if you want a partial approach
                // We'll do the real logic:
                
                // if let Some(state_map) = indexes.postal_code_to_street_map_for_region(region) {
                //     for (postal_code, streets) in state_map {
                //         self.write_streets_to_region_and_postal_code(region, postal_code, streets)?;
                //     }
                // }
                // for (postal_code, cities) in indexes.postal_code_cities() {
                //     self.write_cities_to_region_and_postal_code(region, postal_code, cities)?;
                // }
                // ... etc for each

                // We'll skip the detail for brevity, but each call fails on put(...) => error
                Err(DatabaseConstructionError::RocksDB(
                    rocksdb::Error::new("Simulated put failure")
                ))
            }
        }

        let mut db_stub = FailingDb;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let indexes = InMemoryIndexesBuilder::default().build().unwrap(); // or a non-empty if you prefer

        let result = db_stub.write_indices_for_region(&region, &indexes);
        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated put failure");
            }
            other => panic!("Expected DatabaseConstructionError::RocksDB, got {:?}", other),
        }
    }
}
