// ---------------- [ File: src/address_record_from_element_and_country.rs ]
crate::ix!();

/// Implements conversion from an OSM PBF element and a `Country` into an `AddressRecord`.
/// Dispatches to one of the above converter functions based on the element variant.
impl<'a> TryFrom<(&osmpbf::Element<'a>, &Country)> for AddressRecord {
    type Error = IncompatibleOsmPbfElement;

    fn try_from((element, country): (&osmpbf::Element<'a>, &Country)) -> Result<Self, Self::Error> {
        match element {
            osmpbf::Element::Node(node)       => convert_osm_node_to_address_record(node, *country),
            osmpbf::Element::Way(way)         => convert_osm_way_to_address_record(way, *country),
            osmpbf::Element::Relation(rel)    => convert_osm_relation_to_address_record(rel, *country),
            osmpbf::Element::DenseNode(dense) => convert_osm_dense_node_to_address_record(dense, *country),
        }
    }
}

/// A macro to generate multiple OSM-element-to-AddressRecord converter functions
/// with minimal boilerplate. Each generated function:
///   1. Extracts an identifier (`id`).
///   2. Logs the element processing progress.
///   3. Invokes a shared `address_record_from_tags(...)`.
///   4. Transforms `IncompatibleOsmPbfElement::IncompatibleOsmPbfNode` into the
///      appropriate variant if needed.
///
/// The macro accepts a list of converters to generate. Each converter
/// is declared in the form:
///
/// ```text
///   fn_name, ElementType, "DescriptiveLabel", error_mapping
/// ```
///
/// - `fn_name` is the desired function identifier.
/// - `ElementType` is any type offering:
///      - `.id()` -> i64
///      - `.tags()` -> Iterator<(K, V)>
/// - `"DescriptiveLabel"` is a textual label included in log messages.
/// - `error_mapping` is a closure of the form:
///      `|id, node_err| { <transform> }`
///   which transforms `IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(...)` into the desired variant.
///   If no transformation is needed (i.e. for Node), simply pass an identity closure.
///
/// Example usage for Node (identity):
/// ```text
/// |_, node_err| IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err)
/// ```
///
/// Example usage for Way (transform to IncompatibleOsmPbfWay):
/// ```text
/// |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(
///     IncompatibleOsmPbfWay::Incompatible { id }
/// )
/// ```
macro_rules! generate_osm_address_record_converters {
    ( $($fn_name:ident, $elem_ty:ty, $label:expr, $err_mapping:expr);+ $(;)?) => {
        $(
            #[doc = concat!("Converts an OSM ", $label, " element into an AddressRecord.")]
            ///
            /// # Arguments
            ///
            /// * `elem`    - Reference to the OSM element.
            /// * `country` - The associated `Country`.
            ///
            /// # Returns
            ///
            /// * `Ok(AddressRecord)` if successful.
            /// * `Err(IncompatibleOsmPbfElement)` if conversion fails.
            fn $fn_name(
                elem: &$elem_ty,
                country: Country
            ) -> Result<AddressRecord, IncompatibleOsmPbfElement> {
                let id = elem.id();
                trace!("{}: converting {} with id {}", stringify!($fn_name), $label, id);
                let tag_iter = elem.tags().map(|(k, v)| (k, v));

                match try_build_address_record_from_tags(tag_iter, country, id) {
                    Ok(record) => {
                        trace!("Successfully converted {} (id={}) into AddressRecord", $label, id);
                        Ok(record)
                    }
                    Err(e) => {
                        error!("Failed to convert {} (id={}): {:?}", $label, id, e);
                        Err(match e {
                            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                                $err_mapping(id, node_err)
                            }
                            other => other,
                        })
                    }
                }
            }
        )+
    };
}

