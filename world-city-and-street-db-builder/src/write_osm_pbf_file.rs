crate::ix!();

/// Asynchronously writes two sets of BlobHeader/Blob pairs
/// (header vs. data) to the target file in `.osm.pbf` order.
pub async fn write_osm_pbf_file(
    path: &Path,
    header_blobheader_bytes: &[u8],
    header_blob_bytes: &[u8],
    data_blobheader_bytes: &[u8],
    data_blob_bytes: &[u8]
) -> std::io::Result<()> {
    trace!("write_osm_pbf_file: creating file at {:?}", path);

    let mut file = match tokio::fs::File::create(path).await {
        Ok(f) => {
            debug!("write_osm_pbf_file: file opened at {:?}", path);
            f
        }
        Err(e) => {
            error!("write_osm_pbf_file: failed to create file {:?}: {:?}", path, e);
            return Err(e);
        }
    };

    // Write the OSMHeader portion
    trace!(
        "write_osm_pbf_file: writing header_blobheader={} bytes + header_blob={} bytes",
        header_blobheader_bytes.len(),
        header_blob_bytes.len()
    );
    crate::write_u32_be(&mut file, header_blobheader_bytes.len() as u32).await?;
    file.write_all(header_blobheader_bytes).await?;
    file.write_all(header_blob_bytes).await?;

    // Write the OSMData portion
    trace!(
        "write_osm_pbf_file: writing data_blobheader={} bytes + data_blob={} bytes",
        data_blobheader_bytes.len(),
        data_blob_bytes.len()
    );
    crate::write_u32_be(&mut file, data_blobheader_bytes.len() as u32).await?;
    file.write_all(data_blobheader_bytes).await?;
    file.write_all(data_blob_bytes).await?;

    debug!("write_osm_pbf_file: completed writing to {:?}", path);
    Ok(())
}
