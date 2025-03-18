// ---------------- [ File: src/batch_index_type.rs ]
crate::ix!();

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub enum BatchIndexType {
    Usize,
    Uuid,
}

impl From<&BatchIndex> for BatchIndexType {

    fn from(x: &BatchIndex) -> BatchIndexType {
        match x {
            BatchIndex::Usize(..) => BatchIndexType::Usize,
            BatchIndex::Uuid(..) => BatchIndexType::Uuid,
        }
    }
}

#[cfg(test)]
mod verify_batch_index_type {
    use super::*;

    #[traced_test]
    fn verify_file_pattern_for_usize() {
        info!("Starting test: verify_file_pattern_for_usize");
        let index_type = BatchIndexType::Usize;
        let regex = index_type.file_pattern();

        debug!("Asserting valid matches for integer-based batch file naming...");
        assert!(regex.is_match("batch_input_1.jsonl"), "Should match 1");
        assert!(regex.is_match("batch_output_123.jsonl"), "Should match 123");
        assert!(regex.is_match("batch_error_42.jsonl"), "Should match 42");
        assert!(regex.is_match("batch_input_0000.jsonl"), "Leading zeros are valid");
        assert!(regex.is_match("batch_output_999999999.jsonl"), "Should match large number");

        debug!("Asserting invalid matches for integer-based batch file naming...");
        assert!(!regex.is_match("batch_input_a.jsonl"), "Should not match alpha");
        assert!(!regex.is_match("batch_unknown_1.jsonl"), "Unknown type should fail");
        assert!(!regex.is_match("batch_input_1.txt"), "Wrong extension should fail");
        assert!(!regex.is_match("input_batch_1.jsonl"), "Wrong prefix order should fail");
        assert!(!regex.is_match("batch_input_-1.jsonl"), "Negative not valid for usize");

        info!("Finished test: verify_file_pattern_for_usize");
    }

    #[traced_test]
    fn verify_file_pattern_for_uuid() {
        info!("Starting test: verify_file_pattern_for_uuid");
        let index_type = BatchIndexType::Uuid;
        let regex = index_type.file_pattern();

        debug!("Asserting valid matches for UUID-based batch file naming...");
        assert!(regex.is_match("batch_input_550e8400-e29b-41d4-a716-446655440000.jsonl"));
        assert!(regex.is_match("batch_output_123e4567-e89b-12d3-a456-426655440000.jsonl"));

        debug!("Asserting invalid matches for UUID-based batch file naming...");
        assert!(!regex.is_match("batch_input_550e8400e29b41d4a716446655440000.jsonl"), "Missing dashes");
        assert!(!regex.is_match("batch_error_zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz.jsonl"), "Non-hex characters");
        assert!(!regex.is_match("batch_output_123e4567-e89b-12d3-a456-426655440000.txt"), "Wrong extension");

        info!("Finished test: verify_file_pattern_for_uuid");
    }

    #[traced_test]
    fn verify_file_pattern_edge_cases() {
        info!("Starting test: verify_file_pattern_edge_cases");
        let regex_usize = BatchIndexType::Usize.file_pattern();
        let regex_uuid = BatchIndexType::Uuid.file_pattern();

        debug!("Checking edge cases for Usize pattern...");
        assert!(regex_usize.is_match("batch_input_0.jsonl"), "Zero should be valid");
        assert!(!regex_usize.is_match("batch_input_.jsonl"), "Empty numeric part should fail");
        assert!(!regex_usize.is_match("batch_input_0.json"), "Wrong extension should fail");

        debug!("Checking edge cases for UUID pattern...");
        assert!(regex_uuid.is_match("batch_input_123e4567-e89b-12d3-a456-426655440000.jsonl"));
        assert!(!regex_uuid.is_match("batch_input_123e4567-e89b-12d3-a456-42665544000.jsonl"), "Too short");
        assert!(!regex_uuid.is_match("batch_input_123e4567e89b12d3a456426655440000.jsonl"), "Missing dashes");

        info!("Finished test: verify_file_pattern_edge_cases");
    }

    #[test]
    fn test_generate_batch_file_regex_usize() {
        let index_type = BatchIndexType::Usize;
        let regex = index_type.file_pattern();
        assert!(regex.is_match("batch_input_1.jsonl"));
        assert!(regex.is_match("batch_output_123.jsonl"));
        assert!(regex.is_match("batch_error_42.jsonl"));
        assert!(!regex.is_match("batch_input_a.jsonl"));
        assert!(!regex.is_match("batch_unknown_1.jsonl"));
        assert!(!regex.is_match("batch_input_1.txt"));
        assert!(!regex.is_match("input_batch_1.jsonl"));
    }

    #[test]
    fn test_generate_batch_file_regex_uuid() {
        let index_type = BatchIndexType::Uuid;
        let regex = index_type.file_pattern();
        assert!(regex.is_match("batch_input_550e8400-e29b-41d4-a716-446655440000.jsonl"));
        assert!(regex.is_match("batch_output_123e4567-e89b-12d3-a456-426655440000.jsonl"));
        assert!(!regex.is_match("batch_input_550e8400e29b41d4a716446655440000.jsonl"));
        assert!(!regex.is_match("batch_error_zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz.jsonl"));
        assert!(!regex.is_match("batch_output_123e4567e89b12d3a456426655440000.jsonl"));
        assert!(!regex.is_match("batch_error_123e4567-e89b-12d3-a456-426655440000.txt"));
    }

    #[test]
    fn test_generate_batch_file_regex_edge_cases() {
        // Testing edge cases for both Usize and UUID patterns
        let regex_usize = BatchIndexType::Usize.file_pattern();
        assert!(regex_usize.is_match("batch_input_9999.jsonl"));
        assert!(regex_usize.is_match("batch_input_0000.jsonl")); // Accepting leading zeros as valid
        assert!(!regex_usize.is_match("batch_input_"));
        assert!(!regex_usize.is_match("batch_input_9999.json"));

        let regex_uuid = BatchIndexType::Uuid.file_pattern();
        assert!(regex_uuid.is_match("batch_input_123e4567-e89b-12d3-a456-426655440000.jsonl"));
        assert!(!regex_uuid.is_match("batch_input_123e4567-e89b-12d3-a456-42665544000.jsonl"));
        assert!(!regex_uuid.is_match("batch_input_123e4567e89b12d3a456426655440000.jsonl"));
        assert!(!regex_uuid.is_match("batch_input_123e4567-e89b-12d3-a456-426655440000.txt"));
    }
}
