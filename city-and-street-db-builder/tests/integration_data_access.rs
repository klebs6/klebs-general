// ---------------- [ File: tests/integration_data_access.rs ]
use world_city_and_street_db_builder::*;
use usa::*;
use world_region::*;
use tempfile::*;
use postal_code::*;
use country::*;

#[tokio::test]
async fn test_zip_codes_for_city_in_region_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let us_region = USRegion::UnitedState(UnitedState::Maryland);
    // Convert to WorldRegion:
    let world_region: WorldRegion = us_region.into();

    let db = Database::open(&temp_dir).expect("Failed to open DB");
    {
        let mut db_guard = db.lock().expect("DB lock poisoned");
        let rr = RegionalRecords::mock_for_region(&world_region);
        rr.write_to_storage(&mut *db_guard)
          .expect("Failed to write mock data to storage");
    }

    let da = DataAccess::with_db(db.clone());

    // We'll check that "Baltimore" has postal code 21201 and
    // that "Bethesda" has 20814, both from the mock_for_region data.
    let city1 = CityName::new("Baltimore").unwrap();
    let zips1 = da.postal_codes_for_city_in_region(&world_region, &city1);
    assert!(zips1.is_some(), "Expected to find postal codes for Baltimore in MD");
    let zips1_unwrapped = zips1.unwrap();
    assert!(
        zips1_unwrapped.contains(&PostalCode::new(Country::USA,"21201").unwrap()),
        "Expected to see 21201 in Baltimore's set"
    );

    // Also check that "Bethesda" returns 20814
    let city2 = CityName::new("Bethesda").unwrap();
    let zips2 = da.postal_codes_for_city_in_region(&world_region, &city2);
    assert!(zips2.is_some(), "Expected to find postal codes for Bethesda in MD");
    let zips2_unwrapped = zips2.unwrap();
    assert!(
        zips2_unwrapped.contains(&PostalCode::new(Country::USA,"20814").unwrap()),
        "Expected to see 20814 in Bethesda's set"
    );

    // Negative check: ensure that a random city not in the mock set has no postal codes
    let city_unknown = CityName::new("ImaginaryCity").unwrap();
    let zips_none = da.postal_codes_for_city_in_region(&world_region, &city_unknown);
    assert!(
        zips_none.is_none(),
        "CityName=ImaginaryCity should not exist in the MD mocks!"
    );
}
