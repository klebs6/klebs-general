use usa_city_and_street_db_builder::*;
use usa::*;
use tempfile::*;
use postal_code::*;
use country::*;

#[tokio::test]
async fn test_zip_codes_for_city_in_region_integration() {

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let region = USRegion::UnitedState(UnitedState::Maryland);

    let db = Database::open(&temp_dir).unwrap();
    {
        let mut db_guard = db.lock().unwrap();
        let rr = RegionalRecords::mock_for_region(&region);
        rr.write_to_storage(&mut *db_guard).unwrap();
    }

    let da = DataAccess::with_db(db.clone());
    let city = CityName::new("Baltimore").unwrap();
    let region = USRegion::UnitedState(UnitedState::Maryland);

    let zips = da.zip_codes_for_city_in_region(&region, &city);
    assert!(zips.is_some());
    let zips = zips.unwrap();
    assert!(zips.contains(&PostalCode::new(Country::USA,"21201").unwrap()));
}

