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

#[cfg(test)]
#[disable]
mod test_prepare_single_node_primitive_block {
    use super::*;
    use crate::proto::osmformat; // Adjust if your osmformat module is located elsewhere
    use protobuf::Message;        // For optional serialization checks

    #[test]
    fn test_no_housenumber_minimal_node() {
        let city = "TestCity";
        let street = "TestStreet";
        let lat = 39.283;
        let lon = -76.616;
        let node_id = 42;

        let block = prepare_single_node_primitive_block(
            city,
            street,
            None, // no housenumber
            lat,
            lon,
            node_id
        );

        // Check string table
        let str_table = block.get_stringtable();
        // Expect 5 entries: ["", "addr:city", city, "addr:street", street]
        assert_eq!(str_table.s.len(), 5, "String table size with no housenumber");

        assert_eq!(&str_table.s[0], b"", "Empty string at index 0");
        assert_eq!(&str_table.s[1], b"addr:city");
        assert_eq!(&str_table.s[2], city.as_bytes());
        assert_eq!(&str_table.s[3], b"addr:street");
        assert_eq!(&str_table.s[4], street.as_bytes());

        // There's exactly one PrimitiveGroup
        let groups = block.get_primitivegroup();
        assert_eq!(groups.len(), 1, "Should have 1 primitive group");
        let group = &groups[0];

        // The group should have exactly one node
        let nodes = group.get_nodes();
        assert_eq!(nodes.len(), 1, "Should have 1 node in the group");
        let node = &nodes[0];

        // Check node id
        assert_eq!(node.get_id(), node_id);

        // Check lat/lon scaling
        // lat_nano = (39.283 * 1e9) = 39283000000 => dividing by 100 => 392830000
        let expected_lat = (lat * 1e9) as i64 / 100;
        let expected_lon = (lon * 1e9) as i64 / 100;
        assert_eq!(node.get_lat(), expected_lat);
        assert_eq!(node.get_lon(), expected_lon);

        // The node's keys/vals: we inserted (addr:city => city), (addr:street => street)
        //  => keys: [1, 3], vals: [2, 4] if no housenumber
        let keys = node.get_keys();
        let vals = node.get_vals();
        // Because we inserted city => (1,2) first, then street => (3,4)
        assert_eq!(keys, [1, 3], "Should have exactly two keys for city & street");
        assert_eq!(vals, [2, 4], "Values should match the city & street strings");
    }

    #[test]
    fn test_with_housenumber() {
        let city = "TestCity";
        let street = "TestStreet";
        let housenumber = Some("100-110");
        let lat = 40.0;
        let lon = -75.0;
        let node_id = 1001;

        let block = prepare_single_node_primitive_block(
            city,
            street,
            housenumber,
            lat,
            lon,
            node_id
        );

        // Check string table
        let str_table = block.get_stringtable();
        // Expect 7 entries: ["", "addr:city", city, "addr:street", street, "addr:housenumber", housenumber]
        assert_eq!(str_table.s.len(), 7, "String table size with housenumber");
        assert_eq!(&str_table.s[5], b"addr:housenumber");
        assert_eq!(&str_table.s[6], housenumber.unwrap().as_bytes());

        // Node
        let groups = block.get_primitivegroup();
        assert_eq!(groups.len(), 1, "Should have 1 primitive group with the node");
        let node = &groups[0].get_nodes()[0];
        assert_eq!(node.get_id(), node_id);

        // The node's keys/vals now includes city => (1,2), housenumber => (5,6), street => (3,4)
        let keys = node.get_keys();
        let vals = node.get_vals();
        assert_eq!(keys, [1, 5, 3], "City first, then housenumber, then street");
        assert_eq!(vals, [2, 6, 4], "Paired with city, housenumber, street strings");
    }

    #[test]
    fn test_lat_lon_out_of_range_logs_warning() {
        // We'll just confirm it doesn't panic, but the logs should contain a warning. 
        // We can't check logs easily in a simple test, but let's ensure it doesn't break.
        let block = prepare_single_node_primitive_block(
            "City",
            "Street",
            None,
            95.0, // out of normal lat range
            185.0, // out of normal lon range
            999
        );
        // Check that it still sets the lat/lon
        let node = &block.get_primitivegroup()[0].get_nodes()[0];
        let expected_lat = (95.0 * 1e9) as i64 / 100;
        let expected_lon = (185.0 * 1e9) as i64 / 100;
        assert_eq!(node.get_lat(), expected_lat);
        assert_eq!(node.get_lon(), expected_lon);
    }

    #[test]
    fn test_stringtable_indices_order() {
        // Confirm that the order is stable: 
        //  0 => ""
        //  1 => "addr:city"
        //  2 => city
        //  3 => "addr:street"
        //  4 => street
        //  5 => "addr:housenumber"
        //  6 => housenumber
        let block = prepare_single_node_primitive_block(
            "CITY",
            "STREET",
            Some("111"),
            0.0,
            0.0,
            777
        );
        let st = block.get_stringtable();
        let expect = vec![
            b"",
            b"addr:city",
            b"CITY",
            b"addr:street",
            b"STREET",
            b"addr:housenumber",
            b"111"
        ];
        for (i, &expected_bytes) in expect.iter().enumerate() {
            assert_eq!(&st.s[i], expected_bytes);
        }
    }

    #[test]
    fn test_serialization_and_reparse() {
        let block = prepare_single_node_primitive_block(
            "Baltimore",
            "Main St",
            Some("200"),
            39.28,
            -76.61,
            555
        );
        // We can do a round-trip test: serialize to bytes, then parse back.
        let bytes = block.write_to_bytes().expect("Should serialize");
        let parsed: osmformat::PrimitiveBlock =
            protobuf::Message::parse_from_bytes(&bytes).expect("Should deserialize");
        // Check we got the node_id, lat/lon, etc.
        assert_eq!(parsed.get_granularity(), 100);
        // There's exactly one primitive group, with one node
        let node = parsed.get_primitivegroup()[0].get_nodes()[0].clone();
        assert_eq!(node.get_id(), 555);

        // lat, lon
        let expected_lat = (39.28 * 1e9) as i64 / 100;
        let expected_lon = (-76.61 * 1e9) as i64 / 100;
        assert_eq!(node.get_lat(), expected_lat);
        assert_eq!(node.get_lon(), expected_lon);

        // Now check the string table and the node's key-val pairs
        let st = parsed.get_stringtable();
        // In principle we expect 7 entries, but let's just confirm the main ones
        assert_eq!(&st.s[1], b"addr:city");
        assert_eq!(&st.s[2], b"Baltimore");
        assert_eq!(&st.s[3], b"addr:street");
        assert_eq!(&st.s[4], b"Main St");
        assert_eq!(&st.s[5], b"addr:housenumber");
        assert_eq!(&st.s[6], b"200");
        // Node's key/val arrays
        let keys = node.get_keys();
        let vals = node.get_vals();
        // Should be [1,5,3], [2,6,4] for city, housenumber, street
        assert_eq!(keys, [1,5,3], "Key indices for city, then housenumber, then street");
        assert_eq!(vals, [2,6,4]);
    }
}
