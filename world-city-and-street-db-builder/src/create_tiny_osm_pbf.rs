// ---------------- [ File: src/create_tiny_osm_pbf.rs ]
/// Creates a very small .osm.pbf file with:
///   - A single OSMHeader blob
///   - A single OSMData blob that contains one node with two address tags
///
/// The resulting file should be enough for a test fixture in your integration tests.
///
/// Note: This uses the `osmpbf::proto::{fileformat,osmformat}` modules,
///       which `osmpbf` normally uses internally for reading. They’re not officially
///       documented for writing, but you can still access them in your own code.
/// ---------------- [ File: src/create_tiny_osm_pbf.rs ]
crate::ix!();

// Pull in the generated protobuf structs from the `osmpbf` crate
//
// TODO: pull request created on the upstream to expose these:
//
// ```rust
//use osmpbf::protos::fileformat;
//use osmpbf::protos::osmformat;
//```
use crate::proto::{fileformat, osmformat}; // our newly generated modules

/// Thin wrapper around [`create_small_osm_pbf_file`] producing a single Node
/// without an `addr:housenumber`.
///
/// # Bounding box: near Baltimore
/// # City: `"test city fixture"`, Street: `"test street fixture"`, lat/lon near 39.283/-76.616
pub async fn create_tiny_osm_pbf(path: impl AsRef<Path>) -> std::io::Result<()> {
    trace!("create_tiny_osm_pbf: starting for path={:?}", path.as_ref());

    create_small_osm_pbf_file(
        path.as_ref(),
        (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000),
        "test city fixture",
        "test street fixture",
        None,
        39.283,
        -76.616,
        1001,
    ).await
}

#[cfg(test)]
mod create_tiny_osm_pbf_tests {
    use super::*;

    /// A helper to parse the newly created file and confirm a single node
    /// with city/street and **no** housenumber, lat/lon near (39.283, -76.616).
    fn parse_and_check_pbf(path: &std::path::Path) {
        let reader = ElementReader::from_path(path).expect("Should open pbf for read");
        let mut node_count = 0;
        let (mut found_city, mut found_street, mut found_hn) = (None, None, None);
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
            .expect("Parsing pbf elements must succeed");

        // Expect exactly 1 node
        assert_eq!(node_count, 1, "Should contain exactly one node");
        assert_eq!(
            found_city.as_deref(),
            Some("test city fixture"),
            "addr:city mismatch"
        );
        assert_eq!(
            found_street.as_deref(),
            Some("test street fixture"),
            "addr:street mismatch"
        );
        assert!(
            found_hn.is_none(),
            "Should have no housenumber tag in create_tiny_osm_pbf"
        );

        // lat/lon check - note minor differences from internal granularity
        let lat_ok = (lat_found.unwrap_or_default() - 39.283).abs() < 0.0001;
        let lon_ok = (lon_found.unwrap_or_default() + 76.616).abs() < 0.0001;
        assert!(
            lat_ok && lon_ok,
            "Expected lat=39.283, lon=-76.616; found=({:?},{:?})",
            lat_found,
            lon_found
        );
    }

    #[tokio::test]
    async fn test_create_tiny_osm_pbf_success() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("tiny.osm.pbf");

        let result = create_tiny_osm_pbf(&path).await;
        assert!(result.is_ok(), "Creating tiny osm pbf should succeed");
        parse_and_check_pbf(&path);
    }

    #[tokio::test]
    async fn test_create_tiny_osm_pbf_path_is_directory() {
        let tmp = TempDir::new().unwrap();
        // The function will fail if we pass the directory instead of a file
        let dir_path = tmp.path();

        let result = create_tiny_osm_pbf(dir_path).await;
        assert!(result.is_err(), "Should fail if path is a directory");
        let err = result.err().unwrap();
        assert_eq!(err.kind(), ErrorKind::Other, "Directory => Other error kind");
    }

    #[tokio::test]
    async fn test_create_tiny_osm_pbf_unwritable_path() {
        // OS-specific scenario. On Unix, we can try e.g. "/root/some.pbf"
        // or an invalid path name on Windows. We'll do a partial approach:
        #[cfg(unix)]
        {
            let path = std::path::PathBuf::from("/this/is/not/writable/tiny.osm.pbf");
            let res = create_tiny_osm_pbf(&path).await;
            assert!(res.is_err(), "Unwritable path => should fail");
        }
    }
}
