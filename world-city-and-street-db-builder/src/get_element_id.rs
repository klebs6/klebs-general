// ---------------- [ File: src/get_element_id.rs ]
crate::ix!();

/// Retrieves the element ID (Node, Way, Relation, or DenseNode), or returns "?" if unknown.
/// Primarily used for logging.
pub fn get_element_id(element: &osmpbf::Element) -> String {
    match element {
        osmpbf::Element::Node(n)       => format!("{}", n.id()),
        osmpbf::Element::Way(w)        => format!("{}", w.id()),
        osmpbf::Element::Relation(r)   => format!("{}", r.id()),
        osmpbf::Element::DenseNode(dn) => format!("{}", dn.id()),
    }
}

#[cfg(test)]
mod get_element_id_integration_tests {
    use std::{
        fs::File,
        io::{Write, BufWriter},
        path::PathBuf,
    };
    use tempfile::TempDir;

    use osmpbf::{ElementReader, Element};
    use protobuf::Message;  // from 'protobuf' crate, used by osmpbf
    use crate::get_element_id;

    #[test]
    fn test_get_element_id_for_all_variants() {
        // 1) Write a minimal OSM PBF file that contains:
        //    - A Node(id=101)
        //    - A Way(id=202)
        //    - A Relation(id=303)
        //    - A DenseNode(id=404)
        // Then read it via osmpbf::ElementReader and verify get_element_id returns the correct IDs.

        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let pbf_path = tmp_dir.path().join("test_all_variants.osm.pbf");
        create_test_pbf_with_all_variants(&pbf_path).expect("Failed to create test PBF");

        // 2) Parse the file
        let f = File::open(&pbf_path).expect("Failed to open PBF");
        let reader = ElementReader::new(f);

        // 3) We'll keep track of which IDs we saw
        let mut seen_node    = false;
        let mut seen_way     = false;
        let mut seen_rel     = false;
        let mut seen_dense   = false;

        reader.for_each(|element| {
            let id_str = get_element_id(&element);
            let id_num: i64 = id_str.parse().unwrap_or(-99999);

            match element {
                Element::Node(_) => {
                    assert_eq!(id_num, 101, "Node should have ID=101");
                    seen_node = true;
                },
                Element::Way(_) => {
                    assert_eq!(id_num, 202, "Way should have ID=202");
                    seen_way = true;
                },
                Element::Relation(_) => {
                    assert_eq!(id_num, 303, "Relation should have ID=303");
                    seen_rel = true;
                },
                Element::DenseNode(_) => {
                    assert_eq!(id_num, 404, "DenseNode should have ID=404");
                    seen_dense = true;
                }
            }
        }).expect("Error reading elements from PBF");

        assert!(seen_node,  "Expected to see a Node with ID=101");
        assert!(seen_way,   "Expected to see a Way with ID=202");
        assert!(seen_rel,   "Expected to see a Relation with ID=303");
        assert!(seen_dense, "Expected to see a DenseNode with ID=404");
    }

