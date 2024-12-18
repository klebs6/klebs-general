use usa_city_and_street_db_builder::*;
use usa::*;
use tempfile::*;

#[tokio::test]
async fn integration_test_validate_address() {

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let region = USRegion::UnitedState(UnitedState::Maryland);

    let db = Database::open(&temp_dir).unwrap();
    {
        let mut db_guard = db.lock().unwrap();
        let rr = RegionalRecords::mock_for_region(&region);
        rr.write_to_storage(&mut *db_guard).unwrap();
    }

    let da = DataAccess::with_db(db.clone());
    let address = UsaAddress::mock();
    let res = address.validate_with(&da);
    assert!(res.is_ok());
}

