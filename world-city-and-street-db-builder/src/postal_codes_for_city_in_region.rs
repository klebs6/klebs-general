// ---------------- [ File: src/postal_codes_for_city_in_region.rs ]
crate::ix!();

pub trait PostalCodesForCityInRegion {

    fn postal_codes_for_city_in_region(
        &self, 
        region: &WorldRegion, 
        city:   &CityName
    ) -> Option<BTreeSet<PostalCode>>;
}

impl<I:StorageInterface> PostalCodesForCityInRegion for DataAccess<I> {

    // Example query: given city name, get associated PostalCode codes
    fn postal_codes_for_city_in_region(&self, region: &WorldRegion, city: &CityName) -> Option<BTreeSet<PostalCode>> {
        let key = c2z_key(region,city);
        if let Some(postal_codes) = self.get_postal_code_set(&key) {
            Some(postal_codes)
        } else {
            None
        }
    }
}

#[cfg(test)]
#[disable]
mod test_postal_codes_for_city_in_region {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};
    use std::collections::BTreeSet;

    /// Creates an in-memory or temporary database, returns `(Arc<Mutex<Database>>, TempDir)`.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let db  = I::open(tmp.path()).expect("Failed to open database in temp dir");
        (db, tmp)
    }

    /// Builds a `DataAccess` reference from the same underlying `Database`.
    fn create_data_access<I:StorageInterface>(db_arc: Arc<Mutex<I>>) -> DataAccess<I> {
        DataAccess::with_db(db_arc)
    }

    /// Convenience to store a `BTreeSet<PostalCode>` under the `C2Z:{region_abbr}:{city}` key.
    fn put_c2z_postal_codes<I:StorageInterface>(
        db:     &mut I,
        region: &WorldRegion,
        city:   &CityName,
        codes:  &BTreeSet<PostalCode>,
    ) {
        let key = c2z_key(region, city);
        let val = compress_set_to_cbor(codes);
        db.put(key, val).expect("Storing postal codes should succeed in test setup");
    }

    #[test]
    fn test_no_data_returns_none() {
        let (db_arc, _tmp) = create_temp_db();
        let data_access = create_data_access(db_arc);

        // Suppose region=MD, city=baltimore, but we never stored any c2z data
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("baltimore").unwrap();

        let result = data_access.postal_codes_for_city_in_region(&region, &city);
        assert!(result.is_none(), "No data => should return None");
    }

    #[test]
    fn test_some_data_returns_btreeset_of_postal_codes() {
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("baltimore").unwrap();

        // Insert some sample postal codes
        let mut codes = BTreeSet::new();
        codes.insert(PostalCode::new(Country::USA, "21201").unwrap());
        codes.insert(PostalCode::new(Country::USA, "21230").unwrap());

        put_c2z_postal_codes(&mut db_guard, &region, &city, &codes);

        drop(db_guard); // release the lock before reading

        let result = data_access.postal_codes_for_city_in_region(&region, &city);
        assert!(result.is_some(), "We have data => should return Some");
        let set = result.unwrap();
        assert_eq!(set, codes, "PostalCode set should match what was stored");
    }

    #[test]
    fn test_corrupted_data_returns_none() {
        // If the CBOR data is invalid, `get_cbor_set_typed` returns None internally,
        // so we get None from `postal_codes_for_city_in_region`.
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("annapolis").unwrap();
        let key = c2z_key(&region, &city);

        // Write junk bytes
        db_guard.put(&key, b"corrupted cbor data").unwrap();

        drop(db_guard);

        let result = data_access.postal_codes_for_city_in_region(&region, &city);
        assert!(result.is_none(), "Corrupted data => None (decoding fails)");
    }

    #[test]
    fn test_region_and_city_case_insensitivity() {
        // This depends on your code's normalization. If `city.name()` is already normalized,
        // we can confirm that "Baltimore" with different cases is stored or retrieved.
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        // We'll store data for city="baltimore" => c2z:MD:baltimore
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("BaLtImOrE").unwrap(); // The CityName constructor normalizes it
        let mut codes = BTreeSet::new();
        codes.insert(PostalCode::new(Country::USA, "21201").unwrap());
        put_c2z_postal_codes(&mut db_guard, &region, &city, &codes);

        drop(db_guard);

        // Query using city="BALTIMORE" or "baltimore" => same result
        let city_query = CityName::new("BALTIMORE").unwrap();
        let result = data_access.postal_codes_for_city_in_region(&region, &city_query);
        assert!(result.is_some(), "Should find data regardless of case");
        assert_eq!(result.unwrap(), codes);
    }

    #[test]
    fn test_different_city_different_key() {
        // If we store data for "frederick" but query "hagerstown", 
        // we expect None because the keys differ.
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city_frederick = CityName::new("frederick").unwrap();
        let city_hagerstown = CityName::new("hagerstown").unwrap();

        let mut codes = BTreeSet::new();
        codes.insert(PostalCode::new(Country::USA, "21701").unwrap());
        put_c2z_postal_codes(&mut db_guard, &region, &city_frederick, &codes);

        drop(db_guard);

        // Query a different city
        let result = data_access.postal_codes_for_city_in_region(&region, &city_hagerstown);
        assert!(result.is_none(), "Data belongs to different city => None");
    }
}
