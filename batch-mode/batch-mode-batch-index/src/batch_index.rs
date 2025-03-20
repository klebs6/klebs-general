// ---------------- [ File: src/batch_index.rs ]
crate::ix!();

/// Represents the type of index found in the file names.
#[derive(Serialize,Deserialize,Debug,Clone,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub enum BatchIndex {
    Usize(usize),
    Uuid(Uuid),
}

impl BatchIndex {

    pub fn new() -> Self {
        BatchIndex::Uuid(Uuid::new_v4())
    }

    pub fn from_uuid_str(x: &str) -> Result<Self,UuidParseError> {
        Ok(BatchIndex::Uuid(Uuid::parse_str(x)?))
    }
}

impl From<u64> for BatchIndex {
    fn from(value: u64) -> Self {
        BatchIndex::Usize(value as usize)
    }
}

impl BatchIndex {
    /// Returns `Some(u64)` if this index is a `Usize(u)`, else returns `None` if it’s a UUID.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            BatchIndex::Usize(u) => Some(*u as u64),
            BatchIndex::Uuid(_)  => None,
        }
    }
}

impl Display for BatchIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            BatchIndex::Usize(value) => write!(f, "{}", value),
            BatchIndex::Uuid(value) => write!(f, "{}", value),
        }
    }
}

#[cfg(test)]
mod verify_batch_index {
    use super::*;

    #[traced_test]
    fn verify_file_pattern_for_usize() {
        info!("Starting test: verify_file_pattern_for_usize (BatchIndex)");
        let index = BatchIndex::Usize(4);
        let regex = index.file_pattern();

        debug!("Asserting valid matches for a specific integer-based index...");
        assert!(regex.is_match("batch_input_4.jsonl"));
        assert!(regex.is_match("batch_output_4.jsonl"));
        assert!(regex.is_match("batch_error_4.jsonl"));

        debug!("Asserting invalid matches for a specific integer-based index...");
        assert!(!regex.is_match("batch_input_5.jsonl"), "Should not match 5");
        assert!(!regex.is_match("batch_unknown_4.jsonl"), "Unknown type should fail");
        assert!(!regex.is_match("batch_input_4.txt"), "Wrong extension");
        assert!(!regex.is_match("input_batch_4.jsonl"), "Wrong prefix order");

        info!("Finished test: verify_file_pattern_for_usize (BatchIndex)");
    }

    #[traced_test]
    fn verify_file_pattern_for_uuid() {
        info!("Starting test: verify_file_pattern_for_uuid (BatchIndex)");
        let index = BatchIndex::from_uuid_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let regex = index.file_pattern();

        debug!("Asserting valid matches for a specific UUID-based index...");
        assert!(regex.is_match("batch_input_550e8400-e29b-41d4-a716-446655440000.jsonl"));
        assert!(regex.is_match("batch_output_550e8400-e29b-41d4-a716-446655440000.jsonl"));
        assert!(regex.is_match("batch_error_550e8400-e29b-41d4-a716-446655440000.jsonl"));

        debug!("Asserting invalid matches for a specific UUID-based index...");
        assert!(!regex.is_match("batch_input_123e4567-e89b-12d3-a456-426655440000.jsonl"), "Mismatched UUID");
        assert!(!regex.is_match("batch_error_550e8400e29b41d4a716446655440000.jsonl"), "Missing dashes");
        assert!(!regex.is_match("batch_error_550e8400-e29b-41d4-a716-446655440000.txt"), "Wrong extension");

        info!("Finished test: verify_file_pattern_for_uuid (BatchIndex)");
    }

