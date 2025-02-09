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
#[disable]
mod test_write_osm_pbf_file {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
    use tempfile::TempDir;
    use std::io;
    use std::path::PathBuf;

    /// Helper to read a 4-byte length prefix in big-endian and then read that many bytes.
    /// Returns the loaded Vec of that length if successful.
    async fn read_blob_section(file: &mut tokio::fs::File) -> io::Result<Vec<u8>> {
        let mut len_buf = [0u8; 4];
        if let Err(e) = file.read_exact(&mut len_buf).await {
            return Err(e);
        }
        let length = byteorder::BigEndian::read_u32(&len_buf);
        let mut data = vec![0u8; length as usize];
        file.read_exact(&mut data).await?;
        Ok(data)
    }

    #[tokio::test]
    async fn test_successful_write_and_read_back() {
        // We'll create a temp directory for the file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pbf_path = temp_dir.path().join("test.osm.pbf");

        // Provide some header + data blobs
        let header_blobheader_bytes = b"HEADER_BLOB_HEADER";
        let header_blob_bytes = b"HEADER_BLOB_DATA";
        let data_blobheader_bytes = b"DATA_BLOB_HEADER";
        let data_blob_bytes = b"DATA_BLOB_DATA";

        // (1) Call the function
        write_osm_pbf_file(
            &pbf_path,
            header_blobheader_bytes,
            header_blob_bytes,
            data_blobheader_bytes,
            data_blob_bytes
        ).await
         .expect("Should write file successfully");

        // (2) Read the file back to confirm structure
        let mut file = tokio::fs::File::open(&pbf_path).await
            .expect("Should open for reading");

        // Read the first length + header blobheader
        let hbh = read_blob_section(&mut file).await.expect("Read header_blobheader_bytes");
        assert_eq!(hbh, header_blobheader_bytes);

        // Next read the header blob
        let hb = read_blob_section(&mut file).await.expect("Read header_blob_bytes");
        assert_eq!(hb, header_blob_bytes);

        // Next read the data blobheader
        let dbh = read_blob_section(&mut file).await.expect("Read data_blobheader_bytes");
        assert_eq!(dbh, data_blobheader_bytes);

        // Finally read the data blob
        let db = read_blob_section(&mut file).await.expect("Read data_blob_bytes");
        assert_eq!(db, data_blob_bytes);
    }

    #[tokio::test]
    async fn test_cannot_create_file_returns_error() {
        // We'll try writing to a directory path, expecting an error because we can't create a file with that name
        let temp_dir = TempDir::new().expect("temp dir");
        let path_is_dir = temp_dir.path(); // This is a directory, not a file

        let header_blobheader_bytes = b"some_header";
        let header_blob_bytes = b"some_data";
        let data_blobheader_bytes = b"some_data_header";
        let data_blob_bytes = b"some_data_data";

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

    #[tokio::test]
    async fn test_error_in_writing_blobheader_bytes() {
        // We'll simulate an I/O error by opening a file in read-only mode,
        // so writing the first length prefix fails
        let temp_dir = TempDir::new().unwrap();
        let pbf_path = temp_dir.path().join("read_only.osm.pbf");

        // Pre-create the file
        tokio::fs::File::create(&pbf_path).await.unwrap();
        // Now re-open it read-only
        let read_only_file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(false)
            .open(&pbf_path).await.unwrap();
        drop(read_only_file); // Actually, we can't pass it in directly, we rely on the function to open. We'll do a trick:

        // We'll remove write perms from the file on Unix. Another approach: pass an invalid path. 
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&pbf_path).await.unwrap().permissions();
            perms.set_mode(0o444); // read-only
            tokio::fs::set_permissions(&pbf_path, perms).await.unwrap();
        }

        // Now write => fails
        let result = write_osm_pbf_file(
            &pbf_path,
            b"header_blobheader",
            b"header_blob",
            b"data_blobheader",
            b"data_blob"
        ).await;
        assert!(result.is_err(), "Writing to read-only file => error");
    }

    #[tokio::test]
    async fn test_zero_length_blobs_ok() {
        // A corner case: zero-length header blob, zero-length data blob
        let temp_dir = TempDir::new().expect("temp dir");
        let pbf_path = temp_dir.path().join("empty_blobs.osm.pbf");

        // We'll write empty arrays
        write_osm_pbf_file(
            &pbf_path,
            &[], // header_blobheader
            &[], // header_blob
            &[], // data_blobheader
            &[], // data_blob
        ).await
         .expect("Should succeed even if zero-length");

        // Let's verify the file contents:
        // We expect 2 length prefixes => 0, then 0. Then 2 more => 0, then 0. 
        // So total of 4 times 4 bytes of length => 16 bytes of zeros?
        let mut file = tokio::fs::File::open(&pbf_path).await
            .expect("open file");
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await.unwrap();
        // we expect 4 length prefixes, each 4 bytes, all zeros => total 16 zero bytes
        let expected = vec![0u8; 16];
        assert_eq!(contents, expected, "Should be 16 zero bytes for 4 length prefixes, no payloads");
    }
}
