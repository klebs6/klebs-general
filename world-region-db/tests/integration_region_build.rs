// ---------------- [ File: tests/integration_region_build.rs ]
use world_region_db::*;
use world_region::*;
use usa::*;
use tempfile::*;

#[tokio::test]
async fn integration_test_region_build() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Database::open(&temp_dir).expect("Failed to open DB");

    {
        let mut db_guard = db.lock().expect("DB lock poisoned");

        // Build a small set of USRegion variants, then convert each to a WorldRegion
        let regions: Vec<USRegion> = vec![
            USRegion::UnitedState(UnitedState::Maryland),
            USRegion::UnitedState(UnitedState::Virginia),
            USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia),
        ];

        for us_region in &regions {
            let world_region: WorldRegion = (*us_region).into();

            let rr = RegionalRecords::mock_for_region(&world_region);
            rr.write_to_storage(&mut *db_guard)
              .expect("write_to_storage should succeed for mock data");

            assert!(
                db_guard.region_done(&world_region).expect("region_done failed"),
                "Expected region to be marked done"
            );
        }
    }

    // Optionally, verify we can retrieve some data from each region
    let _da = DataAccess::with_db(db.clone());
    // e.g., check that a city from DC can be found, etc.
    // but for brevity, we just confirm no panic.
}
