// ---------------- [ File: tests/integration_validate_address.rs ]
use world_region_db::*;
use usa::*;
use world_region::*;
use tempfile::*;

#[tokio::test]
async fn integration_test_validate_address() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Database::open(&temp_dir).expect("Failed to open DB");

    {
        let mut db_guard = db.lock().expect("DB lock poisoned");
        // store Maryland
        let md_rr = RegionalRecords::mock_for_region(&WorldRegion::from(
            USRegion::UnitedState(UnitedState::Florida),
        ));
        md_rr.write_to_storage(&mut *db_guard)
            .expect("Failed to store FL data");

        // store Virginia
        let va_rr = RegionalRecords::mock_for_region(&WorldRegion::from(
            USRegion::UnitedState(UnitedState::Texas),
        ));
        va_rr.write_to_storage(&mut *db_guard)
            .expect("Failed to store VA data");
    }

    // Now the default mock address is definitely in the DB
    let address = WorldAddress::mock(); // region=Florida => now in DB
    let da = DataAccess::with_db(db.clone());
    let res = address.validate_with(&da);
    assert!(res.is_ok());
}
