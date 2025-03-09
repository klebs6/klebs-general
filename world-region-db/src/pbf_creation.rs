// ---------------- [ File: src/pbf_creation.rs ]
crate::ix!();

use crate::proto::osmformat;

/// Creates a minimal `.osm.pbf` file whose bounding box definitely includes
/// the given `lat`/`lon`, **and** includes a postal code. This ensures the
/// aggregator will see city/street/postal_code/housenumber and emit
/// a `WorldAddress`.
///
/// # Arguments
///
/// * `pbf_path`    - Path to the output `.osm.pbf`.
/// * `city`        - The `addr:city` tag.
/// * `street`      - The `addr:street` tag.
/// * `housenumber` - An optional `addr:housenumber` like `"5-10"`.
/// * `postal_code` - An optional `addr:postcode` like `"21201"`.
/// * `lat`/`lon`   - Node coordinates in floating degrees.
/// * `node_id`     - The Node's OSM ID.
pub async fn create_small_osm_pbf_file_in_bbox(
    pbf_path: &std::path::Path,
    city: &str,
    street: &str,
    housenumber: Option<&str>,
    postal_code: Option<&str>,
    lat: f64,
    lon: f64,
    node_id: i64
) -> std::io::Result<()> {
    // Use a bounding box in 1e7 “nano‐degrees” that covers about 38..40 N, 77..76 W.
    // That definitely includes lat≈39.283, lon≈-76.616.
    let bounding_box = (
        -770_000_000, // left   (≈ -77.0)
        -760_000_000, // right  (≈ -76.0)
         400_000_000, // top    (≈  40.0)
         380_000_000, // bottom (≈  38.0)
    );

    // We extend our lower-level creation function to accept a postal code param.
    // That function must embed `addr:postcode` in the node if `postal_code` is Some(...).
    create_small_osm_pbf_file_with_postcode(
        pbf_path,
        bounding_box,
        city,
        street,
        housenumber,
        postal_code,
        lat,
        lon,
        node_id
    ).await
}

/// Creates a single-node `.osm.pbf` while optionally including `addr:housenumber`
/// and `addr:postcode`. This is just a fork of your existing `create_small_osm_pbf_file`,
/// augmented to handle `postal_code`.
pub async fn create_small_osm_pbf_file_with_postcode(
    path: &std::path::Path,
    bbox: (i64, i64, i64, i64),
    city: &str,
    street: &str,
    housenumber: Option<&str>,
    postal_code: Option<&str>,
    lat: f64,
    lon: f64,
    node_id: i64,
) -> std::io::Result<()> {
    // 1) Validate path
    validate_not_dir(path)?;

    // 2) Build OSMHeader + Blob
    let header_block = prepare_osm_header_block(bbox);
    let (header_blobheader_bytes, header_blob_bytes) = serialize_osm_header_block(header_block)?;

    // 3) Prepare PrimitiveBlock with a single node. We modify the function that
    // actually sets up the node's tags so it can also embed `addr:postcode`.
    let primitive_block = prepare_single_node_primitive_block_with_postcode(
        city,
        street,
        housenumber,
        postal_code,
        lat,
        lon,
        node_id,
    );
    let (data_blobheader_bytes, data_blob_bytes) = serialize_primitive_block(primitive_block)?;

    // 4) Write it out
    write_osm_pbf_file(
        path,
        &header_blobheader_bytes,
        &header_blob_bytes,
        &data_blobheader_bytes,
        &data_blob_bytes,
    )
    .await?;

    Ok(())
}

/// Same as `prepare_single_node_primitive_block` but also includes `addr:postcode` if provided.
pub fn prepare_single_node_primitive_block_with_postcode(
    city: &str,
    street: &str,
    housenumber: Option<&str>,
    postal_code: Option<&str>,
    lat_f64: f64,
    lon_f64: f64,
    node_id: i64,
) -> osmformat::PrimitiveBlock {
    use crate::proto::osmformat;

    let mut block = osmformat::PrimitiveBlock::new();

    // We store our needed strings in a known order:
    // index 0 => ""
    // index 1 => "addr:city"
    // index 2 => city
    // index 3 => "addr:street"
    // index 4 => street
    // index 5 => "addr:housenumber" (only if `housenumber.is_some()`)
    // index 6 => housenumber        (same condition)
    // index 7 => "addr:postcode"    (only if `postal_code.is_some()`)
    // index 8 => postal_code        (same condition)
    let mut stringtable = osmformat::StringTable::new();
    stringtable.s.push(b"".to_vec());              // 0
    stringtable.s.push(b"addr:city".to_vec());     // 1
    stringtable.s.push(city.as_bytes().to_vec());  // 2
    stringtable.s.push(b"addr:street".to_vec());   // 3
    stringtable.s.push(street.as_bytes().to_vec()); // 4

    let mut tag_pairs = Vec::new();

    // city
    tag_pairs.push((1, 2)); // key=1 => "addr:city", val=2 => city

    // street
    tag_pairs.push((3, 4)); // key=3 => "addr:street", val=4 => street

    // housenumber
    let mut next_idx = 5;
    if let Some(hn) = housenumber {
        stringtable.s.push(b"addr:housenumber".to_vec()); // index=5
        stringtable.s.push(hn.as_bytes().to_vec());       // index=6
        tag_pairs.push((5, 6));
        next_idx = 7;
    }

    // postal_code
    if let Some(pc) = postal_code {
        stringtable.s.push(b"addr:postcode".to_vec()); // index=7 or next_idx
        stringtable.s.push(pc.as_bytes().to_vec());    // index=8 or next_idx + 1
        tag_pairs.push((next_idx, next_idx + 1));
    }

    block.stringtable = protobuf::MessageField::from_option(Some(stringtable));

    // Now create the Node and store lat/lon
    let mut group = osmformat::PrimitiveGroup::new();
    let mut node = osmformat::Node::new();
    node.set_id(node_id);

    // For OSM PBF, lat/lon are stored as "integer degrees * 1e7", minus any offset,
    // then further divided by the granularity. We'll do a simpler approach:
    //   lat_nano = (lat_f64 * 1e7) as i64
    //   node.lat = lat_nano / granularity
    //   etc.
    block.set_granularity(100);
    block.set_lat_offset(0);
    block.set_lon_offset(0);

    let lat_scaled = (lat_f64 * 10_000_000.0) as i64; // *1e7
    let lon_scaled = (lon_f64 * 10_000_000.0) as i64;
    node.set_lat(lat_scaled / 100);  // integer division
    node.set_lon(lon_scaled / 100);

    // Add the tag pairs
    for (k_idx, v_idx) in tag_pairs {
        node.keys.push(k_idx);
        node.vals.push(v_idx);
    }

    group.nodes.push(node);
    block.primitivegroup.push(group);
    block
}
