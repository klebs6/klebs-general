// ---------------- [ File: src/mock_node.rs ]
crate::ix!();

use crate::proto::{osmformat, fileformat};

use std::io::Cursor;

// Disambiguate the byteorder trait methods so we don't conflict with tokio's `AsyncWriteExt`.
use byteorder::{BigEndian, WriteBytesExt as ByteOrderWriteExt};

use osmpbf::{
    BlobReader,
    BlobType,
    Element,
    PrimitiveBlock,
};

/// A helper struct for creating a single mocked Node in an `.osm.pbf`-like byte array.
/// That array is then parsed by `osmpbf::BlobReader`, producing a `PrimitiveBlock` you can
/// retrieve as `MockNode.block`.
///
/// The fix here: We ensure the `PrimitiveBlock` is fully initialized 
/// (granularity, lat_offset, lon_offset) and each Node has lat/lon fields set (even if 0).
///
/// If your code requires *real* lat/lon, change those from 0 to something valid.
#[derive(Debug)]
pub struct MockNode {
    block: PrimitiveBlock,
}

impl MockNode {
    /// Creates a single mocked Node with the given `id` and `(key,val)` tags.
    ///
    /// We fix the `MessageNotInitialized("PrimitiveBlock")` by:
    ///  - Setting `block.set_granularity(...)`
    ///  - Setting `block.set_lat_offset(...)`
    ///  - Setting `block.set_lon_offset(...)`
    ///  - Setting `node.lat` / `node.lon`
    ///  - Using minimal but valid string table references
    ///
    pub fn new(id: i64, tags: &[(&str, &str)]) -> Self {
        // 1) Build a minimal `.osm.pbf` in memory
        let data = build_mock_node_pbf_bytes(id, tags);

        // 2) Parse the array via osmpbf::BlobReader
        let mut reader = BlobReader::new(Cursor::new(data));
        while let Some(blob_res) = reader.next() {
            let blob = blob_res.expect("Error reading mock data blob");
            if let BlobType::OsmData = blob.get_type() {
                let block = blob
                    .to_primitiveblock()
                    .expect("Cannot decode mock data to PrimitiveBlock");
                return Self { block };
            }
        }
        panic!("No OSMData blob found in the mock data!");
    }

    /// Returns this single mocked node as `osmpbf::Element::Node(...)`.
    /// Valid while `MockNode` is in scope, since the `PrimitiveBlock` is owned here.
    pub fn as_element(&self) -> Element<'_> {
        self.block
            .elements()
            .next()
            .expect("PrimitiveBlock is empty? Should have 1 node.")
    }
}

/// Builds a minimal `.osm.pbf` byte array for a single Node with the given `id` and tags.
/// That byte array includes exactly one BlobHeader + Blob with type "OSMData".
fn build_mock_node_pbf_bytes(id: i64, tags: &[(&str, &str)]) -> Vec<u8> {
    // 1) Prepare a string table with all unique keys/values in `tags`.
    //    We'll store them in order: [ "", key1, val1, key2, val2, ... ]
    //    Then node.keys / node.vals are indices into that.
    let mut stringtable = osmformat::StringTable::new();
    let mut st_index_map = Vec::new(); // (k,v) => (k_idx, v_idx)

    // index 0 is always the empty string:
    stringtable.s.push(b"".to_vec());

    for &(k, v) in tags {
        let k_idx = stringtable.s.len() as u32;
        stringtable.s.push(k.as_bytes().to_vec());
        let v_idx = stringtable.s.len() as u32;
        stringtable.s.push(v.as_bytes().to_vec());
        st_index_map.push((k_idx, v_idx));
    }

    // 2) Create a Node
    let mut node = osmformat::Node::new();
    node.set_id(id);
    // lat/lon must be set for the node to be considered "initialized".
    // We'll do 0 for both. If you need real coords, convert them as needed.
    node.set_lat(0); 
    node.set_lon(0);

    // Fill out node.keys/vals from st_index_map
    for (k_i, v_i) in &st_index_map {
        node.keys.push(*k_i);
        node.vals.push(*v_i);
    }

    // 3) Put that Node in a PrimitiveGroup
    let mut group = osmformat::PrimitiveGroup::new();
    group.nodes.push(node);

    // 4) Build the PrimitiveBlock:
    let mut prim_block = osmformat::PrimitiveBlock::new();
    // The fix: set granularity & offsets so the block is "initialized".
    prim_block.set_granularity(100);
    prim_block.set_lat_offset(0);
    prim_block.set_lon_offset(0);
    prim_block.stringtable = protobuf::MessageField::some(stringtable);
    prim_block.primitivegroup.push(group);

    // 5) Serialize this block into a `fileformat::Blob`
    let block_bytes = prim_block
        .write_to_bytes()
        .expect("Could not serialize local PrimitiveBlock");

    let mut blob = fileformat::Blob::new();
    blob.set_raw(block_bytes);
    blob.set_raw_size(blob.raw().len() as i32);

    let blob_bytes = blob
        .write_to_bytes()
        .expect("Could not serialize local Blob");

    // 6) Build a BlobHeader with type = "OSMData"
    let mut header = fileformat::BlobHeader::new();
    header.set_type("OSMData".to_string());
    header.set_datasize(blob_bytes.len() as i32);

    let header_bytes = header
        .write_to_bytes()
        .expect("Could not serialize BlobHeader");

    // 7) Frame as <4-byte header-len><header><blob>
    use byteorder::{BigEndian, WriteBytesExt};
    let mut out = Vec::new();
    byteorder::WriteBytesExt::write_u32::<BigEndian>(&mut out, header_bytes.len() as u32);
    out.extend_from_slice(&header_bytes);
    out.extend_from_slice(&blob_bytes);

    out
}

#[cfg(test)]
mod test_mock_node {
    use super::*;
    use osmpbf::Element;

    #[test]
    fn test_creates_mock_node() {
        let mk = MockNode::new(2023, &[
            ("amenity", "post_box"), 
            ("color", "red"),
        ]);
        let elem = mk.as_element();

        if let Element::Node(n) = elem {
            assert_eq!(n.id(), 2023);

            let tags: Vec<_> = n.tags().collect();
            assert_eq!(tags, vec![
                ("amenity", "post_box"),
                ("color", "red"),
            ]);
        } else {
            panic!("Expected Element::Node");
        }
    }
}
