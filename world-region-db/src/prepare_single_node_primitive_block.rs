// ---------------- [ File: src/prepare_single_node_primitive_block.rs ]
crate::ix!();

use crate::proto::{fileformat,osmformat};

use protobuf::MessageField;
use tracing::{trace, debug, warn};

/// Builds a `PrimitiveBlock` containing exactly one `Node` with:
///   - city (required)
///   - street (required)
///   - optional housenumber
///   - postal code (always)
/// Also sets lat/lon, granularity=100, etc.
pub fn prepare_single_node_primitive_block(
    city:        &str,
    street:      &str,
    postcode:    &str,
    housenumber: Option<&str>,
    lat_f64:     f64,
    lon_f64:     f64,
    node_id:     i64,
) -> osmformat::PrimitiveBlock {
    trace!("prepare_single_node_primitive_block: city={}, street={}, postcode={}, housenumber={:?}, lat={}, lon={}, node_id={}",
        city, street, postcode, housenumber, lat_f64, lon_f64, node_id
    );

    let mut block = osmformat::PrimitiveBlock::new();

    let mut stringtable = osmformat::StringTable::new();
    // index 0 => ""
    stringtable.s.push(b"".to_vec());

    // city => (1,2)
    stringtable.s.push(b"addr:city".to_vec()); // idx=1
    stringtable.s.push(city.as_bytes().to_vec()); // idx=2

    // street => (3,4)
    stringtable.s.push(b"addr:street".to_vec()); // idx=3
    stringtable.s.push(street.as_bytes().to_vec()); // idx=4

    // We track the next available index:
    let mut next_idx = 5;

    // We'll collect the node's (key,val) pairs in a small Vec
    let mut node_keyvals = vec![
        // city => (1,2), street => (3,4)
        (1, 2),
        (3, 4),
    ];

    // If housenumber is present, push it at (next_idx, next_idx+1)
    if let Some(hn) = housenumber {
        stringtable.s.push(b"addr:housenumber".to_vec()); // index=next_idx
        stringtable.s.push(hn.as_bytes().to_vec());       // index=next_idx+1
        node_keyvals.push((next_idx, next_idx + 1));
        next_idx += 2; // now increments by 2
    }

    // Next, push postcode => (next_idx, next_idx+1)
    stringtable.s.push(b"addr:postcode".to_vec()); // index=next_idx
    stringtable.s.push(postcode.as_bytes().to_vec()); // index=next_idx+1
    node_keyvals.push((next_idx, next_idx + 1));
    // next_idx += 2; (not strictly needed unless you add more strings after)

    // store final string table
    block.stringtable = MessageField::from_option(Some(stringtable));

    // set up the Node in a single group
    let mut group = osmformat::PrimitiveGroup::new();
    let mut node  = osmformat::Node::new();
    node.id = Some(node_id);

    block.set_granularity(100);
    block.set_lat_offset(0);
    block.set_lon_offset(0);

    // optional range checks
    if !( -90.0..=90.0 ).contains(&lat_f64) {
        warn!("prepare_single_node_primitive_block: latitude {} out of usual range", lat_f64);
    }
    if !( -180.0..=180.0 ).contains(&lon_f64) {
        warn!("prepare_single_node_primitive_block: longitude {} out of usual range", lon_f64);
    }

    // Convert lat/lon from float deg => “nano‐degrees / granularity”
    let lat_nano = (lat_f64 * 1e9) as i64;
    let lon_nano = (lon_f64 * 1e9) as i64;
    node.lat = Some(lat_nano / 100); 
    node.lon = Some(lon_nano / 100);

    // Add each pair to node.keys, node.vals
    for (k_idx, v_idx) in node_keyvals {
        node.keys.push(k_idx);
        node.vals.push(v_idx);
    }

    group.nodes.push(node);
    block.primitivegroup.push(group);

    debug!(
        "prepare_single_node_primitive_block: done, node_id={} => stringtable.len={}",
        node_id,
        block.stringtable.as_ref().unwrap().s.len()
    );
    block
}

