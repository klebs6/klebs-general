use usa_city_and_street_db_builder::*;
use usa::*;
use tempfile::*;
use std::path::PathBuf;

#[tokio::test]
async fn integration_test_region_build() {

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let region = USRegion::UnitedState(UnitedState::Maryland);

    let db = Database::open(&temp_dir).unwrap();
    {
        let mut db_guard = db.lock().unwrap();

        // Attempt building DMV regions mock:
        let regions: Vec<USRegion> = vec![
            USRegion::UnitedState(UnitedState::Maryland),
            USRegion::UnitedState(UnitedState::Virginia),
            USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia),
        ];

        // We mock actual build since we can't really download here:
        for region in &regions {
            let rr = RegionalRecords::mock_for_region(&region);
            rr.write_to_storage(&mut *db_guard).unwrap();
            assert!(db_guard.region_done(region).unwrap());
        }
    }
}
