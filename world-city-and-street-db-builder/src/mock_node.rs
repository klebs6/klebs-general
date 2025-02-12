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

#[derive(Debug)]
pub struct MockNode {
    block: PrimitiveBlock,
}

impl MockNode {
    pub fn new(id: i64, tags: &[(&str, &str)]) -> Self {
        // 1) Build a minimal PBF byte array with one Node
        let data = build_mock_node_pbf_bytes(id, tags);

        // 2) Parse that array via osmpbf::BlobReader
        let mut reader = BlobReader::new(Cursor::new(data));

        while let Some(blob_res) = reader.next() {
            let blob = blob_res.expect("Error reading mock data blob");

            if let BlobType::OsmData = blob.get_type() {
                // 3) Decode to an osmpbf::PrimitiveBlock
                let block = blob.to_primitiveblock()
                    .expect("Cannot decode to PrimitiveBlock");
                return Self { block };
            }
        }
        panic!("No OSMData blob found in the mock data!");
    }

    /// Return our single mocked node as `Element::Node(...)`.
    /// Valid only while `MockNode` is in scope.
    pub fn as_element(&self) -> Element<'_> {
        self.block
            .elements()
            .next()
            .expect("PrimitiveBlock is empty? Should have 1 node.")
    }
}

/// Build a valid `.osm.pbf` byte array in memory with exactly one Node having
/// `id` and the given `tags`.
fn build_mock_node_pbf_bytes(id: i64, tags: &[(&str, &str)]) -> Vec<u8> {
    // 1) Create your local `osmformat::Node`
    //    NOTE: If your codegen doesn't have `.new()`, try `Node::default()`.
    let mut node = osmformat::Node::new();
    node.id = Some(id); // or node.id = Some(id), or however your fields are shaped

    // You probably have a `Vec<Vec<u8>>` for the string table. Then `keys` and `vals`
    // are typically repeated integers (e.g. `Vec<u32>` or `Vec<u64>`).
    let mut stringtable = osmformat::StringTable::new();

    let mut keys: Vec<u32> = Vec::new();
    let mut vals: Vec<u32> = Vec::new();

    for &(k, v) in tags {
        let k_idx = stringtable.s.len() as u32;
        stringtable.s.push(k.as_bytes().to_vec());
        let v_idx = stringtable.s.len() as u32;
        stringtable.s.push(v.as_bytes().to_vec());

        keys.push(k_idx);
        vals.push(v_idx);
    }

    // Because `set_keys(...)` doesn’t exist in your generated code, do direct assignment:
    // e.g. node.keys = keys; node.vals = vals; 
    // Adjust the field names / types to match your code.
    node.keys = keys;
    node.vals = vals;

    // 2) Put that Node into a PrimitiveGroup, then into a PrimitiveBlock
    let mut group = osmformat::PrimitiveGroup::new();
    group.nodes.push(node);

    let mut prim_block = osmformat::PrimitiveBlock::new();

    // Possibly your code just has `prim_block.stringtable = Some(stringtable)`,
    // or it’s a direct field, or something else. If `mut_stringtable()` doesn't exist,
    // do direct assignment:
    prim_block.stringtable = Some(stringtable).into();
    // or `prim_block.stringtable = stringtable;`
    // depending on what your generated struct looks like.

    prim_block.primitivegroup.push(group);

    // 3) Serialize to bytes -> the raw field of a local `fileformat::Blob`
    let block_bytes = prim_block
        .write_to_bytes()
        .expect("Could not serialize local PrimitiveBlock");
    let mut blob = fileformat::Blob::new();
    blob.set_raw(block_bytes);
    // or: blob.set_raw(block_bytes);

    let blob_bytes = blob
        .write_to_bytes()
        .expect("Could not serialize local Blob");

    // 4) Build a BlobHeader with type="OSMData"
    let mut header = fileformat::BlobHeader::new();
    header.type_ = Some("OSMData".to_string());  
    // or: header.set_type("OSMData".to_string());
    header.datasize = Some(blob_bytes.len() as i32);
    // or: header.set_datasize(blob_bytes.len() as i32);

    let header_bytes = header
        .write_to_bytes()
        .expect("Could not serialize local BlobHeader");

    // 5) Frame as <4-byte header-len><header><blob>
    let mut out = Vec::with_capacity(4 + header_bytes.len() + blob_bytes.len());
    
    // Disambiguate from tokio's `write_u32`:
    ByteOrderWriteExt::write_u32::<BigEndian>(&mut out, header_bytes.len() as u32)
        .unwrap();
    
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
