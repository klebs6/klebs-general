crate::ix!();

#[derive(Clone, Debug)]
pub struct ShouldFailAttr {
    message: Option<String>,
}

impl ShouldFailAttr {

    pub fn new(message: Option<String>) -> Self {
        Self { message }
    }
}

impl MaybeHasExpectedFailureMessage for ShouldFailAttr {

    fn expected_failure_message(&self) -> Option<Cow<'_, str>> {
        self.message.as_deref().map(Cow::Borrowed)
    }
}

impl TryFrom<syn::Attribute> for ShouldFailAttr {

    type Error = ShouldFailAttrError;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        if !attr.path().is_ident("should_fail") {
            return Err(ShouldFailAttrError::NotShouldFailAttr);
        }

        // Check if the attribute has no arguments (no parentheses)
        if let syn::Meta::Path(_) = attr.meta {
            return Ok(ShouldFailAttr::new(None));
        }

        // Use the extracted parser and error mapping logic
        attr.parse_args_with(parse::parse_should_fail_args).map_err(parse::map_parse_error)
    }
}

mod parse {

    use super::*;

    pub fn parse_should_fail_args(input: ParseStream) -> Result<ShouldFailAttr, syn::Error> {
        let mut parsed_message = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;

            if ident == "message" {
                parsed_message = parse_message(input)?;
            } else {
                return Err(syn::Error::new(ident.span(), "unknown key, expected `message`"));
            }

            // Consume the comma if present
            consume_comma_if_present(input)?;
        }

        Ok(ShouldFailAttr::new(parsed_message))
    }

    fn parse_message(input: ParseStream) -> Result<Option<String>, syn::Error> {
        input.parse::<Token![=]>()?;
        let lit: syn::Lit = input.parse()?;
        if let syn::Lit::Str(lit_str) = lit {
            Ok(Some(lit_str.value()))
        } else {
            Err(syn::Error::new(lit.span(), "expected string literal for `message` value"))
        }
    }

    fn consume_comma_if_present(input: ParseStream) -> Result<(), syn::Error> {
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        Ok(())
    }

    pub fn map_parse_error(err: syn::Error) -> ShouldFailAttrError {
        let msg = err.to_string();
        if msg.contains("expected string literal for `message` value") {
            ShouldFailAttrError::InvalidExpectedValueFormat
        } else if msg.contains("expected `=`") || msg.contains("unexpected end of input") {
            ShouldFailAttrError::ExpectedValueMissing
        } else {
            ShouldFailAttrError::MetaParseError(err)
        }
    }
}

#[cfg(test)]
mod test_parse_should_fail_attribute {
    use super::*;
    use syn::{Attribute, parse::Parser};
    use proc_macro2::TokenStream;

    fn parse_attribute(input: &str) -> syn::Result<Attribute> {
        // Parse the input string into tokens
        let tokens: TokenStream = syn::parse_str(input)?;
        // Parse the tokens into a vector of attributes
        let attrs = Attribute::parse_outer.parse2(tokens.clone())?;
        // Ensure there is exactly one attribute
        if attrs.len() == 1 {
            Ok(attrs.into_iter().next().unwrap())
        } else {
            Err(syn::Error::new_spanned(
                tokens,
                format!("Expected exactly one attribute, found {}", attrs.len()),
            ))
        }
    }

    #[test]
    fn test_parse_should_fail_missing_equal_sign() {
        let input = r#"#[should_fail(message "Test failed")]"#; // Invalid, missing `=`
        let attr = parse_attribute(input).unwrap();
        let result = ShouldFailAttr::try_from(attr);
        assert!(
            result.is_err(),
            "Expected ShouldFailAttr::try_from to fail due to missing '='"
        );
        // Optionally, check for the specific error type
        assert!(matches!(
                result.err().unwrap(),
                ShouldFailAttrError::ExpectedValueMissing | ShouldFailAttrError::MetaParseError(_)
        ));
    }

    #[test]
    fn test_parse_should_fail() {
        let input = r#"#[should_fail(message = "Test failed")]"#;
        let attr = parse_attribute(input).unwrap();

        let should_fail_attr = ShouldFailAttr::try_from(attr).unwrap();

        assert_eq!(should_fail_attr.message.as_deref(), Some("Test failed"));
    }

    #[test]
    fn test_parse_should_fail_valid() {
        let input = r#"#[should_fail(message = "Test failed")]"#;
        let attr = parse_attribute(input).unwrap();

        let should_fail_attr = ShouldFailAttr::try_from(attr).unwrap();

        assert_eq!(should_fail_attr.message.as_deref(), Some("Test failed"));
    }

    #[test]
    fn test_parse_should_fail_with_no_args() {
        let input = r#"#[should_fail]"#;
        let attr = parse_attribute(input).unwrap();

        let should_fail_attr = ShouldFailAttr::try_from(attr).unwrap();

        assert_eq!(should_fail_attr.message, None);
    }

    #[test]
    fn test_parse_should_fail_with_only_message() {
        let input = r#"#[should_fail(message = "Test failed")]"#;
        let attr = parse_attribute(input).unwrap();

        let should_fail_attr = ShouldFailAttr::try_from(attr).unwrap();

        assert_eq!(should_fail_attr.message.as_deref(), Some("Test failed"));
    }

    #[test]
    fn test_parse_should_fail_with_multiple_commas() {
        let input = r#"#[should_fail(message = "Test failed",)]"#;
        let attr = parse_attribute(input).unwrap();

        let should_fail_attr = ShouldFailAttr::try_from(attr).unwrap();

        assert_eq!(should_fail_attr.message.as_deref(), Some("Test failed"));
    }

    #[test]
    fn test_parse_should_fail_invalid_message_type() {
        let input = r#"#[should_fail(message = 123)]"#; // Invalid, expecting string
        let attr = parse_attribute(input).unwrap();

        let result = ShouldFailAttr::try_from(attr);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ShouldFailAttrError::InvalidExpectedValueFormat
        ));
    }

    #[test]
    fn test_parse_should_fail_unknown_key() {
        let input = r#"#[should_fail(message = "Test failed", unknown_key = true)]"#; // Invalid, unknown key
        let attr = parse_attribute(input).unwrap();

        let result = ShouldFailAttr::try_from(attr);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ShouldFailAttrError::MetaParseError(_)
        ));
    }

    #[test]
    fn test_parse_should_fail_unexpected_end_of_input() {
        let input = r#"#[should_fail(message = ]"#; // Invalid, unexpected end
        let result = parse_attribute(input);
        assert!(result.is_err(), "Expected parsing to fail due to unexpected end of input");
    }

    #[test]
    fn test_should_fail_attr_with_unknown_key() {
        let input = r#"#[should_fail(unknown = "value")]"#;
        let attr = parse_attribute(input).unwrap();
        let result = ShouldFailAttr::try_from(attr);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ShouldFailAttrError::MetaParseError(_)
        ));
    }
}
