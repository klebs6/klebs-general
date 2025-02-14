// ---------------- [ File: src/street_exists_in_city_in_region.rs ]
crate::ix!();

pub trait StreetExistsInCityInRegion {

    fn street_exists_in_city(
        &self, 
        region_name: &WorldRegion, 
        city:        &CityName, 
        street:      &StreetName
    ) -> bool;
}

impl<I:StorageInterface> StreetExistsInCityInRegion for DataAccess<I> {

    fn street_exists_in_city(
        &self, 
        region: &WorldRegion, 
        city:   &CityName, 
        street: &StreetName

    ) -> bool {

        if let Some(sts) = self.street_names_for_city_in_region(region, city) {
            sts.contains(street)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test_street_exists_in_city_in_region {
    use super::*;
    use std::collections::BTreeSet;
    use tempfile::TempDir;

    /// Helper to store a set of street names under the `C2S:{region_abbr}:{city}` key.
    fn put_c2s_data<I:StorageInterface>(
        db:      &mut I,
        region:  &WorldRegion,
        city:    &CityName,
        streets: &BTreeSet<StreetName>,
    ) {
        let key = c2s_key(region, city);
        let val = compress_set_to_cbor(streets);
        db.put(key, val).expect("Storing city->streets data should succeed");
    }

    #[traced_test]
    fn test_no_data_returns_false() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();
        // No data written, so no lock needed here
        let data_access = DataAccess::with_db(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("EmptyCity").unwrap();
        let street = StreetName::new("UnknownStreet").unwrap();

        let result = data_access.street_exists_in_city(&region, &city, &street);
        assert!(!result, "No data => should be false");
    }

    #[traced_test]
    fn test_street_in_city_returns_true() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("MD").unwrap();
            let city = CityName::new("Baltimore").unwrap();
            let street = StreetName::new("North Avenue").unwrap();

            let mut streets = BTreeSet::new();
            streets.insert(street.clone());
            put_c2s_data(&mut *db_guard, &region, &city, &streets);
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("Baltimore").unwrap();
        let street = StreetName::new("North Avenue").unwrap();

        let result = data_access.street_exists_in_city(&region, &city, &street);
        assert!(result, "Street is in the stored set => true");
    }

    #[traced_test]
    fn test_street_not_in_stored_set_returns_false() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("MD").unwrap();
            let city = CityName::new("Baltimore").unwrap();

            let known_street = StreetName::new("Greenmount Ave").unwrap();
            let mut streets = BTreeSet::new();
            streets.insert(known_street);
            put_c2s_data(&mut *db_guard, &region, &city, &streets);
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("Baltimore").unwrap();
        let missing_street = StreetName::new("DoesNotExist Ave").unwrap();

        let result = data_access.street_exists_in_city(&region, &city, &missing_street);
        assert!(!result, "Missing street => false");
    }

    #[traced_test]
    fn test_different_city_returns_false() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("MD").unwrap();
            let city1 = CityName::new("Baltimore").unwrap();
            let street = StreetName::new("SharedStreet").unwrap();

            let mut streets = BTreeSet::new();
            streets.insert(street.clone());
            put_c2s_data(&mut *db_guard, &region, &city1, &streets);
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city2 = CityName::new("Annapolis").unwrap();
        let street = StreetName::new("SharedStreet").unwrap();

        let result = data_access.street_exists_in_city(&region, &city2, &street);
        assert!(!result, "Stored under different city => false");
    }

    #[traced_test]
    fn test_different_region_returns_false() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region_md = WorldRegion::try_from_abbreviation("MD").unwrap();
            let city = CityName::new("RichCity").unwrap();
            let street = StreetName::new("RichStreet").unwrap();

            let mut streets = BTreeSet::new();
            streets.insert(street.clone());
            put_c2s_data(&mut *db_guard, &region_md, &city, &streets);
        }

        let data_access = DataAccess::with_db(db_arc.clone());

        let region_va = WorldRegion::try_from_abbreviation("VA").unwrap();
        let city = CityName::new("RichCity").unwrap();
        let street = StreetName::new("RichStreet").unwrap();

        let result = data_access.street_exists_in_city(&region_va, &city, &street);
        assert!(!result, "No data for VA => false");
    }

    #[traced_test]
    fn test_corrupted_data_returns_false() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();

        // -- Write scope --
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("MD").unwrap();
            let city = CityName::new("GlitchedCity").unwrap();
            let key = c2s_key(&region, &city);
            db_guard.put(key, b"not valid cbor").unwrap();
        }

        let data_access = DataAccess::with_db(db_arc.clone());
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("GlitchedCity").unwrap();
        let street = StreetName::new("GlitchedStreet").unwrap();

        let result = data_access.street_exists_in_city(&region, &city, &street);
        assert!(!result, "Corrupted => decode fails => false");
    }
}
