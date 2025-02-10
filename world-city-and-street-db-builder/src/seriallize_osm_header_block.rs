// ---------------- [ File: src/seriallize_osm_header_block.rs ]
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
#[disable]
mod test_serialize_osm_header_block {
    use super::*;
    use crate::proto::{fileformat, osmformat};
    use protobuf::Message; // For (de)serialization checks
    use std::io;

    #[traced_test]
    fn test_minimal_header_block_success() {
        // Create a minimal HeaderBlock (no bbox, no replication info, etc.)
        let header_block = osmformat::HeaderBlock::new();

        let result = serialize_osm_header_block(header_block);
        assert!(result.is_ok(), "Serializing a minimal HeaderBlock should succeed");
        let (blob_header_bytes, blob_bytes) = result.unwrap();

        // Check that both are non-empty
        assert!(
            !blob_header_bytes.is_empty(),
            "Should produce non-empty blob_header bytes"
        );
        assert!(
            !blob_bytes.is_empty(),
            "Should produce non-empty blob bytes"
        );
    }

    #[traced_test]
    fn test_basic_header_block_with_bbox() {
        // Provide a bounding box, required features, etc.
        let mut header_block = osmformat::HeaderBlock::new();

        let mut bbox = osmformat::HeaderBBox::new();
        bbox.set_left(-77_000_000_000);
        bbox.set_right(-76_000_000_000);
        bbox.set_top(39_000_000_000);
        bbox.set_bottom(38_000_000_000);

        header_block.bbox = protobuf::MessageField::from_option(Some(bbox));
        header_block
            .required_features
            .push("DenseNodes".to_string());
        header_block
            .required_features
            .push("OsmSchema-V0.6".to_string());

        let result = serialize_osm_header_block(header_block.clone());
        assert!(result.is_ok());
        let (blob_header_bytes, blob_bytes) = result.unwrap();

        // Check BlobHeader
        let blob_header = fileformat::BlobHeader::parse_from_bytes(&blob_header_bytes)
            .expect("Should parse BlobHeader from bytes");
        assert_eq!(blob_header.get_type(), "OSMHeader");
        assert_eq!(
            blob_header.get_datasize() as usize,
            blob_bytes.len(),
            "datasize should match the length of the blob bytes"
        );

        // Check Blob
        let blob = fileformat::Blob::parse_from_bytes(&blob_bytes)
            .expect("Should parse Blob from bytes");
        assert!(blob.has_raw(), "We used raw (no compression) in serialization");
        assert_eq!(
            blob.get_raw_size() as usize,
            blob.get_raw().len(),
            "raw_size should match the length of raw data"
        );

        // Verify we can parse the original HeaderBlock from the raw
        let parsed_header_block =
            osmformat::HeaderBlock::parse_from_bytes(blob.get_raw())
                .expect("Should parse back into HeaderBlock");
        // Compare some fields
        let parsed_bbox = parsed_header_block.get_bbox();
        assert_eq!(parsed_bbox.left(), -77_000_000_000);
        assert_eq!(parsed_bbox.right(), -76_000_000_000);
        assert_eq!(parsed_bbox.top(), 39_000_000_000);
        assert_eq!(parsed_bbox.bottom(), 38_000_000_000);

        let feats = parsed_header_block.get_required_features();
        assert!(
            feats.contains(&"DenseNodes".to_string())
                && feats.contains(&"OsmSchema-V0.6".to_string()),
            "Should contain the required features we set"
        );
    }

    #[traced_test]
    fn test_round_trip_serialization() {
        // We'll fill in a few more fields, then parse them back out of the blob.
        let mut header_block = osmformat::HeaderBlock::new();
        header_block.set_source("TestSource".to_string());
        header_block.set_osmosis_replication_timestamp(123456789);
        header_block.set_osmosis_replication_sequence_number(42);
        header_block.set_osmosis_replication_base_url("http://example.com/replication".to_string());

        // Serialize
        let (blob_header_bytes, blob_bytes) =
            serialize_osm_header_block(header_block.clone())
                .expect("Serialization should succeed");

        // Parse the BlobHeader
        let blob_header =
            fileformat::BlobHeader::parse_from_bytes(&blob_header_bytes)
                .expect("Should parse BlobHeader");
        assert_eq!(blob_header.get_type(), "OSMHeader");

        // Parse the Blob
        let blob =
            fileformat::Blob::parse_from_bytes(&blob_bytes).expect("Should parse Blob");
        let raw_data = blob.get_raw();
        // Now parse back into HeaderBlock
        let parsed_block =
            osmformat::HeaderBlock::parse_from_bytes(raw_data)
                .expect("Should parse raw data into HeaderBlock");

        // Compare fields
        assert_eq!(
            parsed_block.get_source(),
            "TestSource",
            "Should preserve 'source' field"
        );
        assert_eq!(
            parsed_block.get_osmosis_replication_timestamp(),
            123456789
        );
        assert_eq!(
            parsed_block.get_osmosis_replication_sequence_number(),
            42
        );
        assert_eq!(
            parsed_block.get_osmosis_replication_base_url(),
            "http://example.com/replication"
        );
    }

