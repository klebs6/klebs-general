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

impl Display for BatchIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            BatchIndex::Usize(value) => write!(f, "{}", value),
            BatchIndex::Uuid(value) => write!(f, "{}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
