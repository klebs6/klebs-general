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
mod test_street_exists_in_postal_code_in_region {
    use super::*;
    use std::collections::BTreeSet;
    use tempfile::TempDir;

    /// Helper to store a set of street names under `s_key(region, postal_code)`.
    fn put_s_data<I: StorageInterface>(
        db:          &mut I,
        region:      &WorldRegion,
        postal_code: &PostalCode,
        streets:     &BTreeSet<StreetName>,
    ) {
        let key = s_key(region, postal_code);
        let val = compress_set_to_cbor(streets);
        db.put(&key, val).expect("Storing postal->streets data should succeed");
    }

    #[traced_test]
    fn test_no_data_returns_false() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        // No writes => no lock needed
        let data_access = DataAccess::with_db(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let pc = PostalCode::new(Country::USA, "99999").unwrap();
        let street = StreetName::new("MissingStreet").unwrap();

        let result = data_access.street_exists_in_postal_code(&region, &pc, &street);
        assert!(!result, "No data => false");
    }

    #[traced_test]
    fn test_street_in_set_returns_true() {
        let (db_arc, _tmp) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("MD").unwrap();
            let pc = PostalCode::new(Country::USA, "21201").unwrap();
            let street = StreetName::new("North Avenue").unwrap();

            let mut streets = BTreeSet::new();
            streets.insert(street);
            put_s_data(&mut *db_guard, &region, &pc, &streets);
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let pc = PostalCode::new(Country::USA, "21201").unwrap();
        let street = StreetName::new("North Avenue").unwrap();

        let result = data_access.street_exists_in_postal_code(&region, &pc, &street);
        assert!(result, "Street in stored set => true");
    }

    #[traced_test]
    fn test_street_not_in_set_returns_false() {
        let (db_arc, _tmp) = create_temp_db::<Database>();

        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("MD").unwrap();
            let pc = PostalCode::new(Country::USA, "21230").unwrap();

            let known = StreetName::new("KnownStreet").unwrap();
            let mut streets = BTreeSet::new();
            streets.insert(known);
            put_s_data(&mut *db_guard, &region, &pc, &streets);
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let pc = PostalCode::new(Country::USA, "21230").unwrap();
        let missing_street = StreetName::new("MissingStreet").unwrap();

        let result = data_access.street_exists_in_postal_code(&region, &pc, &missing_street);
        assert!(!result, "Street not in set => false");
    }

    #[traced_test]
    fn test_different_postal_code_returns_false() {
        let (db_arc, _tmp) = create_temp_db::<Database>();

        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("MD").unwrap();

            let pc1 = PostalCode::new(Country::USA, "21201").unwrap();
            let street = StreetName::new("SharedStreet").unwrap();
            let mut streets = BTreeSet::new();
            streets.insert(street.clone());
            put_s_data(&mut *db_guard, &region, &pc1, &streets);
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let pc2 = PostalCode::new(Country::USA, "99999").unwrap();
        let street = StreetName::new("SharedStreet").unwrap();

        let result = data_access.street_exists_in_postal_code(&region, &pc2, &street);
        assert!(!result, "Stored under pc1 => query pc2 => false");
    }

    #[traced_test]
    fn test_different_region_returns_false() {
        let (db_arc, _tmp) = create_temp_db::<Database>();

        {
            let mut db_guard = db_arc.lock().unwrap();
            let region_md = WorldRegion::try_from_abbreviation("MD").unwrap();
            let pc = PostalCode::new(Country::USA, "21201").unwrap();
            let street = StreetName::new("RegionBoundStreet").unwrap();

            let mut streets = BTreeSet::new();
            streets.insert(street.clone());
            put_s_data(&mut *db_guard, &region_md, &pc, &streets);
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region_va = WorldRegion::try_from_abbreviation("VA").unwrap();
        let pc = PostalCode::new(Country::USA, "21201").unwrap();
        let street = StreetName::new("RegionBoundStreet").unwrap();

        let result = data_access.street_exists_in_postal_code(&region_va, &pc, &street);
        assert!(!result, "Different region => false");
    }

    #[traced_test]
    fn test_corrupted_cbor_returns_false() {
        let (db_arc, _tmp) = create_temp_db::<Database>();

        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("MD").unwrap();
            let pc = PostalCode::new(Country::USA, "21201").unwrap();
            let key = s_key(&region, &pc);
            db_guard.put(key, b"not valid cbor").unwrap();
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let pc = PostalCode::new(Country::USA, "21201").unwrap();
        let street = StreetName::new("BadStreet").unwrap();

        let result = data_access.street_exists_in_postal_code(&region, &pc, &street);
        assert!(!result, "Corrupted => decode fails => false");
    }
}
