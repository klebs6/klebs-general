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

#[cfg(test)]
#[disable]
mod test_serialize_primitive_block {
    use super::*;
    use crate::proto::{fileformat, osmformat}; // Adjust if your file structure is different
    use protobuf::Message; // For optional round-trip parse checks
    use std::io;

    #[test]
    fn test_minimal_primitive_block_success() {
        // We'll create the smallest valid PrimitiveBlock: 
        // an empty string table and no primitive groups
        let mut pblock = osmformat::PrimitiveBlock::new();
        
        // Attempt serialization
        let result = serialize_primitive_block(pblock);
        assert!(
            result.is_ok(),
            "Serializing a minimal, empty PrimitiveBlock should succeed"
        );

        let (blob_header_bytes, blob_bytes) = result.unwrap();
        assert!(!blob_header_bytes.is_empty(), "Should produce non-empty header bytes");
        assert!(!blob_bytes.is_empty(), "Should produce non-empty blob bytes");
    }

    #[test]
    fn test_basic_primitive_block_with_node() {
        // Create a PrimitiveBlock that has a single PrimitiveGroup with one Node
        let mut node = osmformat::Node::new();
        node.set_id(123);
        node.set_lat(456);
        node.set_lon(789);

        let mut group = osmformat::PrimitiveGroup::new();
        group.mut_nodes().push(node);

        let mut pblock = osmformat::PrimitiveBlock::new();
        pblock.mut_primitivegroup().push(group);

        // Serialize
        let result = serialize_primitive_block(pblock);
        assert!(result.is_ok());
        let (blob_header_bytes, blob_bytes) = result.unwrap();

        // The blob_header must have the correct datasize
        let blob_header = fileformat::BlobHeader::parse_from_bytes(&blob_header_bytes)
            .expect("Should parse BlobHeader from bytes");
        assert_eq!(blob_header.get_type(), "OSMData");
        assert_eq!(blob_header.get_datasize() as usize, blob_bytes.len(), 
            "datasize should match the length of the blob bytes");

        // The blob must contain the raw bytes for the node
        let blob = fileformat::Blob::parse_from_bytes(&blob_bytes)
            .expect("Should parse Blob from bytes");
        assert!(blob.has_raw(), "We used raw (no compression) in serialization");
        assert_eq!(blob.get_raw_size() as usize, blob.get_raw().len(),
            "raw_size should match the length of raw data");
    }

    #[test]
    fn test_round_trip_primitive_block() {
        // We can test that the `primitive_block` is actually embedded in the Blob 
        // and can be parsed back out if needed.

        // Let's build a small block with a single string entry in the table
        let mut pblock = osmformat::PrimitiveBlock::new();

        let mut st = osmformat::StringTable::new();
        st.s.push(b"".to_vec()); // index 0 is empty
        st.s.push(b"addr:housenumber".to_vec()); // index 1
        pblock.set_stringtable(st);

        // Now serialize
        let (blob_header_bytes, blob_bytes) = serialize_primitive_block(pblock.clone())
            .expect("Serialization should succeed");

        // Parse out the Blob
        let blob = fileformat::Blob::parse_from_bytes(&blob_bytes)
            .expect("Should parse Blob from bytes");
        let raw_data = blob.get_raw();
        assert!(!raw_data.is_empty(), "Raw data should contain the serialized PrimitiveBlock");

        // Now parse raw_data back into a `PrimitiveBlock`
        let parsed_block = osmformat::PrimitiveBlock::parse_from_bytes(raw_data)
            .expect("Should parse the raw data back into a PrimitiveBlock");
        // Compare some fields
        assert_eq!(parsed_block.get_stringtable().s.len(), 2);
        assert_eq!(parsed_block.get_stringtable().s[1], b"addr:housenumber");

        // Check the BlobHeader
        let blob_header = fileformat::BlobHeader::parse_from_bytes(&blob_header_bytes)
            .expect("Should parse BlobHeader from bytes");
        assert_eq!(blob_header.get_type(), "OSMData");
        assert_eq!(blob_header.get_datasize() as usize, blob_bytes.len());
    }

