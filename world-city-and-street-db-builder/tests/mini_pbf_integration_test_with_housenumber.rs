use world_city_and_street_db_builder::*;
use country::*;
use usa::*;
use world_region::*;
use tempfile::*;
use osmpbf::ElementReader;
use std::path::Path;

#[tokio::test]
async fn test_parse_osm_with_house_range_and_store_in_db() {
    // 1) Create a temp dir for both DB and the .pbf file
    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let pbf_path = tmp_dir.path().join("test_housenumbers.osm.pbf");

    // 2) Generate our tiny PBF that has addr:housenumber="100-110"
    create_tiny_osm_pbf_with_housenumber(&pbf_path).await
        .expect("failed to create PBF with house range");

    // 3) Open RocksDB
    let db_arc = Database::open(tmp_dir.path()).expect("failed to open DB");

    // Our test region => e.g. "Maryland"
    let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

    // 4) load_osm_data_with_housenumbers => aggregator => store in DB
    {
        let (records, house_number_ranges) 
            = load_osm_data_with_housenumbers(pbf_path,&region).unwrap();

        assert_eq!(records.len(), 1, "We have exactly one node with address info in that tiny PBF");

        let rr = RegionalRecordsBuilder::default()
            .region(region)
            .records(records)
            .house_number_ranges(house_number_ranges)
            .build()
            .unwrap();

        let mut db_guard = db_arc.lock().unwrap();
        rr.write_to_storage(&mut *db_guard).unwrap();
    }

    // 5) Now load from DB: we expect the street to be "test street", normalized => "test street"
    let street_obj = StreetName::new("TestStreet").expect("valid street");
    let db_guard = db_arc.lock().unwrap();
    let hnr_result = db_guard.load_house_number_ranges(&region, &street_obj)
        .expect("should load ranges from DB without error");

    assert!(hnr_result.is_some(), "We must have at least one sub-range");
    let ranges = hnr_result.unwrap();
    assert_eq!(ranges.len(), 1, "Should unify the entire range into 1 subrange");
    let only_range = &ranges[0];
    assert_eq!(only_range.start(), &100);
    assert_eq!(only_range.end(), &110);

    println!("Test OK => found house number sub-range: {}-{}", 
        only_range.start(), only_range.end());
}
