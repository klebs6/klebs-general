// ---------------- [ File: src/parse_and_validate_syntax.rs ]
crate::ix!();

// -------------------------------------------------------------------------
// 1) Parse & validate the source text with RA-AP syntax
// -------------------------------------------------------------------------
pub fn parse_and_validate_syntax(old_text: &str) -> Result<SourceFile, SortAndFormatImportsError> {
    let parse = SourceFile::parse(old_text, Edition::Edition2021);
    let file = parse.tree();

    if !parse.errors().is_empty() {
        let joined = parse
            .errors()
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(SortAndFormatImportsError::RaApParseError { parse_errors: joined });
    }
    Ok(file)
}

#[cfg(test)]
mod test_parse_and_validate_syntax {
    use super::*;

    /// 1) If the code is valid => returns Ok(SourceFile).
    #[test]
    fn test_valid_syntax() {
        let src = "fn main() {}";
        let result = parse_and_validate_syntax(src);
        assert!(result.is_ok(), "Expected Ok for valid syntax");
    }

    /// 2) If the code has parse errors => returns Err(...).
    #[test]
    fn test_invalid_syntax() {
        let src = "fn main( {}"; // missing paren
        let result = parse_and_validate_syntax(src);
        assert!(result.is_err(), "Expected error for invalid syntax");
        if let Err(SortAndFormatImportsError::RaApParseError { parse_errors }) = result {
            assert!(parse_errors.contains("expected"), "Should mention parse error detail");
        } else {
            panic!("Expected RaApParseError variant");
        }
    }

    // add more tests if you wish...
}
