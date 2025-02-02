// ---------------- [ File: src/create_tiny_osm_pbf.rs ]
crate::ix!();

/// Creates a very small .osm.pbf file with:
///   - A single OSMHeader blob
///   - A single OSMData blob that contains one node with two address tags
///
/// The resulting file should be enough for a test fixture in your integration tests.
///
/// Note: This uses the `osmpbf::proto::{fileformat,osmformat}` modules,
///       which `osmpbf` normally uses internally for reading. They’re not officially
///       documented for writing, but you can still access them in your own code.
use std::fs::File;
use std::io::Write;
use byteorder::{BigEndian, WriteBytesExt}; // for writing the 4-byte length prefix
use protobuf::{Message,MessageField};

// Pull in the generated protobuf structs from the `osmpbf` crate
//
// TODO: pull request created on the upstream to expose these:
//
// ```rust
//use osmpbf::protos::fileformat;
//use osmpbf::protos::osmformat;
//```
use crate::proto::{fileformat, osmformat}; // our newly generated modules

pub fn create_tiny_osm_pbf(path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    // 1) Build an OSM HeaderBlock that basically has bounding box + required_features
    let mut headerblock = osmformat::HeaderBlock::new();
    {
        // Optional: set a bounding box
        let mut bbox = osmformat::HeaderBBox::new();
        // We'll just set some random bounding box. Values are in nanodegrees.
        // For example: left = -77 degrees => -77000000000 nanodegrees
        bbox.set_left(-77_000_000_000);
        bbox.set_right(-76_000_000_000);
        bbox.set_top(39_000_000_000);
        bbox.set_bottom(38_000_000_000);
        headerblock.bbox = MessageField::from_option(Some(bbox));

        // Mark the "required_features" so that readers won't refuse it
        headerblock.required_features.push("OsmSchema-V0.6".to_string());
        // "DenseNodes" is often used, but we won't do dense in this snippet. Still, it's typical:
        headerblock.required_features.push("DenseNodes".to_string());
    }

    // Serialize this HeaderBlock into bytes
    let headerblock_bytes = match headerblock.write_to_bytes() {
        Ok(b) => b,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    };

    // Make a fileformat::Blob that contains raw (uncompressed) bytes
    let mut header_blob = fileformat::Blob::new();
    header_blob.set_raw(headerblock_bytes.clone());
    header_blob.set_raw_size(headerblock_bytes.len() as i32);

    // Serialize the blob
    let header_blob_bytes = match header_blob.write_to_bytes() {
        Ok(b) => b,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    };

    // Make a BlobHeader for that blob
    let mut header_blobheader = fileformat::BlobHeader::new();
    header_blobheader.set_type("OSMHeader".to_string());
    header_blobheader.set_datasize(header_blob_bytes.len() as i32);

    // Serialize the BlobHeader itself
    let header_blobheader_bytes = match header_blobheader.write_to_bytes() {
        Ok(b) => b,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    };

    // 2) Build a PrimitiveBlock with a single Node
    let mut primitive_block = osmformat::PrimitiveBlock::new();
    {
        // We need a StringTable entry for each tag key and value, plus one for empty:
        // Let's create a tiny string table with these entries:
        //   index 0 => ""  (conventionally the empty string)
        //   index 1 => "addr:city"
        //   index 2 => "test city fixture"
        //   index 3 => "addr:street"
        //   index 4 => "test street fixture"
        let mut stringtable = osmformat::StringTable::new();
        stringtable.s.push(b"".to_vec()); // index 0
        stringtable.s.push(b"addr:city".to_vec()); // index 1
        stringtable.s.push(b"test city fixture".to_vec()); // index 2
        stringtable.s.push(b"addr:street".to_vec()); // index 3
        stringtable.s.push(b"test street fixture".to_vec()); // index 4

        primitive_block.stringtable = MessageField::from_option(Some(stringtable));

        // We'll define a single PrimitiveGroup that has a single Node
        let mut group = osmformat::PrimitiveGroup::new();

        // Create a Node. We'll do node.id=1001, plus some lat/lon roughly near Baltimore
        // In OSM PBF, lat/lon are stored with offsets + granularity, so see below:
        let mut node = osmformat::Node::new();
        node.set_id(1001);

        // We'll store lat=39.283, lon=-76.616, but in "nanodegrees" with default granularity=100
        // Actually, let's set:
        //   block.granularity=100
        //   block.lat_offset=0, block.lon_offset=0
        //   node.lat = lat_in_units = floor(39.283 / 1e-9 / 100)
        //
        // So 39.283 deg => 39.283 / (1e-9) = 39,283,000,000 nanodeg
        // Div by 100 => 392,830,000 => store that in node.lat
        //
        // We'll do the same for lon => -76.616 => -76,616,000,000 nanodeg => -766,160,000 in node.lon
        // (floor that or cast to i64).
        let granularity = 100;
        let lat_offset = 0;
        let lon_offset = 0;
        primitive_block.set_granularity(granularity);
        primitive_block.set_lat_offset(lat_offset);
        primitive_block.set_lon_offset(lon_offset);

        let lat_f64 = 39.283;
        let lon_f64 = -76.616;
        let lat_nano = (lat_f64 * 1e9) as i64; // e.g. 39_283_000_000
        let lon_nano = (lon_f64 * 1e9) as i64; // e.g. -76_616_000_000
        let lat_units = lat_nano / (granularity as i64); // e.g. 392_830_000
        let lon_units = lon_nano / (granularity as i64); // e.g. -766_160_000

        node.set_lat(lat_units);
        node.set_lon(lon_units);

        // Now let's set tags: "addr:city" => "test city fixture"
        // key index=1, val index=2
        // We'll push them onto node.keys / node.vals
        node.keys.push(1); // index=1 => "addr:city"
        node.vals.push(2); // index=2 => "test city fixture"

        // Add second tag: "addr:street" => "test street fixture"
        node.keys.push(3); // "addr:street"
        node.vals.push(4); // "test street fixture"

        group.nodes.push(node);

        primitive_block.primitivegroup.push(group);
    }

    // Serialize the block
    let primitive_block_bytes = match primitive_block.write_to_bytes() {
        Ok(b) => b,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    };

    // Make a fileformat::Blob
    let mut data_blob = fileformat::Blob::new();
    data_blob.set_raw(primitive_block_bytes.clone());
    data_blob.set_raw_size(primitive_block_bytes.len() as i32);

    // Serialize the blob
    let data_blob_bytes = match data_blob.write_to_bytes() {
        Ok(b) => b,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    };

    // Make a BlobHeader for that second blob
    let mut data_blobheader = fileformat::BlobHeader::new();
    data_blobheader.set_type("OSMData".to_string());
    data_blobheader.set_datasize(data_blob_bytes.len() as i32);

    let data_blobheader_bytes = match data_blobheader.write_to_bytes() {
        Ok(b) => b,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    };

    // 3) Write everything to disk in standard PBF format
    let mut file = File::create(path)?;

    // (a) OSMHeader blob
    file.write_u32::<BigEndian>(header_blobheader_bytes.len() as u32)?; // 4 bytes
    file.write_all(&header_blobheader_bytes)?;
    file.write_all(&header_blob_bytes)?;

    // (b) OSMData blob
    file.write_u32::<BigEndian>(data_blobheader_bytes.len() as u32)?; // 4 bytes
    file.write_all(&data_blobheader_bytes)?;
    file.write_all(&data_blob_bytes)?;

    Ok(())
}

// ----------------------------------------------------
// Example usage in a test:
// ----------------------------------------------------
//
// #[test]
// fn test_create_tiny_osm_pbf() -> std::io::Result<()> {
//     let tmp_dir = tempfile::TempDir::new()?;
//     let pbf_path = tmp_dir.path().join("tiny.osm.pbf");
//
//     create_tiny_osm_pbf(&pbf_path)?;
//     // Now parse it with osmpbf, ensure we see the node and the tags.
//     // ...
//     Ok(())
// }
