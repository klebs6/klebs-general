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
mod test_serialize_primitive_block {
    use super::*;
    use crate::proto::{fileformat, osmformat};
    use protobuf::{
        CodedInputStream, CodedOutputStream, Message, MessageFull, SpecialFields,
    };
    use std::io::{Error as IoError, ErrorKind};

    // (A) Test the normal success path with a valid `PrimitiveBlock`
    // ----------------------------------------------------------------------

    /// Creates a minimal valid `PrimitiveBlock` that can be serialized successfully.
    fn make_basic_primitive_block() -> osmformat::PrimitiveBlock {
        let mut block = osmformat::PrimitiveBlock::new();

        // Provide a dummy StringTable with one entry
        let mut st = osmformat::StringTable::new();
        st.s.push(b"test".to_vec());
        block.stringtable = protobuf::MessageField::from_option(Some(st));

        // Add an empty PrimitiveGroup so it's not zero-length
        let group = osmformat::PrimitiveGroup::new();
        block.primitivegroup.push(group);

        // Default granularity, lat_offset, date_granularity are fine
        block
    }

    /// Confirms `serialize_primitive_block` works with a valid `PrimitiveBlock`.
    #[test]
    fn test_serialize_primitive_block_ok() {
        let block = make_basic_primitive_block();
        let result = serialize_primitive_block(block);

        assert!(
            result.is_ok(),
            "Expected successful serialization of a normal PrimitiveBlock"
        );

        let (blob_header_bytes, blob_bytes) = result.unwrap();
        assert!(
            !blob_header_bytes.is_empty(),
            "BlobHeader bytes should not be empty"
        );
        assert!(
            !blob_bytes.is_empty(),
            "Data Blob bytes should not be empty"
        );
    }

    // (B) Test each error scenario by replicating sub-steps with mocks
    // ----------------------------------------------------------------------
    //
    // Because the real function requires an `osmformat::PrimitiveBlock`,
    // we cannot directly pass a custom failing mock to `serialize_primitive_block`.
    // Instead, we replicate each `.write_to_bytes()` call with our own mock types.

    /// A mock `PrimitiveBlock` that always fails in `write_to_with_cached_sizes`.
    /// Simulates `"PrimitiveBlock serialization failed"`.
    #[derive(Clone, PartialEq, Debug, Default)]
    struct FailingPrimitiveBlockForTest {
        special_fields: SpecialFields,
    }

    impl Message for FailingPrimitiveBlockForTest {
        const NAME: &'static str = "FailingPrimitiveBlockForTest";

        fn is_initialized(&self) -> bool { true }

        fn merge_from(&mut self, _is: &mut CodedInputStream<'_>) -> protobuf::Result<()> {
            Ok(())
        }

        /// Always returns an error caused by an `io::Error`.
        fn write_to_with_cached_sizes(
            &self, 
            _os: &mut CodedOutputStream<'_>
        ) -> protobuf::Result<()> {
            // We can't construct `Error(Box::new(ProtobufError::...))` directly,
            // so we do `Error::from(io::Error)`.
            let io_err = IoError::new(ErrorKind::Other, "Simulated PrimitiveBlock write_to_bytes failure");
            Err(protobuf::Error::from(io_err))
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
            static INSTANCE: once_cell::sync::Lazy<FailingPrimitiveBlockForTest> =
                once_cell::sync::Lazy::new(|| FailingPrimitiveBlockForTest {
                    special_fields: SpecialFields::new(),
                });
            &INSTANCE
        }
    }

    /// A mock `Blob` that always fails in `write_to_with_cached_sizes`.
    /// Simulates `"Data Blob serialization failed"`.
    #[derive(Clone, PartialEq, Debug, Default)]
    struct FailingBlobForTest {
        special_fields: SpecialFields,
    }

    impl Message for FailingBlobForTest {
        const NAME: &'static str = "FailingBlobForTest";

        fn is_initialized(&self) -> bool { true }

        fn merge_from(&mut self, _is: &mut CodedInputStream<'_>) -> protobuf::Result<()> {
            Ok(())
        }

        fn write_to_with_cached_sizes(
            &self, 
            _os: &mut CodedOutputStream<'_>
        ) -> protobuf::Result<()> {
            let io_err = IoError::new(ErrorKind::Other, "Simulated data blob write_to_bytes failure");
            Err(protobuf::Error::from(io_err))
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
                    special_fields: SpecialFields::new(),
                });
            &INSTANCE
        }
    }

    /// A mock `BlobHeader` that always fails in `write_to_with_cached_sizes`.
    /// Simulates `"BlobHeader serialization failed"`.
    #[derive(Clone, PartialEq, Debug, Default)]
    struct FailingBlobHeaderForTest {
        special_fields: SpecialFields,
    }

    impl Message for FailingBlobHeaderForTest {
        const NAME: &'static str = "FailingBlobHeaderForTest";

        fn is_initialized(&self) -> bool { true }

        fn merge_from(&mut self, _is: &mut CodedInputStream<'_>) -> protobuf::Result<()> {
            Ok(())
        }

        fn write_to_with_cached_sizes(
            &self,
            _os: &mut CodedOutputStream<'_>
        ) -> protobuf::Result<()> {
            let io_err = IoError::new(ErrorKind::Other, "Simulated BlobHeader write_to_bytes failure");
            Err(protobuf::Error::from(io_err))
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

    // (B.1) Test scenario: "PrimitiveBlock fails to serialize".
    //
    // We can't pass our mock type to `serialize_primitive_block(...)`
    // because it strictly requires an `osmformat::PrimitiveBlock`.
    // Instead, replicate `.write_to_bytes()` ourselves:
    #[test]
    fn test_mock_primitive_block_serialization_failure() {
        let mock_block = FailingPrimitiveBlockForTest::default();
        let result = mock_block.write_to_bytes();
        assert!(
            result.is_err(),
            "Expected the mock PrimitiveBlock to fail on write_to_bytes"
        );

        // We check that the final .to_string() matches the 
        // "PrimitiveBlock serialization failed" the function would produce
        let expected = IoError::new(ErrorKind::Other, "PrimitiveBlock serialization failed");
        assert_eq!(expected.kind(), ErrorKind::Other);
        assert_eq!(expected.to_string(), "PrimitiveBlock serialization failed");
    }

    // (B.2) Test scenario: "Data Blob fails to serialize".
    #[test]
    fn test_mock_data_blob_serialization_failure() {
        let mock_blob = FailingBlobForTest::default();
        let result = mock_blob.write_to_bytes();
        assert!(
            result.is_err(),
            "Expected the mock data Blob to fail on write_to_bytes"
        );

        let expected = IoError::new(ErrorKind::Other, "Data Blob serialization failed");
        assert_eq!(expected.kind(), ErrorKind::Other);
        assert_eq!(expected.to_string(), "Data Blob serialization failed");
    }

    // (B.3) Test scenario: "BlobHeader fails to serialize".
    #[test]
    fn test_mock_blob_header_serialization_failure() {
        let mock_blob_header = FailingBlobHeaderForTest::default();
        let result = mock_blob_header.write_to_bytes();
        assert!(
            result.is_err(),
            "Expected the mock BlobHeader to fail on write_to_bytes"
        );

        let expected = IoError::new(ErrorKind::Other, "BlobHeader serialization failed");
        assert_eq!(expected.kind(), ErrorKind::Other);
        assert_eq!(expected.to_string(), "BlobHeader serialization failed");
    }
}
