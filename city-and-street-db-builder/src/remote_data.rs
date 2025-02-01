// ---------------- [ File: src/remote_data.rs ]
crate::ix!();

/// Tests for download and MD5 verification (mocked, since we won't do real network ops here)
#[cfg(test)]
mod download_tests {
    use super::*;
    use tokio::runtime::Runtime;
    use std::fs::File as StdFile;
    use std::io::Write;

    #[test]
    fn verify_md5_checksum_mismatch() {
        // Create a tokio runtime without using `unwrap()`.
        let rt = match Runtime::new() {
            Ok(r) => r,
            Err(e) => panic!("Failed to create tokio runtime: {:?}", e),
        };

        // Create a temporary file path.
        let tmp_path = std::env::temp_dir().join("md5_test.osm.pbf");

        // Write some random data to the temporary file.
        {
            let mut f = match StdFile::create(&tmp_path) {
                Ok(created) => created,
                Err(e) => panic!("Failed to create temp file: {:?}", e),
            };
            if let Err(e) = f.write_all(b"some random data") {
                panic!("Failed to write data to file: {:?}", e);
            }
        }

        // Pass both arguments to the `verify_md5_checksum` function,
        // using an obviously incorrect MD5 hash to ensure a mismatch.
        let res = rt.block_on(
            verify_md5_checksum(&tmp_path, "d41d8cd98f00b204e9800998ecf8427e")
        );

        // The result should be an error indicating a checksum mismatch.
        match res {
            Err(Md5ChecksumVerificationError::ChecksumMismatch { .. }) => { /* Test passes */ },
            Err(other) => panic!("Expected ChecksumMismatch, but got: {:?}", other),
            Ok(_) => panic!("Expected verify_md5_checksum to fail, but it succeeded."),
        }
    }
}
