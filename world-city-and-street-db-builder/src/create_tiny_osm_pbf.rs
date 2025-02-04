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
