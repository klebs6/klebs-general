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
mod test_street_exists_globally {
    use super::*;
    use std::collections::BTreeSet;
    use tempfile::TempDir;

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
        let (db_arc, _td) = create_temp_db::<Database>();
        // No direct DB writes here, so no lock needed
        let data_access = DataAccess::with_db(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Unknown Street").unwrap();

        let result = data_access.street_exists_globally(&region, &street);
        assert!(!result, "No data => street should not exist");
    }

    #[traced_test]
    fn test_s2c_key_exists_returns_true() {
        let (db_arc, _td) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("MD").unwrap();
            let street = StreetName::new("Main St").unwrap();

            let mut city_set = BTreeSet::new();
            city_set.insert(CityName::new("Baltimore").unwrap());
            put_s2c_data(&mut *db_guard, &region, &street, &city_set);
        } // lock is dropped here

        // Now we can safely call data_access:
        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Main St").unwrap();

        let result = data_access.street_exists_globally(&region, &street);
        assert!(result, "S2C key is present => street should exist");
    }

    #[traced_test]
    fn test_s2z_key_exists_returns_true() {
        let (db_arc, _td) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("VA").unwrap();
            let street = StreetName::new("Another St").unwrap();

            let mut postal_codes = BTreeSet::new();
            postal_codes.insert(PostalCode::new(Country::USA, "20190").unwrap());
            put_s2z_data(&mut *db_guard, &region, &street, &postal_codes);
        } // drop lock

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let street = StreetName::new("Another St").unwrap();

        let result = data_access.street_exists_globally(&region, &street);
        assert!(result, "S2Z key is present => street should exist");
    }

    #[traced_test]
    fn test_both_keys_exist_also_returns_true() {
        let (db_arc, _td) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("DC").unwrap();
            let street = StreetName::new("Pennsylvania Ave").unwrap();

            let mut city_set = BTreeSet::new();
            city_set.insert(CityName::new("Washington").unwrap());
            put_s2c_data(&mut *db_guard, &region, &street, &city_set);

            let mut postal_codes = BTreeSet::new();
            postal_codes.insert(PostalCode::new(Country::USA, "20001").unwrap());
            put_s2z_data(&mut *db_guard, &region, &street, &postal_codes);
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let street = StreetName::new("Pennsylvania Ave").unwrap();

        let result = data_access.street_exists_globally(&region, &street);
        assert!(result, "If both S2C & S2Z exist, it is definitely true");
    }

    #[traced_test]
    fn test_corrupted_cbor_treated_as_none() {
        let (db_arc, _td) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("MD").unwrap();
            let street = StreetName::new("Corrupted St").unwrap();
            let s2c = s2c_key(&region, &street);
            db_guard.put(&s2c, b"not valid cbor").unwrap();
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Corrupted St").unwrap();

        let result = data_access.street_exists_globally(&region, &street);
        assert!(!result, "Corrupted cbor => decode fails => returns false");
    }

    #[traced_test]
    fn test_region_is_mismatched() {
        let (db_arc, _td) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region_md = WorldRegion::try_from_abbreviation("MD").unwrap();
            let street = StreetName::new("Boundary Street").unwrap();

            let mut city_set = BTreeSet::new();
            city_set.insert(CityName::new("SomeCity").unwrap());
            put_s2c_data(&mut *db_guard, &region_md, &street, &city_set);
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region_va = WorldRegion::try_from_abbreviation("VA").unwrap();
        let street = StreetName::new("Boundary Street").unwrap();

        let result = data_access.street_exists_globally(&region_va, &street);
        assert!(!result, "Data was stored under MD, but we queried VA => false");
    }
}
