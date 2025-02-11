// ---------------- [ File: src/create_small_osm_pbf_file.rs ]
crate::ix!();

/// Creates a minimal `.osm.pbf` file with a single Node.
/// The caller can optionally specify:
///   - A bounding box.
///   - City/street strings.
///   - An optional housenumber.
///
/// This consolidates the logic from both `create_tiny_osm_pbf` and
/// `create_tiny_osm_pbf_with_housenumber` into one function. The two
/// old functions become thin wrappers that invoke this one with fixed
/// parameters.
///
/// # Arguments
///
/// * `path` - Filesystem path for the `.osm.pbf` file to be created.
/// * `bbox` - (left, right, top, bottom) bounding box in "nano-degrees"
///            (1e-9 degrees). E.g., -77_000_000_000 for -77.0.
/// * `city`         - The `addr:city` value to store.
/// * `street`       - The `addr:street` value to store.
/// * `housenumber`  - Optional `addr:housenumber` value, e.g. "100-110".
/// * `lat`/`lon`    - Latitude/Longitude for the node.
/// * `node_id`      - OSM node ID to assign.
///
/// # Returns
///
/// * `Ok(())` on success.
/// * `Err(std::io::Error)` if I/O or serialization fails.
pub async fn create_small_osm_pbf_file(
    path:        &Path,
    bbox:        (i64, i64, i64, i64),
    city:        &str,
    street:      &str,
    postcode:    &str,
    housenumber: Option<&str>,
    lat:         f64,
    lon:         f64,
    node_id:     i64,

) -> std::io::Result<()> {

    trace!(
        "create_small_osm_pbf_file: invoked for path={:?}, node_id={}, city={}, street={}, postcode={:?}, housenumber={:?}, lat={}, lon={}",
        path, node_id, city, street, postcode, housenumber, lat, lon
    );

    // Ensure path is suitable for file creation
    validate_not_dir(path)?;

    // 1) Prepare OSMHeader block & serialize
    let header_block = prepare_osm_header_block(bbox);
    let (header_blobheader_bytes, header_blob_bytes) =
        serialize_osm_header_block(header_block)?;

    // 2) Prepare PrimitiveBlock with a single node & optional housenumber, then serialize
    let primitive_block = prepare_single_node_primitive_block(
        city, 
        street, 
        postcode, 
        housenumber, 
        lat, 
        lon, 
        node_id
    );
    let (data_blobheader_bytes, data_blob_bytes) =
        serialize_primitive_block(primitive_block)?;

    // 3) Perform asynchronous file writes
    write_osm_pbf_file(
        path,
        &header_blobheader_bytes,
        &header_blob_bytes,
        &data_blobheader_bytes,
        &data_blob_bytes
    ).await?;

    info!("create_small_osm_pbf_file: successfully wrote OSM PBF to {:?}", path);
    Ok(())
}

#[cfg(test)]
mod create_small_osm_pbf_file_tests {
    use super::*;
    use std::fs::File;
    use std::io::{Write, ErrorKind};
    use std::path::PathBuf;
    use tempfile::TempDir;
    use osmpbf::{Element, ElementReader};

    /// A helper to parse the resulting `.pbf` and confirm there's exactly one node
    /// with the expected city/street/housenumber. If `expected_hn` is `None`, we ensure
    /// there's no `addr:housenumber`. We also check lat/lon if desired.
    fn parse_and_verify_pbf(
        path: &std::path::Path,
        expected_lat: f64,
        expected_lon: f64,
        expected_city: &str,
        expected_street: &str,
        expected_hn: Option<&str>,
    ) {
        let reader = ElementReader::from_path(path).expect("Should open pbf for read");
        let mut node_count = 0;

        // We'll track what we found
        let mut found_city = None;
        let mut found_street = None;
        let mut found_hn = None;
        let (mut lat_found, mut lon_found) = (None, None);

        reader
            .for_each(|element| {
                if let Element::Node(node) = element {
                    node_count += 1;

                    // lat/lon checks
                    // raw lat/lon from osmpbf is typically in 1e-7 or 1e-9 scaling,
                    // but let's do `node.lat()` if available:
                    lat_found = Some(node.lat());
                    lon_found = Some(node.lon());

                    // gather tags
                    for (k, v) in node.tags() {
                        match k {
                            "addr:city" => found_city = Some(v.to_string()),
                            "addr:street" => found_street = Some(v.to_string()),
                            "addr:housenumber" => found_hn = Some(v.to_string()),
                            _ => {}
                        }
                    }
                }
            })
            .expect("Should parse each element");

        assert_eq!(node_count, 1, "We expect exactly 1 node in the .pbf");

        // lat/lon close to expected? We only do a rough check if desired, because the final
        // pbf lat/lon might reflect internal offset/granularity. But let's do approximate check:
        let lat_ok = (lat_found.unwrap_or_default() - expected_lat).abs() < 0.0001;
        let lon_ok = (lon_found.unwrap_or_default() - expected_lon).abs() < 0.0001;
        assert!(
            lat_ok && lon_ok,
            "Lat/lon do not match expected. Found=({:?},{:?}), want=({},{})",
            lat_found, lon_found, expected_lat, expected_lon
        );

        // city/street
        assert_eq!(
            found_city.as_deref(),
            Some(expected_city),
            "addr:city mismatch"
        );
        assert_eq!(
            found_street.as_deref(),
            Some(expected_street),
            "addr:street mismatch"
        );

        // optional housenumber
        match (expected_hn, found_hn) {
            (Some(want), Some(got)) => {
                assert_eq!(want, got, "addr:housenumber mismatch");
            }
            (Some(want), None) => {
                panic!("Expected housenumber='{}', but found none", want);
            }
            (None, Some(got)) => {
                panic!("Expected no housenumber, found='{}'", got);
            }
            (None, None) => {
                // ok => no housenumber expected or found
            }
        }
    }

