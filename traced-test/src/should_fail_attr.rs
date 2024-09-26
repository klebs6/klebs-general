crate::ix!();

#[derive(Clone, Debug)]
pub struct ShouldFailAttr {
    message: Option<String>,
    trace: Option<bool>,
}

impl ShouldFailAttr {

    pub fn should_trace(&self) -> bool {
        self.trace.unwrap_or(false)
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
            // No arguments, so message and trace are None
            return Ok(ShouldFailAttr { message: None, trace: None });
        }

        // Parse the attribute arguments
        let parser = |input: ParseStream| {
            let mut parsed_message = None;
            let mut parsed_trace = None;

            while !input.is_empty() {
                let ident: Ident = input.parse()?;
                let lookahead = input.lookahead1();
                if ident == "message" {
                    input.parse::<Token![=]>()?;
                    let lit: syn::Lit = input.parse()?;
                    if let syn::Lit::Str(lit_str) = lit {
                        parsed_message = Some(lit_str.value());
                    } else {
                        return Err(syn::Error::new(lit.span(), "expected string literal for `message` value"));
                    }
                } else if ident == "trace" {
                    input.parse::<Token![=]>()?;
                    let lit: syn::Lit = input.parse()?;
                    if let syn::Lit::Bool(lit_bool) = lit {
                        parsed_trace = Some(lit_bool.value);
                    } else {
                        return Err(syn::Error::new(lit.span(), "expected boolean literal for `trace` value"));
                    }
                } else {
                    return Err(syn::Error::new(ident.span(), "unknown key, expected `message` or `trace`"));
                }

                // If there's a comma, consume it
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }

            Ok(ShouldFailAttr {
                message: parsed_message,
                trace: parsed_trace,
            })
        };

        attr.parse_args_with(parser).map_err(|err| {
            let msg = err.to_string();
            if msg.contains("expected string literal for `message` value") {
                ShouldFailAttrError::InvalidExpectedValueFormat
            } else if msg.contains("expected boolean literal for `trace` value") {
                ShouldFailAttrError::InvalidExpectedValueFormat
            } else if msg.contains("expected `=`") || msg.contains("unexpected end of input") {
                ShouldFailAttrError::ExpectedValueMissing
            } else {
                ShouldFailAttrError::MetaParseError(err)
            }
        })
    }
}

impl CheckForAndRetrieveTheUniqueShouldFailAttr for &[syn::Attribute] {
    fn maybe_get_should_fail_attr(&self) -> Result<Option<ShouldFailAttr>, TracedTestError> {
        let mut should_fail_attr = None;

        for attr in *self {
            if attr.path().is_ident("should_fail") {
                if should_fail_attr.is_some() {
                    return Err(TracedTestError::MultipleShouldFailAttrs);
                }

                let parsed_attr =
                    ShouldFailAttr::try_from(attr.clone()).map_err(TracedTestError::ShouldFailAttrError)?;
                should_fail_attr = Some(parsed_attr);
            }
        }

        Ok(should_fail_attr)
    }
}

#[cfg(test)]
mod should_fail_attr_tests {
    use super::*;
    use syn::Attribute;
    use syn::parse_quote;

    #[test]
    fn test_should_fail_attr_with_trace_and_message() {
        let attr: Attribute = parse_quote!(#[should_fail(trace = true, message = "Test failed")]);
        let should_fail_attr = ShouldFailAttr::try_from(attr).expect("Failed to parse should_fail attribute");
        assert_eq!(should_fail_attr.message.as_deref(), Some("Test failed"));
        assert_eq!(should_fail_attr.trace, Some(true));
    }

    #[test]
    fn test_should_fail_attr_with_trace_only() {
        let attr: Attribute = parse_quote!(#[should_fail(trace = true)]);
        let should_fail_attr = ShouldFailAttr::try_from(attr).expect("Failed to parse should_fail attribute");
        assert_eq!(should_fail_attr.message.as_deref(), None);
        assert_eq!(should_fail_attr.trace, Some(true));
    }

    #[test]
    fn test_should_fail_attr_with_message_only() {
        let attr: Attribute = parse_quote!(#[should_fail(message = "Test failed")]);
        let should_fail_attr = ShouldFailAttr::try_from(attr).expect("Failed to parse should_fail attribute");
        assert_eq!(should_fail_attr.message.as_deref(), Some("Test failed"));
        assert_eq!(should_fail_attr.trace, None);
    }

    #[test]
    fn test_should_fail_attr_with_no_args() {
        let attr: Attribute = parse_quote!(#[should_fail]);
        let should_fail_attr = ShouldFailAttr::try_from(attr).expect("Failed to parse should_fail attribute");
        assert_eq!(should_fail_attr.message.as_deref(), None);
        assert_eq!(should_fail_attr.trace, None);
    }
}
