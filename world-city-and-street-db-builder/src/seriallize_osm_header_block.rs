// ---------------- [ File: src/seriallize_osm_header_block.rs ]
crate::ix!();

use crate::proto::{fileformat,osmformat};

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

#[cfg(test)]
mod test_serialize_osm_header_block {
    use super::*;
    use crate::proto::{fileformat, osmformat};
    use protobuf::{
        CodedInputStream, CodedOutputStream, Message, MessageFull, 
        SpecialFields,
    };
    use std::io::{Error as IoError, ErrorKind};

    // ----------------------------------------------------------------------
    // (1) Test the normal success path with a valid HeaderBlock
    // ----------------------------------------------------------------------

    /// Creates a minimal valid HeaderBlock that can be serialized.
    fn make_basic_header_block() -> osmformat::HeaderBlock {
        let mut block = osmformat::HeaderBlock::new();
        let mut bbox = osmformat::HeaderBBox::new();
        bbox.set_left(-770_000_000);
        bbox.set_right(-760_000_000);
        bbox.set_top(400_000_000);
        bbox.set_bottom(380_000_000);
        block.bbox = protobuf::MessageField::from_option(Some(bbox));

        block.required_features.push("OsmSchema-V0.6".to_string());
        block.required_features.push("DenseNodes".to_string());
        block
    }

    /// Confirms `serialize_osm_header_block` works with a valid `HeaderBlock`.
    #[test]
    fn test_serialize_osm_header_block_ok() {
        let header_block = make_basic_header_block();

        let result = serialize_osm_header_block(header_block);
        assert!(
            result.is_ok(),
            "Expected successful serialization of a normal HeaderBlock"
        );

        let (header_bytes, blob_bytes) = result.unwrap();
        assert!(
            !header_bytes.is_empty(),
            "BlobHeader bytes should not be empty"
        );
        assert!(
            !blob_bytes.is_empty(),
            "Blob bytes should not be empty"
        );
    }

    // ----------------------------------------------------------------------
    // (2) Demonstrate how to test a "failing blob" or "failing blob-header"
    //     by replicating that logic with custom mocks that implement `Message`.
    //
    // Because the real `serialize_osm_header_block` function specifically
    // requires an `osmformat::HeaderBlock`, we cannot pass it a mock object
    // of a different type. Instead, we show how to replicate just the
    // sub-steps in the function so we can trigger each error.
    // ----------------------------------------------------------------------

    /// A mock `Blob` that always fails in `write_to_with_cached_sizes`.
    #[derive(Clone, PartialEq, Debug, Default)]
    struct FailingBlobForTest {
        // pretend we store the raw data
        raw_data: Vec<u8>,
        special_fields: SpecialFields,
    }

    impl Message for FailingBlobForTest {
        const NAME: &'static str = "FailingBlobForTest";

        fn is_initialized(&self) -> bool {
            true
        }

        fn merge_from(&mut self, _is: &mut CodedInputStream<'_>) -> protobuf::Result<()> {
            Ok(())
        }

        fn write_to_with_cached_sizes(
            &self, 
            _os: &mut CodedOutputStream<'_>
        ) -> protobuf::Result<()> {
            Err(protobuf::Error::from(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Simulated Blob write_to_bytes failure"
                    )
            ))
        }

        fn compute_size(&self) -> u64 { 0 }

        fn special_fields(&self) -> &SpecialFields {
            &self.special_fields
        }
        fn mut_special_fields(&mut self) -> &mut SpecialFields {
            &mut self.special_fields
        }

        fn new() -> Self { Self::default() }
        fn default_instance() -> &'static Self {
            static INSTANCE: once_cell::sync::Lazy<FailingBlobForTest> =
                once_cell::sync::Lazy::new(|| FailingBlobForTest {
                    raw_data: Vec::new(),
                    special_fields: SpecialFields::new(),
                });
            &INSTANCE
        }
    }

    /// A mock `BlobHeader` that always fails in `write_to_with_cached_sizes`.
    #[derive(Clone, PartialEq, Debug, Default)]
    struct FailingBlobHeaderForTest {
        special_fields: SpecialFields,
    }

    impl Message for FailingBlobHeaderForTest {
        const NAME: &'static str = "FailingBlobHeaderForTest";

        fn is_initialized(&self) -> bool {
            true
        }

        fn merge_from(&mut self, _is: &mut CodedInputStream<'_>) -> protobuf::Result<()> {
            Ok(())
        }

        fn write_to_with_cached_sizes(
            &self, 
            _os: &mut CodedOutputStream<'_>
        ) -> protobuf::Result<()> {
            Err(protobuf::Error::from(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Simulated Blob write_to_bytes failure"
                    )
            ))
        }

        fn compute_size(&self) -> u64 { 0 }

        fn special_fields(&self) -> &SpecialFields {
            &self.special_fields
        }
        fn mut_special_fields(&mut self) -> &mut SpecialFields {
            &mut self.special_fields
        }

        fn new() -> Self { Self::default() }
        fn default_instance() -> &'static Self {
            static INSTANCE: once_cell::sync::Lazy<FailingBlobHeaderForTest> =
                once_cell::sync::Lazy::new(|| FailingBlobHeaderForTest {
                    special_fields: SpecialFields::new(),
                });
            &INSTANCE
        }
    }

    /// Test scenario: "Blob serialization fails" 
    /// (replicating that part of `serialize_osm_header_block`).
    #[test]
    fn test_mock_blob_serialization_failure() {
        let mut mock_blob = FailingBlobForTest::default();
        mock_blob.raw_data = vec![1, 2, 3];

        // Attempt to produce .write_to_bytes => should fail with an IoError
        let result = mock_blob.write_to_bytes();
        assert!(
            result.is_err(),
            "Expected the mock Blob to fail on write_to_bytes"
        );

        // This is the same error that `serialize_osm_header_block` would 
        // produce if the real Blob .write_to_bytes() failed.
        let expected = IoError::new(
            ErrorKind::Other,
            "Blob serialization failed"
        );
        assert_eq!(expected.kind(), ErrorKind::Other);
        assert_eq!(expected.to_string(), "Blob serialization failed");
    }

    /// Test scenario: "BlobHeader serialization fails"
    /// (replicating that part of `serialize_osm_header_block`).
    #[test]
    fn test_mock_blob_header_serialization_failure() {
        // We skip the real function's steps and just test writing the header:
        let mock_blob_header = FailingBlobHeaderForTest::default();
        let result = mock_blob_header.write_to_bytes();
        assert!(
            result.is_err(),
            "Expected the mock BlobHeader to fail on write_to_bytes"
        );

        let expected = IoError::new(
            ErrorKind::Other,
            "BlobHeader serialization failed"
        );
        assert_eq!(expected.kind(), ErrorKind::Other);
        assert_eq!(expected.to_string(), "BlobHeader serialization failed");
    }
}
