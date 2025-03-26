crate::ix!();

/// The struct your code already used
#[derive(Clone, Debug)]
pub struct ShouldFailAttr {
    /// If `#[should_fail(message = "...")]` was present, we store Some("...").
    message: Option<String>,
}

impl ShouldFailAttr {
    pub fn new(message: Option<String>) -> Self {
        Self { message }
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

/// Let other code get the “message” for checks
impl MaybeHasExpectedFailureMessage for ShouldFailAttr {
    fn expected_failure_message(&self) -> Option<Cow<'_, str>> {
        self.message.as_deref().map(Cow::Borrowed)
    }
}

impl TryFrom<Attribute> for ShouldFailAttr {
    type Error = ShouldFailAttrError;

    fn try_from(attr: Attribute) -> Result<Self, Self::Error> {
        // Must be named `#[should_fail]`
        if !attr.path().is_ident("should_fail") {
            return Err(ShouldFailAttrError::NotShouldFailAttr);
        }

        // Let’s examine the attribute’s meta:
        let meta = attr.meta;

        match meta {
            // e.g. `#[should_fail]`
            Meta::Path(_) => {
                // Means "no parentheses", so no message
                Ok(ShouldFailAttr { message: None })
            }

            // e.g. `#[should_fail(...)]`
            Meta::List(meta_list) => parse_list(meta_list),

            // e.g. `#[should_fail = something]` is unusual; we can error
            Meta::NameValue(nv) => {
                Err(ShouldFailAttrError::MetaParseError(syn::Error::new_spanned(
                    nv,
                    "invalid format for #[should_fail]; expected `#[should_fail]` \
                     or `#[should_fail(message = \"...\")]`",
                )))
            }
        }
    }
}

/// Parse the parentheses: #[should_fail(...)]
fn parse_list(meta_list: MetaList) -> Result<ShouldFailAttr, ShouldFailAttrError> {
    let mut message: Option<String> = None;

    // parse_nested_meta will call the closure on each nested token like `message="..."`
    meta_list
        .parse_nested_meta(|nested| {
            if nested.path.is_ident("message") {
                // user wrote: `message = "..."`
                let lit: LitStr = nested.value()?.parse()?;
                message = Some(lit.value());
            } else {
                // unknown key
                return Err(syn::Error::new(
                    nested.path.span(),
                    "unknown key; expected `message = \"...\"`",
                ));
            }
            Ok(())
        })
        .map_err(map_parse_error)?;

    Ok(ShouldFailAttr { message })
}

/// Convert a `syn::Error` to your existing `ShouldFailAttrError`
pub fn map_parse_error(err: syn::Error) -> ShouldFailAttrError {
    trace!("map_parse_error: got syn error: {}", err);
    let msg = err.to_string();

    // Handle case where a non-string literal (like integer) was given for `message`
    if msg.contains("expected string literal for `message` value")
        || msg.contains("expected string literal")
        || msg.contains("invalid value: integer")
    {
        debug!("map_parse_error: treating as InvalidExpectedValueFormat");
        ShouldFailAttrError::InvalidExpectedValueFormat
    }
    // Handle other syntax issues such as missing '=' or unexpected EOI
    else if msg.contains("expected `=`") 
        || msg.contains("expected string literal") 
        || msg.contains("unexpected end of input") 
    {
        debug!("map_parse_error: treating as ExpectedValueMissing");
        ShouldFailAttrError::ExpectedValueMissing
    }
    // Fallback: bubble up the original parse error
    else {
        warn!("map_parse_error: returning MetaParseError");
        ShouldFailAttrError::MetaParseError(err)
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
