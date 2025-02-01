// ---------------- [ File: tests/integration_validate_address.rs ]
use world_city_and_street_db_builder::*;
use usa::*;
use world_region::*;
use tempfile::*;

#[tokio::test]
async fn integration_test_validate_address() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Database::open(&temp_dir).expect("Failed to open DB");

    // Convert to WorldRegion
    let us_region = USRegion::UnitedState(UnitedState::Maryland);
    let world_region: WorldRegion = us_region.into();

    {
        let mut db_guard = db.lock().expect("DB lock poisoned");
        let rr = RegionalRecords::mock_for_region(&world_region);
        rr.write_to_storage(&mut *db_guard)
          .expect("Failed to store region data");
    }

    // Now create a known mock address for the entire system
    let address = WorldAddress::mock();
    let da = DataAccess::with_db(db.clone());

    // Check that the address is valid
    let res = address.validate_with(&da);
    assert!(res.is_ok(), "Expected the default mock address to be valid");
    
    // If you want to be more robust, build a known 'invalid' address
    // to confirm it fails:
    let invalid_address = WorldAddressBuilder::default()
        .region(world_region)
        .postal_code(address.postal_code().clone())  // correct zip
        .city(address.city().clone())                // correct city
        .street(StreetName::new("TotallyFakeStreet").unwrap()) // mismatch
        .build()
        .expect("Build should succeed, we are only testing existence mismatch");
    let invalid_res = invalid_address.validate_with(&da);
    assert!(invalid_res.is_err(), "Street should not exist, expecting error");
}
