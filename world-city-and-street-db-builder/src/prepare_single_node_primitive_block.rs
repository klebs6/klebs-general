// ---------------- [ File: src/prepare_single_node_primitive_block.rs ]
crate::ix!();

use crate::proto::{fileformat,osmformat};

/// Builds a `PrimitiveBlock` containing exactly one `Node` with optional `addr:housenumber`.
/// Also sets lat/lon, granularity, etc.
pub fn prepare_single_node_primitive_block(
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
