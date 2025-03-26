// ---------------- [ File: src/traced_test_attr.rs ]
crate::ix!();

#[derive(Getters, Setters, Debug)]
#[getset(get = "pub", set = "pub")]
pub struct TracedTestAttr {
    trace:          Option<bool>,
    backtrace:      Option<bool>,
    show_timestamp: Option<bool>,
    show_loglevel:  Option<bool>,

    // NEW FIELDS: “should_fail” plus the optional fail-message
    should_fail:    bool,
    fail_message:   Option<String>,
}

impl ShouldTrace for TracedTestAttr {
    fn should_trace_on_success(&self) -> bool {
        match self.trace {
            Some(true)  => true,
            Some(false) => false,
            None        => false,
        }
    }

    fn should_trace_on_failure(&self) -> bool {
        match self.trace {
            Some(true)  => true,
            Some(false) => false,
            None        => true,
        }
    }
}

impl SynParse for TracedTestAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Ident, Token, Lit, LitStr};

        let mut trace          = None;
        let mut backtrace      = None;
        let mut show_timestamp = None;
        let mut show_loglevel  = None;

        let mut should_fail    = false;
        let mut fail_message   = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;

            if ident == "trace" {
                input.parse::<Token![=]>()?;
                let value: Lit = input.parse()?;
                match value {
                    Lit::Bool(b) => trace = Some(b.value),
                    _ => return Err(syn::Error::new(value.span(), "Expected boolean for `trace`")),
                }
            }
            else if ident == "backtrace" {
                input.parse::<Token![=]>()?;
                let value: Lit = input.parse()?;
                match value {
                    Lit::Bool(b) => backtrace = Some(b.value),
                    _ => return Err(syn::Error::new(value.span(), "Expected boolean for `backtrace`")),
                }
            }
            else if ident == "show_timestamp" {
                input.parse::<Token![=]>()?;
                let value: Lit = input.parse()?;
                match value {
                    Lit::Bool(b) => show_timestamp = Some(b.value),
                    _ => return Err(syn::Error::new(value.span(), "Expected boolean for `show_timestamp`")),
                }
            }
            else if ident == "show_loglevel" {
                input.parse::<Token![=]>()?;
                let value: Lit = input.parse()?;
                match value {
                    Lit::Bool(b) => show_loglevel = Some(b.value),
                    _ => return Err(syn::Error::new(value.span(), "Expected boolean for `show_loglevel`")),
                }
            }
            else if ident == "should_fail" {
                // If user wrote: #[traced_test(should_fail, ...)]
                // or:           #[traced_test(should_fail(message="xyz"))]
                should_fail = true;
                if input.peek(syn::token::Paren) {
                    // Parse parentheses: ( message = "..." )
                    let content;
                    syn::parenthesized!(content in input);

                    // We expect something like: message = "some text"
                    let message_ident: Ident = content.parse()?;
                    if message_ident != "message" {
                        return Err(syn::Error::new(
                            message_ident.span(),
                            "expected `message = \"...\"` inside `should_fail(...)`",
                        ));
                    }
                    content.parse::<Token![=]>()?;
                    let message_lit: Lit = content.parse()?;
                    match message_lit {
                        Lit::Str(s) => fail_message = Some(s.value()),
                        _ => return Err(syn::Error::new(
                            message_lit.span(),
                            "expected string literal for `message`",
                        )),
                    }
                }
            }
            else {
                return Err(syn::Error::new(
                    ident.span(),
                    "Unknown attribute key; expected one of `trace`, `backtrace`, `show_timestamp`, `show_loglevel`, or `should_fail`"
                ));
            }

            // consume a trailing comma if present
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                // not a comma, so break from the while
                break;
            }
        }

        Ok(Self {
            trace,
            backtrace,
            show_timestamp,
            show_loglevel,
            should_fail,
            fail_message,
        })
    }
}

impl TracedTestAttr {
    pub fn specified(&self) -> bool {
        self.trace.is_some()
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