    /// Creates a minimal OSM PBF file at `path` that has one Node, one Way, one Relation, and one DenseNode.
    fn create_test_pbf_with_all_variants(path: &PathBuf) -> std::io::Result<()> {
        // We'll use the same technique that is often used in minimal pbf creation:
        // 1) Create the OSMHeader block, serialize it to a Blob.
        // 2) Create a PrimitiveBlock with:
        //     * A Node
        //     * A Way
        //     * A Relation
        //     * A DenseNodes block
        // Then write them out with the appropriate BlobHeader + Blob.

        // The "proto" modules are re‐exported by `osmpbf::proto`
        use crate::proto::fileformat::{Blob, BlobHeader};
        use crate::proto::osmformat::{
            HeaderBlock, HeaderBBox, PrimitiveBlock,
            PrimitiveGroup, Node, Way, Relation, DenseNodes,
        };
        use protobuf::MessageField;

        // (A) Prepare the HeaderBlock
        let mut header_block = HeaderBlock::new();
        {
            let mut bbox = HeaderBBox::new();
            bbox.set_left(-77_000_000_000);
            bbox.set_right(-76_000_000_000);
            bbox.set_top(39_000_000_000);
            bbox.set_bottom(38_000_000_000);
            header_block.bbox = MessageField::some(bbox);

            header_block.required_features.push("OsmSchema-V0.6".to_string());
            header_block.required_features.push("DenseNodes".to_string());
        }
        let header_bytes = header_block.write_to_bytes().unwrap();

        let mut header_blob = Blob::new();
        header_blob.set_raw(header_bytes.clone());
        header_blob.set_raw_size(header_bytes.len() as i32);
        let header_blob_bytes = header_blob.write_to_bytes().unwrap();

        let mut header_blobheader = BlobHeader::new();
        header_blobheader.set_type("OSMHeader".to_string());
        header_blobheader.set_datasize(header_blob_bytes.len() as i32);
        let header_blobheader_bytes = header_blobheader.write_to_bytes().unwrap();

        // (B) Prepare a PrimitiveBlock with Node, Way, Relation, DenseNodes
        let mut prim_block = PrimitiveBlock::new();

        // We must have a StringTable. We'll keep it trivial for now: index 0 => ""
        {
            use crate::proto::osmformat::StringTable;
            let mut st = StringTable::new();
            st.s.push(b"".to_vec());
            prim_block.stringtable = MessageField::some(st);
        }

        // We use granularity=100 to avoid lat/lon offset complexities, or keep it default
        prim_block.set_granularity(100);
        prim_block.set_lat_offset(0);
        prim_block.set_lon_offset(0);

        let mut group = PrimitiveGroup::new();

        // (1) Node with id=101
        {
            let mut n = Node::new();
            n.set_id(101);
            // Some minimal lat/lon just to have valid data
            n.set_lat(39_000_000_000 / 100); // means 39.0
            n.set_lon(-76_000_000_000 / 100); // means -76.0
            group.nodes.push(n);
        }

        // (2) Way with id=202
        {
            let mut w = Way::new();
            w.set_id(202);
            group.ways.push(w);
        }

        // (3) Relation with id=303
        {
            let mut r = Relation::new();
            r.set_id(303);
            group.relations.push(r);
        }

        // (4) DenseNodes block, with exactly 1 DenseNode => id=404
        {
            let mut dn = DenseNodes::new();
            // The 'id' field is stored in delta‐encoded form. So if we want exactly one DenseNode with ID=404,
            // we push 404 as the first (and only) ID. The library’s DenseNode decoding will interpret it.
            dn.id.push(404);
            // We must also set lat, lon. Same approach: delta-encoded, so push 39_000_000_000/100, etc.
            dn.lat.push(39_000_000_000 / 100);
            dn.lon.push(-76_000_000_000 / 100);
            group.dense = MessageField::some(dn);
        }

        prim_block.primitivegroup.push(group);
        let prim_bytes = prim_block.write_to_bytes().unwrap();

        let mut data_blob = Blob::new();
        data_blob.set_raw(prim_bytes.clone());
        data_blob.set_raw_size(prim_bytes.len() as i32);
        let data_blob_bytes = data_blob.write_to_bytes().unwrap();

        let mut data_blobheader = BlobHeader::new();
        data_blobheader.set_type("OSMData".to_string());
        data_blobheader.set_datasize(data_blob_bytes.len() as i32);
        let data_blobheader_bytes = data_blobheader.write_to_bytes().unwrap();

        // (C) Write all to file
        let mut f = BufWriter::new(File::create(path)?);
        // 1) Write the header blob length + header blobheader bytes + header blob
        let size_buf = (header_blobheader_bytes.len() as u32).to_be_bytes();
        f.write_all(&size_buf)?;
        f.write_all(&header_blobheader_bytes)?;
        f.write_all(&header_blob_bytes)?;

        // 2) Write the data blob length + data blobheader bytes + data blob
        let size_buf2 = (data_blobheader_bytes.len() as u32).to_be_bytes();
        f.write_all(&size_buf2)?;
        f.write_all(&data_blobheader_bytes)?;
        f.write_all(&data_blob_bytes)?;

        f.flush()?;
        Ok(())
    }
}