    #[traced_test]
    fn verify_file_pattern_edge_cases() {
        info!("Starting test: verify_file_pattern_edge_cases (BatchIndex)");

        debug!("Checking edge cases for integer-based index 0...");
        let regex_usize = BatchIndex::Usize(0).file_pattern();
        assert!(regex_usize.is_match("batch_input_0.jsonl"), "Should match 0");
        assert!(regex_usize.is_match("batch_error_0.jsonl"), "Should match error file with 0");
        assert!(!regex_usize.is_match("batch_input_.jsonl"), "Empty numeric part should fail");

        debug!("Checking edge cases for specific UUID...");
        let regex_uuid = BatchIndex::from_uuid_str("123e4567-e89b-12d3-a456-426655440000").unwrap().file_pattern();
        assert!(regex_uuid.is_match("batch_input_123e4567-e89b-12d3-a456-426655440000.jsonl"));
        assert!(!regex_uuid.is_match("batch_input_123e4567e89b12d3a456426655440000.jsonl"), "Missing dashes");
        assert!(!regex_uuid.is_match("batch_input_123e4567-e89b-12d3-a456-426655440000.txt"), "Wrong extension");

        info!("Finished test: verify_file_pattern_edge_cases (BatchIndex)");
    }

    #[test]
    fn test_generate_batch_file_regex_usize() {
        let index = BatchIndex::Usize(4);
        let regex = index.file_pattern();
        assert!(regex.is_match("batch_input_4.jsonl"));
        assert!(regex.is_match("batch_output_4.jsonl"));
        assert!(regex.is_match("batch_error_4.jsonl"));
        assert!(!regex.is_match("batch_input_5.jsonl"));
        assert!(!regex.is_match("batch_unknown_4.jsonl"));
        assert!(!regex.is_match("batch_input_4.txt"));
        assert!(!regex.is_match("input_batch_4.jsonl"));
    }

    #[test]
    fn test_generate_batch_file_regex_uuid() {
        let index = BatchIndex::from_uuid_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let regex = index.file_pattern();
        assert!(regex.is_match("batch_input_550e8400-e29b-41d4-a716-446655440000.jsonl"));
        assert!(regex.is_match("batch_output_550e8400-e29b-41d4-a716-446655440000.jsonl"));
        assert!(regex.is_match("batch_error_550e8400-e29b-41d4-a716-446655440000.jsonl"));
        assert!(!regex.is_match("batch_input_123e4567-e89b-12d3-a456-426655440000.jsonl"));
        assert!(!regex.is_match("batch_error_zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz.jsonl"));
        assert!(!regex.is_match("batch_output_550e8400e29b41d4a716446655440000.jsonl"));
        assert!(!regex.is_match("batch_error_550e8400-e29b-41d4-a716-446655440000.txt"));
    }

    #[test]
    fn test_generate_batch_file_regex_edge_cases() {
        // Testing edge cases for both Usize and UUID patterns
        let regex_usize = BatchIndex::Usize(0).file_pattern();
        assert!(regex_usize.is_match("batch_input_0.jsonl"));
        assert!(regex_usize.is_match("batch_output_0.jsonl"));
        assert!(regex_usize.is_match("batch_error_0.jsonl"));
        assert!(!regex_usize.is_match("batch_input_1.jsonl"));
        assert!(!regex_usize.is_match("batch_input_.json"));

        let regex_uuid = BatchIndex::from_uuid_str("123e4567-e89b-12d3-a456-426655440000").unwrap().file_pattern();
        assert!(regex_uuid.is_match("batch_input_123e4567-e89b-12d3-a456-426655440000.jsonl"));
        assert!(regex_uuid.is_match("batch_output_123e4567-e89b-12d3-a456-426655440000.jsonl"));
        assert!(regex_uuid.is_match("batch_error_123e4567-e89b-12d3-a456-426655440000.jsonl"));
        assert!(!regex_uuid.is_match("batch_input_123e4567-e89b-12d3-a456-42665544000.jsonl"));
        assert!(!regex_uuid.is_match("batch_input_123e4567e89b12d3a456426655440000.jsonl"));
        assert!(!regex_uuid.is_match("batch_input_123e4567-e89b-12d3-a456-426655440000.txt"));
    }
}