    #[traced_test]
    async fn test_create_small_osm_pbf_file_success_no_housenumber() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("single_node.osm.pbf");

        // bounding box => near -77..-76..39..38 for example
        let bbox = (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000);
        let lat = 39.283_f64;
        let lon = -76.616_f64;
        let city = "Baltimore";
        let street = "North Avenue";
        let postcode = "11111";

        let res = create_small_osm_pbf_file(
            &path,
            bbox,
            city,
            street,
            postcode,
            None, // no housenumber
            lat,
            lon,
            1001,
        )
        .await;

        assert!(res.is_ok(), "Should succeed creating .pbf file");

        // parse => confirm 1 node with city/street, no housenumber
        parse_and_verify_pbf(&path, lat, lon, "Baltimore", "North Avenue", None);
    }

    #[traced_test]
    async fn test_create_small_osm_pbf_file_success_with_housenumber() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("with_hn.osm.pbf");

        let bbox = (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000);
        let lat = 40.0;
        let lon = -75.5;
        let city = "TestCity";
        let street = "TestStreet";
        let housenumber = Some("100-110");
        let postcode = "11111";

        let res = create_small_osm_pbf_file(
            &path,
            bbox,
            city,
            street,
            postcode,
            housenumber,
            lat,
            lon,
            2002,
        )
        .await;

        assert!(res.is_ok());
        parse_and_verify_pbf(&path, lat, lon, "TestCity", "TestStreet", housenumber);
    }

    #[traced_test]
    async fn test_create_small_osm_pbf_file_path_is_dir() {
        let tmp = TempDir::new().unwrap();
        // We'll pass the directory path instead of a file => `validate_not_dir(...)` => error
        let dir_path = tmp.path();

        let bbox = (-70_000_000_000, -69_000_000_000, 35_000_000_000, 34_000_000_000);
        let lat = 35.0;
        let lon = -69.5;

        let res = create_small_osm_pbf_file(
            dir_path,
            bbox,
            "City",
            "Street",
            "11111",
            None,
            lat,
            lon,
            9999,
        )
        .await;

        assert!(res.is_err(), "Passing a directory => std::io::Error");
        let err = res.err().unwrap();
        assert_eq!(err.kind(), ErrorKind::Other, "We produce an Other or relevant error kind");
    }

    #[traced_test]
    async fn test_create_small_osm_pbf_file_unwritable_path() {
        // We'll try to pass something like '/root/some_unwritable_file.osm.pbf' on Unix,
        // or a bogus path on Windows. This is OS-specific. If we can't reliably 
        // test unwritable, we can at least do a partial approach: pass a path 
        // containing invalid characters on Windows, or something.

        #[cfg(unix)]
        {
            let path = PathBuf::from("/this/path/is/not/writable.osm.pbf");
            let bbox = (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000);

            let res = create_small_osm_pbf_file(
                &path, 
                bbox, 
                "City", 
                "Street", 
                "11111", 
                None, 
                39.0, 
                -76.0, 
                123
            ).await;

            assert!(res.is_err(), "Unwritable path => error");
        }
    }

    #[traced_test]
    async fn test_create_small_osm_pbf_file_various_bbox() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("varied_bbox.osm.pbf");

        // Suppose we do a bounding box for e.g. from lat=45 => 46, so:
        // left= -120.0 => -120_000_000_000, right=-119.0 => -119_000_000_000, top=46.0 => 46_000_000_000, bottom=45 => 45_000_000_000
        let bbox = (-120_000_000_000, -119_000_000_000, 46_000_000_000, 45_000_000_000);

        let city = "AnywhereVille";
        let street = "Main St";
        let postcode = "11111";
        let lat = 45.5;
        let lon = -119.5;

        let res = create_small_osm_pbf_file(
            &path,
            bbox,
            city,
            street,
            postcode,
            None,
            lat,
            lon,
            3003,
        )
        .await;
        assert!(res.is_ok());

        // parse => confirm lat/lon, city/street
        parse_and_verify_pbf(&path, lat, lon, "AnywhereVille", "Main St", None);
    }

    #[traced_test]
    async fn test_create_small_osm_pbf_file_lat_lon_out_of_range() {
        // The function doesn't necessarily enforce lat/lon within -90..90 or -180..180.
        // It logs a warning. We'll see if it just proceeds. 
        // We'll just confirm it doesn't fail.
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("extreme_latlon.osm.pbf");

        let bbox = (-180_000_000_000, 180_000_000_000, 90_000_000_000, -90_000_000_000);
        // lat= 95.0 => out of normal range, lon=200 => out
        let lat = 95.0;
        let lon = 200.0;
        let city = "OutOfRangeCity";
        let street = "SomeRoad";
        let postcode = "11111";

        let res = create_small_osm_pbf_file(
            &path,
            bbox,
            city,
            street,
            postcode,
            Some("42"),
            lat,
            lon,
            4004,
        )
        .await;

        // The code might only log a warning about lat/lon range. We assume it doesn't fail.
        // If your code specifically forbids lat>90 => error, adapt the test.
        assert!(res.is_ok(), "We do not forcibly fail if lat/lon out-of-range, just logs a warning");

        // parse => confirm it *did* produce a node. We'll see lat/lon might be truncated by granularity.
        parse_and_verify_pbf(&path, lat, lon, "OutOfRangeCity", "SomeRoad", Some("42"));
    }
}
