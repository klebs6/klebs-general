// ---------------- [ File: src/serialize_primitive_block.rs ]
crate::ix!();

use crate::proto::{fileformat,osmformat};

/// Serializes a [`PrimitiveBlock`] into a `Blob` and `BlobHeader`.
pub fn serialize_primitive_block(
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
