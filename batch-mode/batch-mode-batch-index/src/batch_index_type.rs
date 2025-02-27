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
mod tests {
    use super::*;
    use regex::Regex;

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