#[cfg(test)]
mod test_prepare_single_node_primitive_block {
    use super::*;
    use crate::proto::osmformat; 
    use protobuf::Message; // For write_to_bytes/parse_from_bytes

    #[traced_test]
    fn test_no_housenumber_minimal_node() {
        let city     = "TestCity";
        let street   = "TestStreet";
        let postcode = "11111";
        let lat      = 39.283;
        let lon      = -76.616;
        let node_id  = 42;

        let block = prepare_single_node_primitive_block(
            city,
            street,
            postcode,
            None, // no housenumber
            lat,
            lon,
            node_id
        );

        // The `stringtable` field is a `MessageField<StringTable>`.
        // Access it via `.stringtable.as_ref()`.
        let str_table_opt = block.stringtable.as_ref();
        assert!(
            str_table_opt.is_some(),
            "Expected a populated stringtable"
        );
        let str_table = str_table_opt.unwrap();

        // If there's no housenumber, we expect 7 entries total:
        // 0 => ""
        // 1 => "addr:city"
        // 2 => city
        // 3 => "addr:street"
        // 4 => street
        // 5 => "addr:postcode"
        // 6 => postcode
        assert_eq!(
            str_table.s.len(),
            7,
            "String table size with no housenumber"
        );
        assert_eq!(&str_table.s[0], b"", "Empty string at index 0");
        assert_eq!(&str_table.s[1], b"addr:city");
        assert_eq!(&str_table.s[2], city.as_bytes());
        assert_eq!(&str_table.s[3], b"addr:street");
        assert_eq!(&str_table.s[4], street.as_bytes());
        assert_eq!(&str_table.s[5], b"addr:postcode");
        assert_eq!(&str_table.s[6], postcode.as_bytes());

        // The `primitivegroup` field is a `RepeatedField<PrimitiveGroup>`.
        // Access it via `&block.primitivegroup`.
        let groups = &block.primitivegroup;
        assert_eq!(groups.len(), 1, "Should have 1 primitive group");
        let group = &groups[0];

        let nodes = &group.nodes;
        assert_eq!(nodes.len(), 1, "Should have 1 node in the group");
        let node = &nodes[0];

        // Check node ID
        assert_eq!(node.id.unwrap_or_default(), node_id);

        // lat/lon scaling check
        let expected_lat = (lat * 1e9) as i64 / 100;
        let expected_lon = (lon * 1e9) as i64 / 100;
        assert_eq!(node.lat.unwrap_or_default(), expected_lat);
        assert_eq!(node.lon.unwrap_or_default(), expected_lon);

        // The node's keys/vals. For no housenumber, we appended:
        //   city => (1,2), street => (3,4), postcode => (5,6)
        let keys = &node.keys;
        let vals = &node.vals;
        assert_eq!(keys, &[1, 3, 5], "Should have city/street/postcode keys");
        assert_eq!(vals, &[2, 4, 6], "Should have city/street/postcode vals");
    }

    #[traced_test]
    fn test_with_housenumber() {
        let city        = "TestCity";
        let street      = "TestStreet";
        let postcode    = "11111";
        let housenumber = Some("100-110");
        let lat         = 40.0;
        let lon         = -75.0;
        let node_id     = 1001;

        let block = prepare_single_node_primitive_block(
            city,
            street,
            postcode,
            housenumber,
            lat,
            lon,
            node_id
        );

        let str_table_opt = block.stringtable.as_ref();
        assert!(str_table_opt.is_some());
        let str_table = str_table_opt.unwrap();

        // We expect 9 entries if we have housenumber:
        // 0 => ""
        // 1 => "addr:city"
        // 2 => city
        // 3 => "addr:street"
        // 4 => street
        // 5 => "addr:housenumber"
        // 6 => housenumber
        // 7 => "addr:postcode"
        // 8 => postcode
        assert_eq!(str_table.s.len(), 9);
        assert_eq!(&str_table.s[5], b"addr:housenumber");
        assert_eq!(&str_table.s[6], housenumber.unwrap().as_bytes());

        let groups = &block.primitivegroup;
        assert_eq!(groups.len(), 1);
        let node = &groups[0].nodes[0];
        assert_eq!(node.id.unwrap_or_default(), node_id);

        let keys = &node.keys;
        let vals = &node.vals;
        // We appended city => (1,2), street => (3,4), housenumber => (5,6), then postcode => (7,8)
        assert_eq!(*keys, [1, 3, 5, 7]);
        assert_eq!(*vals, [2, 4, 6, 8]);
    }

