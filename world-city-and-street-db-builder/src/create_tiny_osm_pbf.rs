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

/// Ensures the path does not point to an existing directory.
fn validate_not_dir(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        let msg = format!("Refusing to create file at {:?}, path is a directory", path);
        error!("{}", msg);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, msg));
    }
    Ok(())
}

/// Builds an OSM header block with given bounding box and required features.
fn prepare_osm_header_block(bbox: (i64, i64, i64, i64)) -> osmformat::HeaderBlock {
    trace!("prepare_osm_header_block: using bbox={:?}", bbox);

    let (left, right, top, bottom) = bbox;
    let mut headerblock = osmformat::HeaderBlock::new();

    let mut hbbox = osmformat::HeaderBBox::new();
    hbbox.set_left(left);
    hbbox.set_right(right);
    hbbox.set_top(top);
    hbbox.set_bottom(bottom);

    headerblock.bbox = protobuf::MessageField::from_option(Some(hbbox));
    headerblock.required_features.push("OsmSchema-V0.6".to_string());
    headerblock.required_features.push("DenseNodes".to_string());

    debug!("prepare_osm_header_block: HeaderBlock created");
    headerblock
}

/// Serializes the given `HeaderBlock` into a `Blob` and `BlobHeader`.
fn serialize_osm_header_block(
    header_block: osmformat::HeaderBlock
) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    trace!("serialize_osm_header_block: serializing HeaderBlock");

    let header_block_bytes = header_block.write_to_bytes().map_err(|e| {
        error!("serialize_osm_header_block: protobuf error: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "HeaderBlock serialization failed")
    })?;

    let mut blob = fileformat::Blob::new();
    blob.set_raw(header_block_bytes.clone());
    blob.set_raw_size(header_block_bytes.len() as i32);

    let blob_bytes = blob.write_to_bytes().map_err(|e| {
        error!("serialize_osm_header_block: blob error: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Blob serialization failed")
    })?;

    let mut blob_header = fileformat::BlobHeader::new();
    blob_header.set_type("OSMHeader".to_string());
    blob_header.set_datasize(blob_bytes.len() as i32);

    let blob_header_bytes = blob_header.write_to_bytes().map_err(|e| {
        error!("serialize_osm_header_block: blob header error: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "BlobHeader serialization failed")
    })?;

    debug!("serialize_osm_header_block: Blob and BlobHeader ready");
    Ok((blob_header_bytes, blob_bytes))
}

/// Builds a `PrimitiveBlock` containing exactly one `Node` with optional `addr:housenumber`.
/// Also sets lat/lon, granularity, etc.
fn prepare_single_node_primitive_block(
    city: &str,
    street: &str,
    housenumber: Option<&str>,
    lat_f64: f64,
    lon_f64: f64,
    node_id: i64,
) -> osmformat::PrimitiveBlock {
    trace!("prepare_single_node_primitive_block: city={}, street={}, housenumber={:?}, lat={}, lon={}", 
           city, street, housenumber, lat_f64, lon_f64);

    let mut block = osmformat::PrimitiveBlock::new();

    // 1) Build string table
    let mut stringtable = osmformat::StringTable::new();
    // index 0 => ""
    // index 1 => "addr:city"
    // index 2 => city
    // index 3 => "addr:street"
    // index 4 => street
    // if housenumber is Some(...), we'll add at index 5 => "addr:housenumber" and index 6 => housenumber
    stringtable.s.push(b"".to_vec());
    stringtable.s.push(b"addr:city".to_vec());
    stringtable.s.push(city.as_bytes().to_vec());
    stringtable.s.push(b"addr:street".to_vec());
    stringtable.s.push(street.as_bytes().to_vec());

    let mut node_keyvals = Vec::new();

    // city => city
    node_keyvals.push((1, 2)); // (key=1 => "addr:city", val=2 => city)

    let mut string_index = 5; 
    if let Some(hn) = housenumber {
        stringtable.s.push(b"addr:housenumber".to_vec()); // index=5
        stringtable.s.push(hn.as_bytes().to_vec());       // index=6
        node_keyvals.push((5, 6));
        string_index = 7; // next index if we had more
    }

    // street => street
    // We'll place them after we finalize string indexes for housenumber (to keep consistent ordering).
    // Actually we already have "addr:street" => index=3, the street string => index=4
    // We'll push that to node_keyvals last to keep a consistent grouping
    node_keyvals.push((3, 4));

    block.stringtable = protobuf::MessageField::from_option(Some(stringtable));

    // 2) Node config
    let mut group = osmformat::PrimitiveGroup::new();
    let mut node = osmformat::Node::new();
    node.set_id(node_id);

    block.set_granularity(100);
    block.set_lat_offset(0);
    block.set_lon_offset(0);

    // Bounds checking
    if lat_f64 < -90.0 || lat_f64 > 90.0 {
        warn!("prepare_single_node_primitive_block: latitude {} out of usual range", lat_f64);
    }
    if lon_f64 < -180.0 || lon_f64 > 180.0 {
        warn!("prepare_single_node_primitive_block: longitude {} out of usual range", lon_f64);
    }

    let lat_nano = (lat_f64 * 1e9) as i64;
    let lon_nano = (lon_f64 * 1e9) as i64;
    node.set_lat(lat_nano / 100); 
    node.set_lon(lon_nano / 100);

    // For each (k, v) in node_keyvals => node.keys.push(k), node.vals.push(v)
    for (k_idx, v_idx) in node_keyvals {
        node.keys.push(k_idx);
        node.vals.push(v_idx);
    }

    group.nodes.push(node);
    block.primitivegroup.push(group);

    debug!("prepare_single_node_primitive_block: PrimitiveBlock with node_id={} created", node_id);
    block
}

/// Serializes a [`PrimitiveBlock`] into a `Blob` and `BlobHeader`.
fn serialize_primitive_block(
    primitive_block: osmformat::PrimitiveBlock
) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    trace!("serialize_primitive_block: converting PrimitiveBlock to Blob + BlobHeader");

    let block_bytes = primitive_block.write_to_bytes().map_err(|e| {
        error!("serialize_primitive_block: protobuf error: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "PrimitiveBlock serialization failed")
    })?;

    let mut blob = fileformat::Blob::new();
    blob.set_raw(block_bytes.clone());
    blob.set_raw_size(block_bytes.len() as i32);

    let blob_bytes = blob.write_to_bytes().map_err(|e| {
        error!("serialize_primitive_block: Blob serialization error: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Data Blob serialization failed")
    })?;

    let mut blob_header = fileformat::BlobHeader::new();
    blob_header.set_type("OSMData".to_string());
    blob_header.set_datasize(blob_bytes.len() as i32);

    let blob_header_bytes = blob_header.write_to_bytes().map_err(|e| {
        error!("serialize_primitive_block: BlobHeader serialization error: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "BlobHeader serialization failed")
    })?;

    debug!("serialize_primitive_block: Blob + BlobHeader ready");
    Ok((blob_header_bytes, blob_bytes))
}

/// Asynchronously writes two sets of BlobHeader/Blob pairs
/// (header vs. data) to the target file in `.osm.pbf` order.
pub async fn write_osm_pbf_file(
    path: &Path,
    header_blobheader_bytes: &[u8],
    header_blob_bytes: &[u8],
    data_blobheader_bytes: &[u8],
    data_blob_bytes: &[u8]
) -> std::io::Result<()> {
    trace!("write_osm_pbf_file: creating file at {:?}", path);

    let mut file = match tokio::fs::File::create(path).await {
        Ok(f) => {
            debug!("write_osm_pbf_file: file opened at {:?}", path);
            f
        }
        Err(e) => {
            error!("write_osm_pbf_file: failed to create file {:?}: {:?}", path, e);
            return Err(e);
        }
    };

    // Write the OSMHeader portion
    trace!(
        "write_osm_pbf_file: writing header_blobheader={} bytes + header_blob={} bytes",
        header_blobheader_bytes.len(),
        header_blob_bytes.len()
    );
    crate::write_u32_be(&mut file, header_blobheader_bytes.len() as u32).await?;
    file.write_all(header_blobheader_bytes).await?;
    file.write_all(header_blob_bytes).await?;

    // Write the OSMData portion
    trace!(
        "write_osm_pbf_file: writing data_blobheader={} bytes + data_blob={} bytes",
        data_blobheader_bytes.len(),
        data_blob_bytes.len()
    );
    crate::write_u32_be(&mut file, data_blobheader_bytes.len() as u32).await?;
    file.write_all(data_blobheader_bytes).await?;
    file.write_all(data_blob_bytes).await?;

    debug!("write_osm_pbf_file: completed writing to {:?}", path);
    Ok(())
}
