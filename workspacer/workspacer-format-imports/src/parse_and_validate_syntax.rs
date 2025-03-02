// ---------------- [ File: src/parse_and_validate_syntax.rs ]
crate::ix!();

pub fn parse_and_validate_syntax(old_text: &str) -> Result<SourceFile, SortAndFormatImportsError> {
    info!(
        "parse_and_validate_syntax => start; old_text length={}",
        old_text.len()
    );
    let parse = SourceFile::parse(old_text, Edition::Edition2021);
    let file = parse.tree();

    if !parse.errors().is_empty() {
        error!(
            "parse_and_validate_syntax => errors found => count={}",
            parse.errors().len()
        );
        let joined = parse
            .errors()
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(SortAndFormatImportsError::RaApParseError {
            parse_errors: joined,
        });
    }

    debug!("parse_and_validate_syntax => no parse errors => returning Ok(file)");
    info!("parse_and_validate_syntax => done");
    Ok(file)
}

#[cfg(test)]
mod test_parse_and_validate_syntax {
    use super::*;

    /// 1) If the code is valid => returns Ok(SourceFile).
    #[test]
    fn test_valid_syntax() {
        info!("test_valid_syntax => start");
        let src = "fn main() {}";
        let result = parse_and_validate_syntax(src);
        assert!(result.is_ok(), "Expected Ok for valid syntax");
        info!("test_valid_syntax => success");
    }

    /// 2) If the code has parse errors => returns Err(...).
    #[test]
    fn test_invalid_syntax() {
        info!("test_invalid_syntax => start");
        let src = "fn main( {}"; // missing paren
        let result = parse_and_validate_syntax(src);
        assert!(result.is_err(), "Expected error for invalid syntax");
        if let Err(SortAndFormatImportsError::RaApParseError { parse_errors }) = result {
            debug!("Got parse_errors => {:?}", parse_errors);
            assert!(
                parse_errors.contains("expected"),
                "Should mention parse error detail"
            );
        } else {
            panic!("Expected RaApParseError variant");
        }
        info!("test_invalid_syntax => success");
    }
}