    #[traced_test]
    fn test_lat_lon_out_of_range_logs_warning() {
        let block = prepare_single_node_primitive_block(
            "City",
            "Street",
            "11111",
            None,
            95.0,  // out of normal lat range
            185.0, // out of normal lon range
            999
        );
        let node = &block.primitivegroup[0].nodes[0];
        let expected_lat = (95.0 * 1e9) as i64 / 100;
        let expected_lon = (185.0 * 1e9) as i64 / 100;
        assert_eq!(node.lat.unwrap_or_default(), expected_lat);
        assert_eq!(node.lon.unwrap_or_default(), expected_lon);
    }

    #[traced_test]
    fn test_stringtable_indices_order() {
        // Confirm stable ordering:
        //  0 => ""
        //  1 => "addr:city"
        //  2 => city
        //  3 => "addr:street"
        //  4 => street
        //  5 => "addr:housenumber"
        //  6 => housenumber
        //  7 => "addr:postcode"
        //  8 => postcode
        let block = prepare_single_node_primitive_block(
            "CITY",
            "STREET",
            "11111",
            Some("111"),
            0.0,
            0.0,
            777
        );
        let st = block.stringtable.as_ref().unwrap();
        // Fix the missing comma between b"111" and b"addr:postcode":
        let expected: Vec<&[u8]> = vec![
            b"",
            b"addr:city",
            b"CITY",
            b"addr:street",
            b"STREET",
            b"addr:housenumber",
            b"111",
            b"addr:postcode",
            b"11111"
        ];
        assert_eq!(st.s.len(), expected.len());

        for (i, &exp_bytes) in expected.iter().enumerate() {
            assert_eq!(&st.s[i], exp_bytes, "Mismatch at index {}", i);
        }
    }

    #[traced_test]
    fn test_serialization_and_reparse() {
        let block = prepare_single_node_primitive_block(
            "Baltimore",
            "Main St",
            "11111",
            Some("200"),
            39.28,
            -76.61,
            555
        );
        let bytes = block.write_to_bytes().expect("Should serialize");
        let parsed: osmformat::PrimitiveBlock =
            protobuf::Message::parse_from_bytes(&bytes).expect("Should deserialize");

        // We never had a `get_granularity()` method; we can just check the field:
        assert_eq!(parsed.granularity.unwrap_or_default(), 100);

        // There's exactly one group, one node
        let node = &parsed.primitivegroup[0].nodes[0];
        assert_eq!(node.id.unwrap_or_default(), 555);

        let expected_lat = (39.28 * 1e9) as i64 / 100;
        let expected_lon = (-76.61 * 1e9) as i64 / 100;
        assert_eq!(node.lat.unwrap_or_default(), expected_lat);
        assert_eq!(node.lon.unwrap_or_default(), expected_lon);

        // Check the stringtable
        let st = parsed.stringtable.as_ref().unwrap();
        assert_eq!(&st.s[1], b"addr:city");
        assert_eq!(&st.s[2], b"Baltimore");
        assert_eq!(&st.s[3], b"addr:street");
        assert_eq!(&st.s[4], b"Main St");
        assert_eq!(&st.s[5], b"addr:housenumber");
        assert_eq!(&st.s[6], b"200");
        assert_eq!(&st.s[7], b"addr:postcode");
        assert_eq!(&st.s[8], b"11111");

        let keys = &node.keys;
        let vals = &node.vals;
        assert_eq!(keys, &[1, 3, 5, 7]);
        assert_eq!(vals, &[2, 4, 6, 8]);
    }
}