// Generate all the converters in a single macro invocation.
generate_osm_address_record_converters!(
    convert_osm_node_to_address_record,
    osmpbf::Node,
    "Node",
    // Node just passes the node-specific error along (identity).
    |_, node_err| IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err);

    convert_osm_way_to_address_record,
    osmpbf::Way,
    "Way",
    // Convert IncompatibleOsmPbfNode(...) to IncompatibleOsmPbfWay(...).
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(
        IncompatibleOsmPbfWay::Incompatible { id }
    );

    convert_osm_relation_to_address_record,
    osmpbf::Relation,
    "Relation",
    // Convert IncompatibleOsmPbfNode(...) to IncompatibleOsmPbfRelation(...).
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfRelation(
        IncompatibleOsmPbfRelation::Incompatible { id }
    );

    convert_osm_dense_node_to_address_record,
    osmpbf::DenseNode,
    "DenseNode",
    // Convert IncompatibleOsmPbfNode(...) to IncompatibleOsmPbfDenseNode(...).
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfDenseNode(
        IncompatibleOsmPbfDenseNode::Incompatible { id }
    );
);

#[cfg(test)]
mod address_record_from_osm_tests {
    use super::*; // bring in your main code: TryFrom<(&Element,&Country)>, ...
    use std::io::Write;
    use tempfile::TempDir;

    // --- We import osmformat from your generated proto module. Adjust as needed. ---
    use crate::proto::osmformat::{PrimitiveBlock, StringTable, PrimitiveGroup, Node, Way, Relation, DenseNodes};
    // For building OSM Blobs, etc.
    use crate::proto::fileformat::{Blob, BlobHeader};
    use protobuf::Message; // .write_to_bytes()
    // This is the "osmpbf" crate's top-level re-export or your local wrappers
    // We assume `osmpbf::ElementReader` is in scope

