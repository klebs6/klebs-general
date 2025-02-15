// ---------------- [ File: tests/mini_pbf_integration_test.rs ]
/// This file demonstrates an **integration test** that uses a small `.osm.pbf` fixture
/// to test the full pipeline:
///   1. Loading the file with `osmpbf::ElementReader`
///   2. Iterating each element
///   3. Attempting `AddressRecord::try_from((element, &Country::USA))`
///   4. Checking the resulting addresses
///
/// Create a minimal "tiny.osm.pbf" fixture in `tests/fixtures/tiny.osm.pbf`.

use world_region_db::*;
use country::*;
use osmpbf::ElementReader;
use std::path::Path;

#[tokio::test]
async fn test_parse_tiny_osm_pbf_fixture() {

    // 1) create ephemeral .osm.pbf in a temp dir
    let tmp          = tempfile::TempDir::new().unwrap();
    let fixture_path = tmp.path().join("tiny.osm.pbf");

    create_tiny_osm_pbf(&fixture_path).await.expect("create pbf");

    let reader = match ElementReader::from_path(&fixture_path) {
        Ok(r) => r,
        Err(e) => {
            panic!("Failed to open fixture at {:?}: {:?}", fixture_path, e);
        }
    };

    let mut addresses = Vec::new();
    // For each element in the PBF, we try converting it to an AddressRecord:
    let parse_result = reader.for_each(|element| {
        println!("element: {:#?}", element);
        match AddressRecord::try_from((&element, &Country::USA)) {
            Ok(addr) => {
                addresses.push(addr);
            }
            Err(_e) => {
                // We can ignore the error or log it if desired,
                // since not every OSM element has to be an address.
            }
        }
    });

    // If reading succeeded, parse_result is Ok
    assert!(parse_result.is_ok(), "Should parse the file successfully");

    // Now we have some addresses in `addresses`:
    assert!(
        !addresses.is_empty(),
        "Expected at least one address in the tiny fixture!"
    );

    // We might check for a known address from the fixture:
    let found = addresses.iter().any(|addr| {
        addr.city().as_ref().map(|c| c.name()) == Some(&"test city fixture".to_string())
            && addr.street().as_ref().map(|s| s.name()) == Some(&"test street fixture".to_string())
    });
    assert!(
        found,
        "Expected to find a known fixture address, but not found in parsed results!"
    );
}