    #[test]
    fn test_error_in_primitive_block_serialization() {
        // This function handles errors from `primitive_block.write_to_bytes()`.
        // We can simulate a failure by mocking or forcing an error. 
        // The real Protobuf might not fail easily with normal data. We'll do a minimal stub approach.

        struct FailingPrimitiveBlock;
        impl protobuf::Message for FailingPrimitiveBlock {
            fn is_initialized(&self) -> bool { true }
            fn merge_from(&mut self, _is: &mut protobuf::CodedInputStream) -> protobuf::ProtobufResult<()> {
                unimplemented!()
            }
            fn compute_size(&self) -> u32 { 0 }
            fn write_to_with_cached_sizes(&self, _os: &mut protobuf::CodedOutputStream) -> protobuf::ProtobufResult<()> {
                // Force an error
                Err(protobuf::ProtobufError::IoError(io::Error::new(
                    io::ErrorKind::Other, 
                    "Simulated failure in writing primitive block"
                )))
            }
            fn get_cached_size(&self) -> u32 { 0 }
            fn write_to_bytes(&self) -> protobuf::ProtobufResult<Vec<u8>> {
                let mut v = Vec::with_capacity(self.compute_size() as usize);
                {
                    let mut cos = protobuf::CodedOutputStream::vec(&mut v);
                    self.write_to_with_cached_sizes(&mut cos)?;
                }
                Ok(v)
            }
        }

        // Casting it to our osmformat::PrimitiveBlock shape is not feasible, 
        // so we just adapt the function to accept anything that implements the same trait 
        // (or we rename the function). We'll do a minimal demonstration:
        // We'll define a local function that uses the same logic as `serialize_primitive_block`, 
        // but with the trait signature slightly generalized.
        fn serialize_failing_pb(
            failing_pb: &dyn protobuf::Message
        ) -> io::Result<(Vec<u8>, Vec<u8>)> {
            let block_bytes = failing_pb.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("PrimitiveBlock serialization failed: {:?}", e))
            })?;

            let mut blob = fileformat::Blob::new();
            blob.set_raw(block_bytes.clone());
            blob.set_raw_size(block_bytes.len() as i32);

            let blob_bytes = blob.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Blob serialization failed: {:?}", e))
            })?;

            let mut blob_header = fileformat::BlobHeader::new();
            blob_header.set_type("OSMData".to_string());
            blob_header.set_datasize(blob_bytes.len() as i32);

            let blob_header_bytes = blob_header.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("BlobHeader serialization failed: {:?}", e))
            })?;

            Ok((blob_header_bytes, blob_bytes))
        }

        // Now attempt to serialize and confirm we get an error
        let failing_pb = FailingPrimitiveBlock;
        let result = serialize_failing_pb(&failing_pb);
        assert!(
            result.is_err(),
            "Should fail in primitive block serialization"
        );

        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Simulated failure"),
            "Error should contain the forced message. Got: {}", err
        );
    }

    #[test]
    fn test_error_in_blob_serialization() {
        // Similarly, if writing the `Blob` to bytes fails, we should get an error.
        // The real `fileformat::Blob::write_to_bytes()` might not fail easily. 
        // We'll mock the `fileformat::Blob` similarly. 
        // We'll define a local function that calls the same steps, but uses a mock for the second step.

        struct MockBlob { raw: Vec<u8> }
        impl protobuf::Message for MockBlob {
            fn is_initialized(&self) -> bool { true }
            fn merge_from(&mut self, _is: &mut protobuf::CodedInputStream) -> protobuf::ProtobufResult<()> {
                unimplemented!()
            }
            fn compute_size(&self) -> u32 { 0 }
            fn write_to_with_cached_sizes(&self, _os: &mut protobuf::CodedOutputStream) -> protobuf::ProtobufResult<()> {
                // Force an error to simulate a failure in the second step
                Err(protobuf::ProtobufError::IoError(io::Error::new(
                    io::ErrorKind::Other, 
                    "Simulated blob serialization error"
                )))
            }
            fn get_cached_size(&self) -> u32 { 0 }
        }

        // We'll forcibly skip the first step's logic. 
        // The real function calls `primitive_block.write_to_bytes()` first. We'll pretend that succeeded
        let block_bytes = vec![1,2,3]; // some placeholder
        let mut blob = MockBlob { raw: block_bytes.clone() };

        // We'll define a local function that tries to do the second step with our mock
        fn attempt_blob_serialization(mock_blob: &mut MockBlob) -> io::Result<Vec<u8>> {
            // `mock_blob.write_to_bytes()` => forced error
            let bytes = mock_blob.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Data Blob serialization failed: {:?}", e))
            })?;
            Ok(bytes)
        }

        // Attempt and confirm error
        let result = attempt_blob_serialization(&mut blob);
        assert!(
            result.is_err(),
            "Should fail with our forced mock error"
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Simulated blob serialization error"),
            "Expected forced message, got: {}", err
        );
    }

    #[test]
    fn test_error_in_blobheader_serialization() {
        // The final step is the `BlobHeader` writing. We'll mock that similarly. 
        // This is quite contrived because the real `fileformat::BlobHeader::write_to_bytes()` 
        // seldom fails unless we forcibly fail or run out of memory.

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
                    "Simulated blobheader serialization failure"
                )))
            }
            fn get_cached_size(&self) -> u32 { 0 }
        }

        // We'll define a local function that tries to do that final step
        fn attempt_blobheader_serialization(mock_header: &MockBlobHeader) -> io::Result<Vec<u8>> {
            let bytes = mock_header.write_to_bytes().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("BlobHeader serialization failed: {:?}", e))
            })?;
            Ok(bytes)
        }

        let mock_header = MockBlobHeader;
        let result = attempt_blobheader_serialization(&mock_header);
        assert!(
            result.is_err(),
            "Should fail with forced mock error"
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Simulated blobheader serialization failure"),
            "Expected forced message, got: {}", err
        );
    }
}
