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
