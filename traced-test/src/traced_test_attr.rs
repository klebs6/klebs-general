crate::ix!();

#[derive(Debug)]
pub struct TracedTestAttr {
    trace: Option<bool>,
}

impl TracedTestAttr {

    pub fn specified(&self) -> bool {
        self.trace.is_some()
    }
}

impl ShouldTrace for TracedTestAttr {
    fn should_trace_on_success(&self) -> bool {
        match self.trace {
            Some(true) => true,    // Always trace on success
            Some(false) => false,  // Never trace on success
            None => false,         // Default: do not trace on success
        }
    }

    fn should_trace_on_failure(&self) -> bool {
        match self.trace {
            Some(true) => true,    // Always trace on failure
            Some(false) => false,  // Never trace on failure
            None => true,          // Default: trace on failure
        }
    }
}

impl SynParse for TracedTestAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut trace = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;

            if ident == "trace" {

                input.parse::<Token![=]>()?;
                let value: Lit = input.parse()?;

                match value {
                    Lit::Bool(lit_bool) => {
                        trace = Some(lit_bool.value);
                    }
                    _ => {
                        return Err(syn::Error::new(value.span(), "Expected boolean value for 'trace'"));
                    }
                }
            } else {
                return Err(syn::Error::new(ident.span(), "Unknown attribute key, expected 'trace'"));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(TracedTestAttr { trace })
    }
}

#[cfg(test)]
mod traced_test_attr_tests {
    use super::*;
    use syn::{Attribute, parse::Parser};
    use proc_macro2::TokenStream;

    fn parse_traced_test_attr(input: &str) -> syn::Result<TracedTestAttr> {
        let tokens: TokenStream = syn::parse_str(input)?;
        TracedTestAttr::parse.parse2(tokens)
    }

    #[test]
    fn test_parse_traced_test_attr_with_trace_true() {
        let input = r#"trace = true"#;
        let attr = parse_traced_test_attr(input).unwrap();
        assert_eq!(attr.trace, Some(true));
    }

    #[test]
    fn test_parse_traced_test_attr_with_trace_false() {
        let input = r#"trace = false"#;
        let attr = parse_traced_test_attr(input).unwrap();
        assert_eq!(attr.trace, Some(false));
    }

    #[test]
    fn test_parse_traced_test_attr_without_trace() {
        let input = r#""#;
        let attr = parse_traced_test_attr(input).unwrap();
        assert_eq!(attr.trace, None);
    }

    #[test]
    fn test_parse_traced_test_attr_with_unknown_key() {
        let input = r#"unknown = true"#;
        let result = parse_traced_test_attr(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_traced_test_attr_with_invalid_trace_value() {
        let input = r#"trace = "not_a_bool""#;
        let result = parse_traced_test_attr(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_traced_test_attr_with_multiple_args() {
        let input = r#"trace = true, unknown = false"#;
        let result = parse_traced_test_attr(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_traced_test_attr_with_extra_comma() {
        let input = r#"trace = true,"#;
        let attr = parse_traced_test_attr(input).unwrap();
        assert_eq!(attr.trace, Some(true));
    }

    #[test]
    fn test_parse_traced_test_attr_with_missing_equal() {
        let input = r#"trace true"#;
        let result = parse_traced_test_attr(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_traced_test_attr_with_empty_input() {
        let input = r#""#;
        let attr = parse_traced_test_attr(input).unwrap();
        assert_eq!(attr.trace, None);
    }
}
