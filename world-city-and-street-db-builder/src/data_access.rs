// ---------------- [ File: src/data_access.rs ]
crate::ix!();

/// DataAccess struct for queries
#[derive(Getters,Clone)]
#[getset(get="pub")]
pub struct DataAccess {
    db: Arc<Mutex<Database>>,
}

impl DataAccess {

    /// Creates a new DataAccess that wraps the given Database (thread-safe).
    pub fn with_db(db: Arc<Mutex<Database>>) -> Self {
        info!("creating DataAccess object");
        DataAccess { db }
    }
}

impl DataAccessInterface for DataAccess {}

pub trait DataAccessInterface
: CityNamesForPostalCodeInRegion
+ GatherAllZipsInRegion
+ GetCborSetTyped
+ GetCitySetForKey
+ GetPostalCodeSetForKey
+ GetStreetSetForKey
+ PostalCodesForCityInRegion
+ StreetExistsGlobally
+ StreetExistsInCityInRegion
+ StreetExistsInPostalCodeInRegion
+ StreetNamesForCityInRegion
+ StreetNamesForPostalCodeInRegion
{}

#[cfg(test)]
mod data_access_tests {

    use super::*;

    /// Creates a fresh Database + DataAccess. 
    /// The question mark operator (`?`) will automatically convert errors 
    /// to DataAccessError, thanks to error_tree! definitions.
    fn create_db_and_da() -> Result<(Arc<Mutex<Database>>, DataAccess), DataAccessError> {
        let tmp = TempDir::new()?;                // => DataAccessError::Io if fails
        let db = Database::open(tmp.path())?;     // => DataAccessError::DatabaseConstructionError if fails
        let da = DataAccess::with_db(db.clone());
        Ok((db, da))
    }

    /// Helper: writes a BTreeSet<T> as CBOR into the DB at `key`.
    fn put_set_into_db<T: serde::Serialize + serde::de::DeserializeOwned + Ord + Clone>(
        db: &mut Database,
        key: &str,
        items: &BTreeSet<T>
    ) -> Result<(), DataAccessError> {
        let cbor_data = compress_set_to_cbor(items);
        db.put(key, cbor_data)?; // => DataAccessError::DatabaseConstructionError
        Ok(())
    }

