crate::ix!();

/// Serializes the given `HeaderBlock` into a `Blob` and `BlobHeader`.
pub fn serialize_osm_header_block(
    header_block: osmformat::HeaderBlock
) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    trace!("serialize_osm_header_block: serializing HeaderBlock");

    let header_block_bytes = header_block.write_to_bytes().map_err(|e| {
        error!("serialize_osm_header_block: protobuf error: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "HeaderBlock serialization failed")
    })?;

    let mut blob = fileformat::Blob::new();
    blob.set_raw(header_block_bytes.clone());
    blob.set_raw_size(header_block_bytes.len() as i32);

    let blob_bytes = blob.write_to_bytes().map_err(|e| {
        error!("serialize_osm_header_block: blob error: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Blob serialization failed")
    })?;

    let mut blob_header = fileformat::BlobHeader::new();
    blob_header.set_type("OSMHeader".to_string());
    blob_header.set_datasize(blob_bytes.len() as i32);

    let blob_header_bytes = blob_header.write_to_bytes().map_err(|e| {
        error!("serialize_osm_header_block: blob header error: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "BlobHeader serialization failed")
    })?;

    debug!("serialize_osm_header_block: Blob and BlobHeader ready");
    Ok((blob_header_bytes, blob_bytes))
}