    #[traced_test]
    fn test_error_in_header_block_serialization() {
        // We'll do a minimal approach to force an error in `header_block.write_to_bytes()`.
        // The real `protobuf::Message` generally won't fail easily on normal data, so we mock it.

        struct FailingHeaderBlock;
        impl protobuf::Message for FailingHeaderBlock {
            fn is_initialized(&self) -> bool { true }
            fn merge_from(
                &mut self, 
                _is: &mut protobuf::CodedInputStream
            ) -> protobuf::ProtobufResult<()> {
                unimplemented!()
            }
            fn compute_size(&self) -> u32 { 0 }
            fn write_to_with_cached_sizes(
                &self, 
                _os: &mut protobuf::CodedOutputStream
            ) -> protobuf::ProtobufResult<()> {
                // Force an error
                Err(protobuf::ProtobufError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    "Simulated header block write failure",
                )))
            }
            fn get_cached_size(&self) -> u32 { 0 }
        }

        let failing_block = osmformat::HeaderBlock::new(); 
        // We can override the actual `write_to_bytes` by transmuting or partial mocking.
        // For clarity, let's define a local function that does the same steps but uses `FailingHeaderBlock`.
        fn serialize_failing_header_block() -> io::Result<(Vec<u8>, Vec<u8>)> {
            let mock = FailingHeaderBlock;
            let block_bytes = mock.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("HeaderBlock serialization failed: {:?}", e))
            })?;

            let mut blob = fileformat::Blob::new();
            blob.set_raw(block_bytes.clone());
            blob.set_raw_size(block_bytes.len() as i32);

            let blob_bytes = blob.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Blob serialization failed: {:?}", e))
            })?;

            let mut blob_header = fileformat::BlobHeader::new();
            blob_header.set_type("OSMHeader".to_string());
            blob_header.set_datasize(blob_bytes.len() as i32);

            let blob_header_bytes = blob_header.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("BlobHeader serialization failed: {:?}", e))
            })?;

            Ok((blob_header_bytes, blob_bytes))
        }

        let result = serialize_failing_header_block();
        assert!(result.is_err(), "Should fail due to forced error");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Simulated header block write failure"),
            "Error message should indicate the forced failure. Got: {}", err
        );
    }

    #[traced_test]
    fn test_error_in_blob_serialization() {
        // Similarly, we can force an error in writing the Blob. We'll define a local function
        // that calls `blob.write_to_bytes()` but uses a mock. The real code is identical in approach.

        struct MockBlob { raw: Vec<u8> }
        impl protobuf::Message for MockBlob {
            fn is_initialized(&self) -> bool { true }
            fn merge_from(&mut self, _is: &mut protobuf::CodedInputStream) -> protobuf::ProtobufResult<()> {
                unimplemented!()
            }
            fn compute_size(&self) -> u32 { 0 }
            fn write_to_with_cached_sizes(&self, _os: &mut protobuf::CodedOutputStream) -> protobuf::ProtobufResult<()> {
                // Force an error
                Err(protobuf::ProtobufError::IoError(io::Error::new(
                    io::ErrorKind::Other, 
                    "Simulated Blob serialization error"
                )))
            }
            fn get_cached_size(&self) -> u32 { 0 }
        }

        fn attempt_blob_serialization() -> io::Result<(Vec<u8>, Vec<u8>)> {
            // Let's pretend we already got `header_block_bytes`, so we skip that step
            let block_bytes = vec![1,2,3];
            let mut blob = MockBlob { raw: block_bytes };

            let blob_bytes = blob.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Blob serialization failed: {:?}", e))
            })?;

            // Now do the blob header
            let mut blob_header = fileformat::BlobHeader::new();
            blob_header.set_type("OSMHeader".to_string());
            blob_header.set_datasize(blob_bytes.len() as i32);
            let blob_header_bytes = blob_header.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("BlobHeader serialization failed: {:?}", e))
            })?;

            Ok((blob_header_bytes, blob_bytes))
        }

        let result = attempt_blob_serialization();
        assert!(result.is_err(), "Should fail from forced error in writing the Blob");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Simulated Blob serialization error"),
            "Expected forced error. Got: {}", err
        );
    }

    #[traced_test]
    fn test_error_in_blob_header_serialization() {
        // We'll force an error in writing the BlobHeader. This is contrived, but consistent with the pattern.
        struct MockBlobHeader;
        impl protobuf::Message for MockBlobHeader {
            fn is_initialized(&self) -> bool { true }
            fn merge_from(&mut self, _is: &mut protobuf::CodedInputStream) -> protobuf::ProtobufResult<()> {
                unimplemented!()
            }
            fn compute_size(&self) -> u32 { 0 }
            fn write_to_with_cached_sizes(&self, _os: &mut protobuf::CodedOutputStream) -> protobuf::ProtobufResult<()> {
                Err(protobuf::ProtobufError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    "Simulated BlobHeader serialization failure"
                )))
            }
            fn get_cached_size(&self) -> u32 { 0 }
        }

        fn attempt_blobheader_serialization() -> io::Result<(Vec<u8>, Vec<u8>)> {
            // Suppose we've built a normal blob with some bytes
            let blob_bytes = vec![4,5,6];

            // Now we try to write the blob header but fail
            let mock_header = MockBlobHeader;
            let blob_header_bytes = mock_header.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("BlobHeader serialization failed: {:?}", e))
            })?;

            Ok((blob_header_bytes, blob_bytes))
        }

        let result = attempt_blobheader_serialization();
        assert!(result.is_err(), "Should fail with forced error");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Simulated BlobHeader serialization failure"),
            "Expected forced error. Got: {}", err
        );
    }
}
