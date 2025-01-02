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
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        let rt = Runtime::new().unwrap();
        let tmp_path = std::env::temp_dir().join("md5_test.osm.pbf");
        {
            let mut f = StdFile::create(&tmp_path).unwrap();
            f.write_all(b"some random data").unwrap();
        }

        let res = rt.block_on(verify_md5_checksum(&tmp_path));
        assert!(res.is_err());
        match res.err().unwrap() {
            Md5ChecksumVerificationError::ChecksumMismatch { .. } => {},
            _ => panic!("Expected ChecksumMismatch"),
        }
    }
}
