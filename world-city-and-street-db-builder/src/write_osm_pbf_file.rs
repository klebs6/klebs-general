// ---------------- [ File: src/write_osm_pbf_file.rs ]
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

#[cfg(test)]
mod test_write_osm_pbf_file {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
    use tempfile::TempDir;
    use std::io;
    use std::path::PathBuf;

    // We'll parse the actual BlobHeader with the generated fileformat code from your proto. 
    // If your code is in crate::proto::fileformat, import it:
    use crate::proto::fileformat; // or wherever your generated types live
    use protobuf::Message;        // for parse_from_bytes()

    /// Creates a valid BlobHeader with `datasize`, `type`, etc.
    fn make_blobheader(blob_type: &str, datasize: usize) -> Vec<u8> {
        let mut hdr = fileformat::BlobHeader::new();
        hdr.set_type(blob_type.to_string()); // e.g. "OSMHeader" or "OSMData"
        hdr.set_datasize(datasize as i32);
        hdr.write_to_bytes().expect("BlobHeader serialization")
    }

    /// Creates a valid Blob with `raw` = `data`.
    fn make_blob(data: &[u8]) -> Vec<u8> {
        let mut blob = fileformat::Blob::new();
        blob.set_raw(data.to_vec());
        blob.set_raw_size(data.len() as i32);
        blob.write_to_bytes().expect("Blob serialization")
    }

    /// Reads exactly one OSM PBF block (BlobHeader+Blob).
    /// Returns (header_bytes, blob_bytes).
    async fn read_pbf_block(file: &mut tokio::fs::File) -> io::Result<(Vec<u8>, Vec<u8>)> {
        // 1) Read 4-byte length => size of BlobHeader
        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf).await?; 
        let blobheader_len = byteorder::BigEndian::read_u32(&len_buf);

        // 2) Read that many bytes => the BlobHeader
        let mut blobheader_data = vec![0u8; blobheader_len as usize];
        file.read_exact(&mut blobheader_data).await?;

        // 3) Parse the BlobHeader to learn how many bytes in the Blob
        //    (the 'datasize' field):
        let header_msg = fileformat::BlobHeader::parse_from_bytes(&blobheader_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let datasize = header_msg.datasize() as usize;

        // 4) Read exactly 'datasize' bytes => the Blob
        let mut blob_data = vec![0u8; datasize];
        file.read_exact(&mut blob_data).await?;

        // Return them so the test can compare with the expected arrays
        Ok((blobheader_data, blob_data))
    }

    #[traced_test]
    async fn test_successful_write_and_read_back() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pbf_path = temp_dir.path().join("test.osm.pbf");

        // Suppose we want the “header” block to have some 5-byte raw data: [1,2,3,4,5].
        // We'll actually build a real Blob for that raw data:
        let header_blob_bytes = make_blob(&[1,2,3,4,5]);
        // Then create a BlobHeader that says "OSMHeader" and .datasize =  the blob’s length
        let header_blobheader_bytes = make_blobheader("OSMHeader", header_blob_bytes.len());

        // For the “data” block, same pattern; maybe 3 bytes: [9,9,9].
        let data_blob_bytes = make_blob(&[9,9,9]);
        let data_blobheader_bytes = make_blobheader("OSMData", data_blob_bytes.len());

        // 1) Write the file
        write_osm_pbf_file(
            &pbf_path,
            &header_blobheader_bytes,
            &header_blob_bytes,
            &data_blobheader_bytes,
            &data_blob_bytes,
        )
            .await
            .expect("Should write file successfully");

        // 2) Read & parse
        let mut file = tokio::fs::File::open(&pbf_path).await
            .expect("Should open for reading");

        let (hbh_data, hb_data) = read_pbf_block(&mut file).await
            .expect("read first block");
        // Confirm that hbh_data == the bytes we made for header_blobheader
        assert_eq!(hbh_data, header_blobheader_bytes);
        // Confirm that hb_data == the bytes we made for header_blob
        assert_eq!(hb_data, header_blob_bytes);

        let (dbh_data, db_data) = read_pbf_block(&mut file).await
            .expect("read second block");
        assert_eq!(dbh_data, data_blobheader_bytes);
        assert_eq!(db_data, data_blob_bytes);
    }


    #[traced_test]
    async fn test_cannot_create_file_returns_error() {
        let temp_dir = TempDir::new().expect("temp dir");
        let path_is_dir = temp_dir.path(); // a directory, not a file

        let header_blobheader_bytes = b"some_header";
        let header_blob_bytes       = b"some_data";
        let data_blobheader_bytes   = b"some_data_header";
        let data_blob_bytes         = b"some_data_data";

        let result = write_osm_pbf_file(
            path_is_dir,
            header_blobheader_bytes,
            header_blob_bytes,
            data_blobheader_bytes,
            data_blob_bytes
        ).await;

        assert!(
            result.is_err(),
            "Attempting to create a file with a directory path => error"
        );
    }

    #[traced_test]
    async fn test_error_in_writing_blobheader_bytes() {
        // We'll simulate an I/O error by making the file read-only (on Unix).
        let temp_dir = TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("read_only.osm.pbf");

        // Pre-create the file
        tokio::fs::File::create(&pbf_path).await.unwrap();

        // Attempt to remove write perms:
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = tokio::fs::metadata(&pbf_path).await.unwrap();
            let mut perms = meta.permissions();
            perms.set_mode(0o444); // read-only
            tokio::fs::set_permissions(&pbf_path, perms).await.unwrap();
        }

        // Now writing should fail
        let result = write_osm_pbf_file(
            &pbf_path,
            b"header_blobheader",
            b"header_blob",
            b"data_blobheader",
            b"data_blob"
        ).await;
        assert!(result.is_err(), "Writing to read-only file => error");
    }

    #[traced_test]
    async fn test_zero_length_blobs_ok() {
        let temp_dir = TempDir::new().expect("temp dir");
        let pbf_path = temp_dir.path().join("empty_blobs.osm.pbf");

        // Build an empty Blob + BlobHeader for the first block
        let header_blob_bytes = make_blob(&[]);  // no data
        let header_blobheader_bytes = make_blobheader("OSMHeader", header_blob_bytes.len());

        // Build an empty Blob + BlobHeader for the second block
        let data_blob_bytes = make_blob(&[]);
        let data_blobheader_bytes = make_blobheader("OSMData", data_blob_bytes.len());

        write_osm_pbf_file(
            &pbf_path,
            &header_blobheader_bytes,
            &header_blob_bytes,
            &data_blobheader_bytes,
            &data_blob_bytes
        )
            .await
            .expect("Should succeed even if zero-length");

        let mut file = tokio::fs::File::open(&pbf_path).await
            .expect("open file for reading");

        let (empty_hbh, empty_hb) = read_pbf_block(&mut file).await
            .expect("reading first block");
        assert!(empty_hb.is_empty(),  "header_blob was zero-length");
        // The hbh_data itself is *not* necessarily empty at the byte level
        // because a real proto with datasize=0 might be a few bytes.
        // But if you want to confirm it’s the same bytes you wrote:
        assert_eq!(empty_hbh, header_blobheader_bytes);

        let (empty_dbh, empty_db) = read_pbf_block(&mut file).await
            .expect("reading second block");
        assert!(empty_db.is_empty(),  "data_blob was zero-length");
        assert_eq!(empty_dbh, data_blobheader_bytes);
    }
}
