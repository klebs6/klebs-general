// ---------------- [ File: src/city_names_for_postal_code_in_region.rs ]
crate::ix!();

pub trait CityNamesForPostalCodeInRegion {

    fn cities_for_postal_code(
        &self, 
        region_name: &WorldRegion, 
        postal_code: &PostalCode
    ) -> Option<BTreeSet<CityName>>;
}

impl<I:StorageInterface> CityNamesForPostalCodeInRegion for DataAccess<I> {

    fn cities_for_postal_code(&self, region: &WorldRegion, postal_code: &PostalCode) -> Option<BTreeSet<CityName>> {
        let key = z2c_key(region,postal_code);
        self.get_city_set(&key)
    }
}

#[cfg(test)]
mod city_names_for_postal_code_in_region_tests {
    use super::*;
    use std::collections::BTreeSet;
    use tempfile::TempDir;

    /// Creates a temporary DB, returning `Arc<Mutex<Database>>` and the corresponding `DataAccess`.
    fn create_db_and_data_access<I:StorageInterface>() -> (Arc<Mutex<I>>, DataAccess<I>) {
        let tmp_dir = TempDir::new().unwrap();
        let db = I::open(tmp_dir.path()).unwrap();
        let da = DataAccess::with_db(db.clone());
        (db, da)
    }

    /// Helper to encode a set of `CityName` as CBOR and store it under the `Z2C:<region>:<postal>` key.
    fn store_city_set<I:StorageInterface>(
        db:          &mut I,
        region:      &WorldRegion,
        postal_code: &PostalCode,
        cities:      &BTreeSet<CityName>
    ) {
        let key = z2c_key(region, postal_code);
        let cbor_val = crate::compress_set_to_cbor(cities);
        db.put(key.as_bytes(), cbor_val).unwrap();
    }

    /// Builds a typed CityName set from multiple &str inputs
    fn city_set(cities: &[&str]) -> BTreeSet<CityName> {
        cities
            .iter()
            .map(|name| CityName::new(name).unwrap())
            .collect()
    }

    // A small helper region. If your code needs a real one, adapt accordingly.
    fn region_usa() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    #[test]
    fn test_cities_for_postal_code_no_key() {
        let (db_arc, da) = create_db_and_data_access::<Database>();
        let region = region_usa();
        let postal = PostalCode::new(Country::USA, "12345").unwrap();

        // No data => searching yields None
        let found = da.cities_for_postal_code(&region, &postal);
        assert!(found.is_none(), "No key => None");
    }

    #[test]
    fn test_cities_for_postal_code_single_city() {
        let (db_arc, da) = create_db_and_data_access::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = region_usa();
            let postal = PostalCode::new(Country::USA, "21201").unwrap();
            let cities = city_set(&["Baltimore"]);
            store_city_set(&mut *db_guard, &region, &postal, &cities);
        }

        let region = region_usa();
        let postal = PostalCode::new(Country::USA, "21201").unwrap();
        let found = da.cities_for_postal_code(&region, &postal);
        assert!(found.is_some(), "Should find a single city set");
        let set = found.unwrap();
        assert_eq!(set.len(), 1);
        assert!(set.contains(&CityName::new("Baltimore").unwrap()));
    }

    #[test]
    fn test_cities_for_postal_code_multiple_cities() {
        let (db_arc, da) = create_db_and_data_access::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = region_usa();
            let postal = PostalCode::new(Country::USA, "21202").unwrap();
            let cities = city_set(&["Baltimore", "Towson", "Columbia"]);
            store_city_set(&mut *db_guard, &region, &postal, &cities);
        }

        let region = region_usa();
        let postal = PostalCode::new(Country::USA, "21202").unwrap();
        let found = da.cities_for_postal_code(&region, &postal);
        assert!(found.is_some());
        let set = found.unwrap();
        assert_eq!(set.len(), 3, "Should have 3 distinct cities");
        assert!(set.contains(&CityName::new("Baltimore").unwrap()));
        assert!(set.contains(&CityName::new("Towson").unwrap()));
        assert!(set.contains(&CityName::new("Columbia").unwrap()));
    }

    #[test]
    fn test_cities_for_postal_code_empty_cbor() {
        let (db_arc, da) = create_db_and_data_access::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = region_usa();
            let postal = PostalCode::new(Country::USA, "55555").unwrap();
            let empty = BTreeSet::new(); // no city
            store_city_set(&mut *db_guard, &region, &postal, &empty);
        }

        // If the stored set is empty, get_city_set(...) -> None
        let region = region_usa();
        let postal = PostalCode::new(Country::USA, "55555").unwrap();
        let found = da.cities_for_postal_code(&region, &postal);
        assert!(found.is_none(), "Empty => None");
    }

    #[test]
    fn test_cities_for_postal_code_corrupted_cbor() {
        let (db_arc, da) = create_db_and_data_access::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            // We'll forcibly store invalid cbor
            let region = region_usa();
            let postal = PostalCode::new(Country::USA, "99999").unwrap();
            let key = z2c_key(&region, &postal);
            let corrupted_bytes = b"This is not valid cbor".to_vec();
            db_guard.put(key.as_bytes(), corrupted_bytes).unwrap();
        }

        let region = region_usa();
        let postal = PostalCode::new(Country::USA, "99999").unwrap();
        let found = da.cities_for_postal_code(&region, &postal);
        // Because we can't decode, get_city_set => None
        assert!(found.is_none(), "Corrupted => decode fails => None");
    }

    #[test]
    fn test_cities_for_postal_code_lock_poisoning() {
        let (db_arc, da) = create_db_and_data_access::<Database>();

        // Force lock poisoning
        let _ = std::panic::catch_unwind(|| {
            let guard = db_arc.lock().unwrap();
            panic!("Intentional panic to poison the lock");
        });

        // Now the lock is poisoned => get_city_set => None
        // or it might log a warning, depending on your implementation.
        let region = region_usa();
        let postal = PostalCode::new(Country::USA, "00000").unwrap();
        let found = da.cities_for_postal_code(&region, &postal);
        assert!(found.is_none(), "Lock poisoning => likely returns None");
    }
}
