// ---------------- [ File: src/should_panic_attr.rs ]
crate::ix!();

#[derive(Clone, Debug)]
pub struct ShouldPanicAttr {
    expected: Option<String>,
}

impl MaybeHasPanicMessage for ShouldPanicAttr {

    fn panic_message(&self) -> Option<Cow<'_, str>> {
        self.expected.as_deref().map(Cow::Borrowed)
    }
}

impl TryFrom<syn::Attribute> for ShouldPanicAttr {

    type Error = ShouldPanicAttrError;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        if !attr.path().is_ident("should_panic") {
            return Err(ShouldPanicAttrError::NotShouldPanicAttr);
        }

        // Check if the attribute has no arguments (no parentheses)
        if let syn::Meta::Path(_) = attr.meta {
            // No arguments, so expected is None
            return Ok(ShouldPanicAttr { expected: None });
        }

        // Otherwise, parse the arguments
        let parser = |input: ParseStream| {
            let ident: Ident = input.parse()?;
            if ident != "expected" {
                return Err(syn::Error::new(ident.span(), "unknown key, expected `expected`"));
            }
            input.parse::<Token![=]>()?;
            let lit: syn::Lit = input.parse()?;
            if let syn::Lit::Str(lit_str) = lit {
                Ok(ShouldPanicAttr {
                    expected: Some(lit_str.value()),
                })
            } else {
                Err(syn::Error::new(lit.span(), "expected string literal for `expected` value"))
            }
        };

        attr.parse_args_with(parser).map_err(|err| {
            let msg = err.to_string();
            if msg.contains("expected string literal for `expected` value") {
                ShouldPanicAttrError::InvalidExpectedValueFormat
            } else if msg.contains("expected `=`") || msg.contains("expected string literal") || msg.contains("unexpected end of input") {
                ShouldPanicAttrError::ExpectedValueMissing
            } else {
                ShouldPanicAttrError::MetaParseError(err)
            }
        })
    }
}

//--------------------------------------------[extract-from-attributes]
impl CheckForAndRetrieveTheUniqueShouldPanicAttr for &[syn::Attribute] {

    fn maybe_get_should_panic_attr(&self) -> Result<Option<ShouldPanicAttr>, ShouldPanicAttrError> {
        let mut should_panic_attr = None;

        for attr in *self {
            if attr.path().is_ident("should_panic") {
                if should_panic_attr.is_some() {
                    // We already found a `should_panic` attribute, return an error for duplicates.
                    return Err(ShouldPanicAttrError::MultipleShouldPanicAttrs);
                }

                // Try converting the attribute into a `ShouldPanicAttr`
                let parsed_attr = ShouldPanicAttr::try_from(attr.clone())?;
                should_panic_attr = Some(parsed_attr);
            }
        }

        Ok(should_panic_attr)
    }
}

#[cfg(test)]
mod retrieve_should_panic_attr_tests {
    use crate::ShouldPanicAttrError;
    use super::*;
    use syn::{Attribute, parse_quote};

    #[test]
    fn test_should_panic_attr_missing_expected_key() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[should_panic]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_ok());
        let attr = result.unwrap();
        assert!(attr.is_some());
        assert!(attr.unwrap().expected.is_none());
    }

    #[test]
    fn test_should_panic_attr_with_missing_expected_value() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[should_panic(expected)]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_err());
        if let Err(ShouldPanicAttrError::ExpectedValueMissing) = result {
        } else {
            panic!("Expected ExpectedValueMissing error");
        }
    }

    #[test]
    fn test_single_should_panic_attr_with_expected() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[should_panic(expected = "some error")]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_ok());
        let attr = result.unwrap();
        assert!(attr.is_some());
        assert_eq!(attr.unwrap().expected.unwrap(), "some error");
    }

    #[test]
    fn test_should_panic_attr_with_invalid_literal_type() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[should_panic(expected = 123)]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_err());
        if let Err(ShouldPanicAttrError::InvalidExpectedValueFormat) = result {
        } else {
            panic!("Expected InvalidExpectedValueFormat error");
        }
    }

    #[test]
    fn test_should_panic_attr_multiple_attributes() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[should_panic(expected = "error 1")]),
            parse_quote!(#[should_panic(expected = "error 2")]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ShouldPanicAttrError::MultipleShouldPanicAttrs);
    }

    #[test]
    fn test_should_panic_attr_with_valid_and_invalid() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[should_panic(expected = "valid error")]),
            parse_quote!(#[should_panic(expected = 123)]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_err());
        if let Err(ShouldPanicAttrError::MultipleShouldPanicAttrs) = result {
        } else {
            panic!("Expected MultipleShouldPanicAttrs error");
        }
    }

    #[test]
    fn test_should_panic_attr_with_non_should_panic_attributes() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[other_attr(some_value = "something")]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_should_panic_attr_with_mixed_attributes() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[other_attr(some_value = "something")]),
            parse_quote!(#[should_panic(expected = "valid error")]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_ok());
        let attr = result.unwrap();
        assert!(attr.is_some());
        assert_eq!(attr.unwrap().expected.unwrap(), "valid error");
    }

    #[test]
    fn test_should_panic_attr_with_empty_expected() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[should_panic(expected = "")]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_ok());
        let attr = result.unwrap();
        assert!(attr.is_some());
        assert_eq!(attr.unwrap().expected.unwrap(), "");
    }
}
