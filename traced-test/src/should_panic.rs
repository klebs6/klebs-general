crate::ix!();

#[derive(Clone,Debug)]
pub struct ShouldPanicAttr {
    expected: String,
}

//--------------------------------------------[try-from-attribute]
impl TryFrom<syn::Attribute> for ShouldPanicAttr {
    type Error = ShouldPanicAttrError;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        if !attr.path().is_ident("should_panic") {
            return Err(ShouldPanicAttrError::NotShouldPanicAttr);
        }

        let mut parsed_attr = None;

        // Parse the nested meta attributes
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("expected") {
                // Attempt to extract the value
                let value = meta.value();
                // Check if the value is missing
                if value.is_err() {
                    return Err(syn::Error::new(meta.input.span(), "Invalid expected value format"));
                }

                // Parse the value into a literal
                let lit = value?.parse::<syn::Lit>()?;
                if let syn::Lit::Str(lit_str) = lit {
                    parsed_attr = Some(ShouldPanicAttr {
                        expected: lit_str.value(),
                    });
                    return Ok(());
                } else {
                    // If the value is not a string literal, raise InvalidExpectedValueFormat
                    return Err(syn::Error::new(meta.input.span(), "Invalid expected value format"));
                }
            }

            // If the `expected` key was not found, raise ExpectedValueMissing
            Err(syn::Error::new(meta.input.span(), "expected key not found"))
        })
        // Convert syn::Error to ShouldPanicAttrError using map_err
        .map_err(|err| {
            if err.to_string().contains("Invalid expected value format") {
                ShouldPanicAttrError::InvalidExpectedValueFormat
            } else {
                ShouldPanicAttrError::ExpectedValueMissing
            }
        })?;

        parsed_attr.ok_or(ShouldPanicAttrError::ExpectedValueMissing)
    }
}

//--------------------------------------------[extract-from-attributes]
pub trait CheckForAndRetrieveTheUniqueShouldPanicAttr {

    /// Returns `Some(ShouldPanicAttr)` if the attribute is found.
    /// Returns `Err` if there's a duplicate or any other error while parsing.
    fn maybe_get_should_panic_attr(&self) -> Result<Option<ShouldPanicAttr>, ShouldPanicAttrError>;
}

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

//--------------------------------------------[has-panic-message]
pub trait HasPanicMessage {

    fn panic_message(&self) -> Cow<'_,str>;
}

impl HasPanicMessage for ShouldPanicAttr {

    fn panic_message(&self) -> Cow<'_,str> {
        Cow::Borrowed(&self.expected)
    }
}


//--------------------------------------------[tests]
#[cfg(test)]
mod should_panic_attr_tests {
    use super::*;
    use syn::{Attribute, parse_quote};

    #[test]
    fn test_single_should_panic_attr_with_expected() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[should_panic(expected = "some error")]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_ok());
        let attr = result.unwrap();
        assert!(attr.is_some());
        assert_eq!(attr.unwrap().expected, "some error");
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
    fn test_should_panic_attr_missing_expected_key() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[should_panic]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_err());
        if let Err(ShouldPanicAttrError::ExpectedValueMissing) = result {
        } else {
            panic!("Expected ExpectedValueMissing error");
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
        assert_eq!(attr.unwrap().expected, "valid error");
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
        assert_eq!(attr.unwrap().expected, "");
    }

    #[test]
    fn test_should_panic_attr_with_missing_expected_value() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[should_panic(expected)]),
        ];

        let result = attrs.as_slice().maybe_get_should_panic_attr();
        assert!(result.is_err());
        if let Err(ShouldPanicAttrError::InvalidExpectedValueFormat) = result {
        } else {
            panic!("Expected InvalidExpectedValueFormat error");
        }
    }
}
