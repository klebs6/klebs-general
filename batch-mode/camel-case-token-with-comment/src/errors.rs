// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum TokenParseError {
        InvalidTokenName,
        InvalidTokenLine(String),
        MissingTokenNameField,

        #[cmp_neq]
        IoError(std::io::Error),
    }
}

#[cfg(test)]
mod test_token_parse_error {
    use super::*;

    #[traced_test]
    fn test_token_parse_error_variants() {
        tracing::info!("Testing TokenParseError variants");

        let e1 = TokenParseError::InvalidTokenName;
        let e2 = TokenParseError::InvalidTokenLine("some line".to_string());
        let e3 = TokenParseError::MissingTokenNameField;
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "oh no!");
        let e4 = TokenParseError::IoError(io_err);

        match e1 {
            TokenParseError::InvalidTokenName => {
                tracing::info!("Matched InvalidTokenName successfully");
            }
            _ => panic!("Should have matched InvalidTokenName"),
        }

        match e2 {
            TokenParseError::InvalidTokenLine(ref s) if s == "some line" => {
                tracing::info!("Matched InvalidTokenLine with expected content");
            }
            _ => panic!("Should have matched InvalidTokenLine"),
        }

        match e3 {
            TokenParseError::MissingTokenNameField => {
                tracing::info!("Matched MissingTokenNameField successfully");
            }
            _ => panic!("Should have matched MissingTokenNameField"),
        }

        match e4 {
            TokenParseError::IoError(ref err) => {
                assert_eq!(format!("{}", err), "oh no!");
                tracing::info!("Matched IoError as expected");
            }
            _ => panic!("Should have matched IoError"),
        }
    }
}