    // Some sample “constructor” functions:
    fn region_md() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }
    fn region_va() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Virginia).into()
    }

    fn city_baltimore() -> CityName {
        CityName::new("Baltimore").unwrap()
    }
    fn city_calverton() -> CityName {
        CityName::new("Calverton").unwrap()
    }

    fn street_north_avenue() -> StreetName {
        StreetName::new("North Avenue").unwrap()
    }
    fn street_catlett_road() -> StreetName {
        StreetName::new("Catlett Road").unwrap()
    }

    fn postal_21201() -> PostalCode {
        PostalCode::new(Country::USA, "21201").unwrap()
    }
    fn postal_20138_9997() -> PostalCode {
        PostalCode::new(Country::USA, "20138-9997").unwrap()
    }

    // --------------------------------------------
    // Basic .get_xxx_set usage
    // --------------------------------------------

    #[test]
    fn test_get_city_set_no_key() -> Result<(), DataAccessError> {
        let (_db, da) = create_db_and_da()?;
        let result = da.get_city_set("Z2C:US:99999");
        assert!(result.is_none());
        Ok(())
    }

    #[test]
    fn test_get_city_set_valid() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut db_guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = "Z2C:US:21201";
            let mut city_set = BTreeSet::new();
            city_set.insert(city_baltimore());
            put_set_into_db(&mut db_guard, key, &city_set)?;
        }
        let found = da.get_city_set("Z2C:US:21201");
        assert!(found.is_some());
        let set = found.unwrap();
        assert_eq!(set.len(), 1);
        assert!(set.contains(&city_baltimore()));
        Ok(())
    }

    #[test]
    fn test_get_city_set_empty() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut db_guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = "Z2C:US:EMPTY";
            let empty: BTreeSet<CityName> = BTreeSet::new();
            put_set_into_db(&mut db_guard, key, &empty)?;
        }
        let found = da.get_city_set("Z2C:US:EMPTY");
        assert!(found.is_none(), "Empty => None");
        Ok(())
    }

    #[test]
    fn test_get_street_set_single() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut db_guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = "C2S:US:baltimore";
            let mut st_set = BTreeSet::new();
            st_set.insert(street_north_avenue());
            put_set_into_db(&mut db_guard, key, &st_set)?;
        }
        let found = da.get_street_set("C2S:US:baltimore");
        assert!(found.is_some());
        let st = found.unwrap();
        assert!(st.contains(&street_north_avenue()));
        Ok(())
    }

    #[test]
    fn test_get_postal_code_set_multiple() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut db_guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = "C2Z:US:baltimore";
            let mut zips = BTreeSet::new();
            zips.insert(postal_21201());
            zips.insert(PostalCode::new(Country::USA, "21202").unwrap());
            put_set_into_db(&mut db_guard, key, &zips)?;
        }
        let found = da.get_postal_code_set("C2Z:US:baltimore");
        assert!(found.is_some());
        let codes = found.unwrap();
        assert_eq!(codes.len(), 2);
        Ok(())
    }

    // --------------------------------------------
    // Testing the trait-based convenience methods
    // --------------------------------------------

    #[test]
    fn test_postal_codes_for_city_in_region() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = c2z_key(&region_md(), &city_baltimore());
            let mut set = BTreeSet::new();
            set.insert(postal_21201());
            put_set_into_db(&mut guard, &key, &set)?;
        }
        let found = da.postal_codes_for_city_in_region(&region_md(), &city_baltimore());
        assert!(found.is_some());
        let s = found.unwrap();
        assert!(s.contains(&postal_21201()));
        Ok(())
    }

    #[test]
    fn test_street_names_for_city_in_region() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = c2s_key(&region_md(), &city_baltimore());
            let mut st = BTreeSet::new();
            st.insert(street_north_avenue());
            put_set_into_db(&mut guard, &key, &st)?;
        }
        let found = da.street_names_for_city_in_region(&region_md(), &city_baltimore());
        assert!(found.is_some());
        assert!(found.unwrap().contains(&street_north_avenue()));
        Ok(())
    }

    #[test]
    fn test_cities_for_postal_code() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = z2c_key(&region_md(), &postal_21201());
            let mut cset = BTreeSet::new();
            cset.insert(city_baltimore());
            put_set_into_db(&mut guard, &key, &cset)?;
        }
        let found = da.cities_for_postal_code(&region_md(), &postal_21201());
        assert!(found.is_some());
        assert!(found.unwrap().contains(&city_baltimore()));
        Ok(())
    }

    #[test]
    fn test_street_names_for_postal_code_in_region() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = s_key(&region_md(), &postal_21201());
            let mut st = BTreeSet::new();
            st.insert(street_north_avenue());
            put_set_into_db(&mut guard, &key, &st)?;
        }
        let found = da.street_names_for_postal_code_in_region(&region_md(), &postal_21201());
        assert!(found.is_some());
        assert!(found.unwrap().contains(&street_north_avenue()));
        Ok(())
    }

    // --------------------------------------------
    // Checking existence queries
    // --------------------------------------------

    #[test]
    fn test_street_exists_in_city_true() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = c2s_key(&region_md(), &city_baltimore());
            let mut st = BTreeSet::new();
            st.insert(street_north_avenue());
            put_set_into_db(&mut guard, &key, &st)?;
        }
        let exists = da.street_exists_in_city(&region_md(), &city_baltimore(), &street_north_avenue());
        assert!(exists);
        Ok(())
    }

    #[test]
    fn test_street_exists_in_city_false() -> Result<(), DataAccessError> {
        let (_db, da) = create_db_and_da()?;
        let exists = da.street_exists_in_city(&region_md(), &city_baltimore(), &street_north_avenue());
        assert!(!exists);
        Ok(())
    }

    #[test]
    fn test_street_exists_in_postal_code_true() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = s_key(&region_md(), &postal_21201());
            let mut st = BTreeSet::new();
            st.insert(street_north_avenue());
            put_set_into_db(&mut guard, &key, &st)?;
        }
        let exists = da.street_exists_in_postal_code(&region_md(), &postal_21201(), &street_north_avenue());
        assert!(exists);
        Ok(())
    }

    #[test]
    fn test_street_exists_in_postal_code_false() -> Result<(), DataAccessError> {
        let (_db, da) = create_db_and_da()?;
        let exists = da.street_exists_in_postal_code(&region_md(), &postal_21201(), &street_north_avenue());
        assert!(!exists);
        Ok(())
    }

    #[test]
    fn test_street_exists_globally_true() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let region = region_va();
            let street = street_catlett_road();

            let s2c_k = s2c_key(&region, &street);
            let mut cities = BTreeSet::new();
            cities.insert(city_calverton());
            put_set_into_db(&mut guard, &s2c_k, &cities)?;

            let s2z_k = s2z_key(&region, &street);
            let mut zips = BTreeSet::new();
            zips.insert(postal_20138_9997());
            put_set_into_db(&mut guard, &s2z_k, &zips)?;
        }
        let region = region_va();
        let street = street_catlett_road();
        let exists = da.street_exists_globally(&region, &street);
        assert!(exists);
        Ok(())
    }

    #[test]
    fn test_street_exists_globally_false() -> Result<(), DataAccessError> {
        let (_db, da) = create_db_and_da()?;
        let region = region_va();
        let street = street_catlett_road();
        let exists = da.street_exists_globally(&region, &street);
        assert!(!exists);
        Ok(())
    }

    // --------------------------------------------
    // Lock poisoning scenario
    // --------------------------------------------

    #[test]
    fn test_lock_poisoning_logged_as_warning() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        // Force a lock poison:
        let _ = std::panic::catch_unwind(|| {
            let guard = db.lock().unwrap();
            let _ = &guard; // just hold it
            panic!("Intentionally poisoning the lock");
        });
        // Now the lock is poisoned. 
        // The data_access calls `db.lock()`, sees an error => DataAccessError::LockPoisoned => logs -> returns None
        let val = da.get_city_set("some_key");
        assert!(val.is_none());
        Ok(())
    }
}