    /// Creates a small `.osm.pbf` containing four elements:
    ///   1. A Node with full address tags => should parse OK.
    ///   2. A Node with no address tags => => IncompatibleOsmPbfNode error.
    ///   3. A Way with no address tags => => IncompatibleOsmPbfWay error.
    ///   4. A Relation with no address tags => => IncompatibleOsmPbfRelation error.
    ///   5. A DenseNode with no address tags => => IncompatibleOsmPbfDenseNode error.
    ///
    /// For the Way or Relation to be recognized as such, we add them in `PrimitiveGroup`.
    /// The parser sees them as separate elements. This is fairly minimal/contrived.
    pub fn create_test_pbf_with_node_way_relation_dense(
        path: &std::path::Path,
    ) -> std::io::Result<()> {
        // 1) Build an OSM HeaderBlock with bounding box
        let mut header_block = crate::proto::osmformat::HeaderBlock::new();
        {
            let mut bbox = crate::proto::osmformat::HeaderBBox::new();
            bbox.set_left(-77_000_000_000);
            bbox.set_right(-76_000_000_000);
            bbox.set_top(39_000_000_000);
            bbox.set_bottom(38_000_000_000);
            header_block.bbox = protobuf::MessageField::from_option(Some(bbox));
            header_block.required_features.push("OsmSchema-V0.6".to_string());
            header_block.required_features.push("DenseNodes".to_string());
        }
        let header_block_bytes = header_block.write_to_bytes()?;

        let mut header_blob = Blob::new();
        header_blob.set_raw(header_block_bytes.clone());
        header_blob.set_raw_size(header_block_bytes.len() as i32);
        let header_blob_bytes = header_blob.write_to_bytes()?;

        let mut header_blobheader = BlobHeader::new();
        header_blobheader.set_type("OSMHeader".to_string());
        header_blobheader.set_datasize(header_blob_bytes.len() as i32);
        let header_blobheader_bytes = header_blobheader.write_to_bytes()?;

        // 2) Create a PrimitiveBlock with:
        //    - Node #1 => with tags: addr:city="TestCity", addr:street="TestStreet", addr:postcode="12345"
        //    - Node #2 => with no address tags (just some random tag or none)
        //    - Way #3 => empty references, no tags
        //    - Relation #4 => no tags
        //    - DenseNode #5 => no tags
        let mut primitive_block = PrimitiveBlock::new();
        {
            let mut s_table = StringTable::new();
            // index=0 => ""
            s_table.s.push(b"".to_vec());
            // We'll store all possible needed strings
            // 1) "addr:city"
            s_table.s.push(b"addr:city".to_vec());   // idx=1
            s_table.s.push(b"TestCity".to_vec());    // idx=2
            // 2) "addr:street"
            s_table.s.push(b"addr:street".to_vec()); // idx=3
            s_table.s.push(b"TestStreet".to_vec());  // idx=4
            // 3) "addr:postcode"
            s_table.s.push(b"addr:postcode".to_vec()); // idx=5
            s_table.s.push(b"12345".to_vec());          // idx=6
            // optionally a random tag for Node #2
            s_table.s.push(b"foo".to_vec());       // idx=7
            s_table.s.push(b"bar".to_vec());       // idx=8
            primitive_block.stringtable = protobuf::MessageField::from_option(Some(s_table));

            primitive_block.set_granularity(100);
            primitive_block.set_lat_offset(0);
            primitive_block.set_lon_offset(0);

            let mut group = PrimitiveGroup::new();

            // Node #1 => ID=1, lat/lon near 39.0/-76.0, tags for city/street/postcode
            {
                let mut node1 = Node::new();
                node1.set_id(1);
                let lat1 = (39.0 * 1e9) as i64 / 100; // => 390000000
                let lon1 = (-76.0 * 1e9) as i64 / 100; 
                node1.set_lat(lat1);
                node1.set_lon(lon1);
                // For tags, we push keys=[1,3,5], vals=[2,4,6]
                // meaning: ("addr:city" => "TestCity"), etc.
                node1.keys.push(1);
                node1.vals.push(2);
                node1.keys.push(3);
                node1.vals.push(4);
                node1.keys.push(5);
                node1.vals.push(6);
                group.nodes.push(node1);
            }

            // Node #2 => ID=2, some lat/lon, no *address* tags
            {
                let mut node2 = Node::new();
                node2.set_id(2);
                node2.set_lat((38.5 * 1e9) as i64 / 100);
                node2.set_lon((-76.5 * 1e9) as i64 / 100);
                // We can add a random tag or skip
                // e.g. keys=[7], vals=[8] => ("foo"=>"bar")
                node2.keys.push(7);
                node2.vals.push(8);
                group.nodes.push(node2);
            }

            // Way #3 => ID=3, no tags. We'll skip "refs" array => minimal
            {
                let mut way3 = Way::new();
                way3.set_id(3);
                // no tags => no address => triggers IncompatibleOsmPbfWay
                // no refs => minimal
                group.ways.push(way3);
            }

            // Relation #4 => ID=4, no tags => triggers IncompatibleOsmPbfRelation
            {
                let mut rel4 = Relation::new();
                rel4.set_id(4);
                // no tags => no members => minimal
                group.relations.push(rel4);
            }

            // DenseNode #5 => ID=5, no address tags => triggers IncompatibleOsmPbfDenseNode
            {
                let mut dnodes = DenseNodes::new();
                // we store 1 ID => 5
                dnodes.id.push(5);
                // lat/lon => we store deltas
                // Typically you do "dnodes.lat.push(some_delta)" etc.
                // We'll do minimal: store lat= some delta => 0 => might produce lat=0 
                // or do a single lat offset
                dnodes.lat.push(39_000_000); // e.g. 39.0 * 1e7 if granularity=100
                dnodes.lon.push(-76_000_000);

                // no "keys_vals" => no tags
                // see the official docs for DenseNodes if you want real tags

                group.dense = protobuf::MessageField::from_option(Some(dnodes));
            }

            primitive_block.primitivegroup.push(group);
        }

        let primitive_bytes = primitive_block.write_to_bytes()?;

        let mut data_blob = Blob::new();
        data_blob.set_raw(primitive_bytes.clone());
        data_blob.set_raw_size(primitive_bytes.len() as i32);
        let data_blob_bytes = data_blob.write_to_bytes()?;

        let mut data_blobheader = BlobHeader::new();
        data_blobheader.set_type("OSMData".to_string());
        data_blobheader.set_datasize(data_blob_bytes.len() as i32);
        let data_blobheader_bytes = data_blobheader.write_to_bytes()?;

        // 3) Write to the output file
        let mut file = std::fs::File::create(path)?;
        // (a) OSMHeader
        let header_len = (header_blobheader_bytes.len() as u32).to_be_bytes();
        file.write_all(&header_len)?;
        file.write_all(&header_blobheader_bytes)?;
        file.write_all(&header_blob_bytes)?;
        // (b) OSMData
        let data_len = (data_blobheader_bytes.len() as u32).to_be_bytes();
        file.write_all(&data_len)?;
        file.write_all(&data_blobheader_bytes)?;
        file.write_all(&data_blob_bytes)?;

        Ok(())
    }

