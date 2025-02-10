// ---------------- [ File: src/create_tiny_osm_pbf_with_housenumber.rs ]
// ---------------- [ File: src/create_tiny_osm_pbf_with_housenumber.rs ]
crate::ix!();

/// Thin wrapper around [`create_small_osm_pbf_file`] producing a single Node
/// with `addr:housenumber = "100-110"`.
///
/// # Bounding box: near Baltimore
/// # City: `"TestCity"`, Street: `"TestStreet"`, housenumber: `"100-110"`
pub async fn create_tiny_osm_pbf_with_housenumber(path: impl AsRef<Path>) -> std::io::Result<()> {
    trace!("create_tiny_osm_pbf_with_housenumber: starting for path={:?}", path.as_ref());

    create_small_osm_pbf_file(
        path.as_ref(),
        (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000),
        "TestCity",
        "TestStreet",
        Some("100-110"),
        39.283,
        -76.616,
        1001,
    ).await
}

#[cfg(test)]
mod create_tiny_osm_pbf_with_housenumber_tests {
    use super::*;
    use std::io::ErrorKind;
    use tempfile::TempDir;
    use osmpbf::{ElementReader, Element};

    /// Helper to parse the .pbf and confirm we have exactly one node
    /// with city="TestCity", street="TestStreet", housenumber="100-110",
    /// lat/lon near (39.283, -76.616).
    fn parse_and_verify_with_housenumber(path: &std::path::Path) {
        let reader = ElementReader::from_path(path).expect("open pbf");
        let mut node_count = 0;
        let mut found_city = None;
        let mut found_street = None;
        let mut found_hn = None;
        let (mut lat_found, mut lon_found) = (None, None);

        reader
            .for_each(|element| {
                if let Element::Node(node) = element {
                    node_count += 1;
                    lat_found = Some(node.lat());
                    lon_found = Some(node.lon());
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
            .expect("parse each element in pbf");

        assert_eq!(node_count, 1, "Expected exactly one node");
        assert_eq!(
            found_city.as_deref(),
            Some("TestCity"),
            "addr:city mismatch"
        );
        assert_eq!(
            found_street.as_deref(),
            Some("TestStreet"),
            "addr:street mismatch"
        );
        assert_eq!(
            found_hn.as_deref(),
            Some("100-110"),
            "addr:housenumber mismatch"
        );

        // Rough lat/lon check
        let lat_ok = (lat_found.unwrap_or_default() - 39.283).abs() < 0.0001;
        let lon_ok = (lon_found.unwrap_or_default() + 76.616).abs() < 0.0001;
        assert!(
            lat_ok && lon_ok,
            "Expected lat=39.283, lon=-76.616; found=({:?},{:?})",
            lat_found,
            lon_found
        );
    }

    #[traced_test]
    async fn test_create_tiny_osm_pbf_with_housenumber_success() {
        let tmp = TempDir::new().unwrap();
        let pbf_path = tmp.path().join("with_hn.osm.pbf");
        let res = create_tiny_osm_pbf_with_housenumber(&pbf_path).await;
        assert!(res.is_ok(), "Should successfully create pbf with housenumber");

        parse_and_verify_with_housenumber(&pbf_path);
    }

    #[traced_test]
    async fn test_create_tiny_osm_pbf_with_housenumber_path_is_directory() {
        let tmp = TempDir::new().unwrap();
        // Passing the directory path => should fail
        let dir_path = tmp.path();

        let res = create_tiny_osm_pbf_with_housenumber(dir_path).await;
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert_eq!(err.kind(), ErrorKind::Other, "Directory => error kind=Other");
    }

    #[traced_test]
    async fn test_create_tiny_osm_pbf_with_housenumber_unwritable_path() {
        #[cfg(unix)]
        {
            let pbf_path = std::path::PathBuf::from("/this/is/not/writable/with_hn.osm.pbf");
            let res = create_tiny_osm_pbf_with_housenumber(&pbf_path).await;
            assert!(res.is_err(), "Unwritable path => error");
        }
    }
}
