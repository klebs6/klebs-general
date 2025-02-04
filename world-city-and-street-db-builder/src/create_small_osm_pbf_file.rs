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
    path: &Path,
    bbox: (i64, i64, i64, i64),
    city: &str,
    street: &str,
    housenumber: Option<&str>,
    lat: f64,
    lon: f64,
    node_id: i64,
) -> std::io::Result<()> {
    trace!(
        "create_small_osm_pbf_file: invoked for path={:?}, node_id={}, city={}, street={}, housenumber={:?}, lat={}, lon={}",
        path, node_id, city, street, housenumber, lat, lon
    );

    // Ensure path is suitable for file creation
    validate_not_dir(path)?;

    // 1) Prepare OSMHeader block & serialize
    let header_block = prepare_osm_header_block(bbox);
    let (header_blobheader_bytes, header_blob_bytes) =
        serialize_osm_header_block(header_block)?;

    // 2) Prepare PrimitiveBlock with a single node & optional housenumber, then serialize
    let primitive_block = prepare_single_node_primitive_block(city, street, housenumber, lat, lon, node_id);
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
