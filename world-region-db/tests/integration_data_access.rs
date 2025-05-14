// ---------------- [ File: tests/integration_data_access.rs ]
use world_region_db::*;
use usa::*;
use world_region::*;
use tempfile::*;
use postal_code::*;
use country::*;

#[tokio::test]
async fn test_zip_codes_for_city_in_region_integration() {
    // We'll now use Tennessee instead of Maryland
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let us_region = USRegion::UnitedState(UnitedState::Tennessee);
    let world_region: WorldRegion = us_region.into();

    let db = Database::open(&temp_dir).expect("Failed to open DB");
    {
        let mut db_guard = db.lock().expect("DB lock poisoned");
        let rr = RegionalRecords::mock_for_region(&world_region);
        rr.write_to_storage(&mut *db_guard)
          .expect("Failed to write mock data to storage");
    }

    let da = DataAccess::with_db(db.clone());

    // Based on `tennessee_mock_records()`, we expect:
    // Memphis => 38103, Knoxville => 37902, Nashville => 37201

    // 1) Check Memphis => "38103"
    let memphis = CityName::new("memphis").unwrap();
    let zips_memphis = da.postal_codes_for_city_in_region(&world_region, &memphis);
    assert!(zips_memphis.is_some(), "Expected postal codes for Memphis, TN");
    let zips_memphis_unwrapped = zips_memphis.unwrap();
    assert!(
        zips_memphis_unwrapped.contains(&PostalCode::new(Country::USA,"38103").unwrap()),
        "Expected to see 38103 in Memphis's set"
    );

    // 2) Check Knoxville => "37902"
    let knoxville = CityName::new("knoxville").unwrap();
    let zips_knoxville = da.postal_codes_for_city_in_region(&world_region, &knoxville);
    assert!(zips_knoxville.is_some(), "Expected postal codes for Knoxville, TN");
    let zips_knoxville_unwrapped = zips_knoxville.unwrap();
    assert!(
        zips_knoxville_unwrapped.contains(&PostalCode::new(Country::USA,"37902").unwrap()),
        "Expected to see 37902 in Knoxville's set"
    );

    // 3) Check Nashville => "37201"
    let nashville = CityName::new("nashville").unwrap();
    let zips_nashville = da.postal_codes_for_city_in_region(&world_region, &nashville);
    assert!(zips_nashville.is_some(), "Expected postal codes for Nashville, TN");
    let zips_nashville_unwrapped = zips_nashville.unwrap();
    assert!(
        zips_nashville_unwrapped.contains(&PostalCode::new(Country::USA,"37201").unwrap()),
        "Expected to see 37201 in Nashville's set"
    );

    // 4) Negative check: "ImaginaryCity" => no postal codes
    let city_unknown = CityName::new("ImaginaryCity").unwrap();
    let zips_none = da.postal_codes_for_city_in_region(&world_region, &city_unknown);
    assert!(
        zips_none.is_none(),
        "CityName=ImaginaryCity should not exist in the TN mocks!"
    );
}
