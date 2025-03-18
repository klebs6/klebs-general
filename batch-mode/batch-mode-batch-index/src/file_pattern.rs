// ---------------- [ File: src/file_pattern.rs ]
crate::ix!();

pub trait FilePattern {

    fn file_pattern(&self) -> Regex;
}

impl FilePattern for BatchIndex {

    /// Constructs a regex pattern for matching batch filenames based on the provided `BatchIndex`.
    ///
    /// # Arguments
    /// * `self` - The `BatchIndex` (either Usize or Uuid) which is used to determine the regex pattern.
    ///
    /// # Returns
    /// * `Regex` - A compiled regex pattern for matching batch file names.
    ///
    /// # Panics
    /// This function will panic if the regex pattern is invalid.
    fn file_pattern(&self) -> Regex {
        let index_pattern = match self {
            BatchIndex::Usize(value) => format!("{}", value),
            BatchIndex::Uuid(value) => format!("{}", value),
        };

        Regex::new(&format!(
            r"^batch_(input|output|error|metadata)_{index_pattern}\.jsonl$",
            index_pattern = index_pattern
        ))
        .expect("Invalid regex pattern")
    }
}

impl FilePattern for BatchIndexType {

    /// Constructs a regex pattern for matching batch filenames based on the provided `BatchIndex`.
    ///
    /// # Arguments
    /// * `index_type` - The `BatchIndex` (either Usize or Uuid) which is used to determine the regex pattern.
    ///
    /// # Returns
    /// * `Regex` - A compiled regex pattern for matching batch file names.
    ///
    /// # Panics
    /// This function will panic if the regex pattern is invalid.
    fn file_pattern(&self) -> Regex {
        let index_pattern = match self {
            BatchIndexType::Usize => r"\d+",
            BatchIndexType::Uuid => r"[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",
        };

        Regex::new(&format!(
            r"batch_(input|output|error|metadata)_{index_pattern}\.jsonl$",
            index_pattern = index_pattern
        )).expect("Invalid regex pattern")
    }
}