    #[traced_test]
    fn test_tryfrom_mixed_elements_in_one_file() {
        // This single .pbf has:
        //   Node #1 => valid address => parse => Ok(AddressRecord)
        //   Node #2 => no address => => IncompatibleOsmPbfNode
        //   Way #3 => => IncompatibleOsmPbfWay
        //   Relation #4 => => IncompatibleOsmPbfRelation
        //   DenseNode #5 => => IncompatibleOsmPbfDenseNode
        let tmp = TempDir::new().unwrap();
        let pbf_path = tmp.path().join("mixed.osm.pbf");
        create_test_pbf_with_node_way_relation_dense(&pbf_path).expect("create minimal pbf");

        let reader = osmpbf::ElementReader::from_path(&pbf_path).expect("open .pbf");
        let country = Country::USA;

        let mut encountered_valid_node = false;
        let mut encountered_bad_node = false;
        let mut encountered_way_error = false;
        let mut encountered_rel_error = false;
        let mut encountered_dn_error = false;

        // We'll parse each element => call `AddressRecord::try_from((&element, &country))`
        // and check the outcome.
        reader.for_each(|element| {
            let res = AddressRecord::try_from((&element, &country));
            match element {
                osmpbf::Element::Node(n) if n.id() == 1 => {
                    // This should be the *valid* node with city/street/postcode
                    assert!(res.is_ok(), "Node #1 => valid address");
                    encountered_valid_node = true;
                }
                osmpbf::Element::Node(n) if n.id() == 2 => {
                    // Missing address => => IncompatibleOsmPbfNode
                    assert!(res.is_err());
                    if let Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err)) = res {
                        // Good. 
                        assert_eq!(n.id(), 2);
                        encountered_bad_node = true;
                    } else {
                        panic!("Node #2 => expected IncompatibleOsmPbfNode error");
                    }
                }
                osmpbf::Element::Node(n) => {
                    todo!()
                }
                osmpbf::Element::Way(w) => {
                    // => IncompatibleOsmPbfWay
                    assert_eq!(w.id(), 3);
                    assert!(res.is_err());
                    match res {
                        Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(way_err)) => {
                            assert_eq!(way_err, IncompatibleOsmPbfWay::Incompatible { id: 3 });
                            encountered_way_error = true;
                        }
                        other => panic!("Way #3 => expected IncompatibleOsmPbfWay, got {:?}", other),
                    }
                }
                osmpbf::Element::Relation(rel) => {
                    // => IncompatibleOsmPbfRelation
                    assert_eq!(rel.id(), 4);
                    assert!(res.is_err());
                    match res {
                        Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfRelation(r_err)) => {
                            assert_eq!(r_err, IncompatibleOsmPbfRelation::Incompatible { id: 4 });
                            encountered_rel_error = true;
                        }
                        other => panic!("Relation #4 => expected IncompatibleOsmPbfRelation, got {:?}", other),
                    }
                }
                osmpbf::Element::DenseNode(dn) => {
                    // => IncompatibleOsmPbfDenseNode
                    assert_eq!(dn.id(), 5);
                    assert!(res.is_err());
                    match res {
                        Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfDenseNode(dn_err)) => {
                            assert_eq!(dn_err, IncompatibleOsmPbfDenseNode::Incompatible { id: 5 });
                            encountered_dn_error = true;
                        }
                        other => panic!("DenseNode #5 => expected IncompatibleOsmPbfDenseNode, got {:?}", other),
                    }
                }
            }
        }).expect("iterator ok");

        assert!(encountered_valid_node, "Node #1 tested");
        assert!(encountered_bad_node,  "Node #2 tested");
        assert!(encountered_way_error, "Way #3 tested");
        assert!(encountered_rel_error, "Relation #4 tested");
        assert!(encountered_dn_error,  "DenseNode #5 tested");
    }
}
