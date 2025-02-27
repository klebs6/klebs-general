// ---------------- [ File: src/operator_items_parser.rs ]
crate::ix!();

/// A small struct that captures all `op="..."`
#[derive(Getters, Debug)]
#[getset(get="pub")]
pub struct OperatorItemsParser {
    items: Vec<OperatorSpecItem>,
}

impl syn::parse::Parse for OperatorItemsParser {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        info!("OperatorItemsParser::parse: START");
        info!("  remaining input = '{}'", input.to_string());

        let mut items = Vec::new();

        // We'll parse *zero or more* `op="..."` pairs,
        // requiring commas between multiple pairs.
        // After each pair, if there's no trailing comma, we break.
        while !input.is_empty() {
            // 1) Ensure the next token is an Ident == "op"
            if !input.peek(syn::Ident) {
                // The next token is not an Ident => leftover
                let leftover = input.to_string();
                let msg = format!("unexpected token(s) leftover: {}", leftover);
                info!("  returning error => {}", msg);
                return Err(syn::Error::new(input.span(), msg));
            }

            let ident_op: syn::Ident = input.parse()?;
            if ident_op != "op" {
                let msg = format!("Expected `op`, found `{}`", ident_op);
                info!("  returning error => {}", msg);
                return Err(syn::Error::new(ident_op.span(), msg));
            }

            // 2) Parse `=`
            let eq_tok: syn::Token![=] = input.parse()?; 
            info!("  parsed '=' token: {:?}", eq_tok.span());

            // 3) Parse literal string => e.g. "FooOp"
            let lit_str: syn::LitStr = input.parse()?;
            let path_str = lit_str.value();
            info!("  parsed lit_str => {:?}", path_str);

            // 4) Convert that string into a `syn::Path`
            let path = match syn::parse_str::<syn::Path>(&path_str) {
                Ok(p) => {
                    info!("  parsed Path => {}", quote::ToTokens::to_token_stream(&p));
                    p
                }
                Err(_err) => {
                    let msg = format!("Could not parse `{}` as a Path.", path_str);
                    info!("  returning error => {}", msg);
                    return Err(syn::Error::new(lit_str.span(), msg));
                }
            };
            items.push(OperatorSpecItem::new(path));

            // 5) If there's a trailing comma, parse it and continue parsing more items.
            //    Otherwise, break the loop => we either hit the end or leftover tokens.
            if input.peek(syn::Token![,]) {
                let _comma: syn::Token![,] = input.parse()?;
                info!("  consumed trailing comma, continuing parse");
            } else {
                // No trailing comma => we're done reading `op="..."` pairs
                break;
            }
        }

        // 6) If any tokens remain, they're leftover => error
        if !input.is_empty() {
            let leftover = input.to_string();
            let msg = format!("unexpected token(s) leftover: {}", leftover);
            info!("  returning error => {}", msg);
            return Err(syn::Error::new(input.span(), msg));
        }

        info!(
            "OperatorItemsParser::parse: FINISH, parsed {} items",
            items.len()
        );
        Ok(OperatorItemsParser { items })
    }
}

#[cfg(test)]
mod test_operator_items_parser_code {
    use super::*;
    use syn::parse::Parser;
    use syn::{parse_quote, DeriveInput};

    /// Helper function to test parse_available_operators_attribute usage.
    /// We assume it's something like:
    ///
    /// ```ignore
    /// pub fn parse_available_operators_attribute(
    ///     di: &DeriveInput
    /// ) -> Result<(Vec<OperatorSpecItem>, Generics), TokenStream>
    /// ```
    /// 
    /// We'll stub out or pseudocode the logic here.
    pub fn parse_available_operators_attribute(
        di: &DeriveInput
    ) -> Result<(Vec<OperatorSpecItem>, syn::Generics), proc_macro2::TokenStream> {
        info!("parse_available_operators_attribute: START");
        // check for attribute presence
        let attr = match di.attrs.iter().find(|a| a.path().is_ident("available_operators")) {
            Some(a) => a,
            None => {
                let err = quote::quote_spanned! { di.ident.span() =>
                    compile_error!("Missing `#[available_operators(...)]` attribute!");
                };
                info!("  returning error => Missing `#[available_operators(...)]` attribute!");
                return Err(err);
            }
        };

        // parse OperatorItemsParser
        let parser = OperatorItemsParser::parse;
        let parsed_items_parser = match attr.parse_args_with(OperatorItemsParser::parse) {
            Ok(p) => p,
            Err(e) => {
                let e_str = e.to_string();
                let err = quote::quote_spanned! { attr.span() =>
                    compile_error!(#e_str);
                };
                return Err(err);
            }
        };

        // For demonstration, return the items plus the generics from the DeriveInput
        info!("  parse_available_operators_attribute: SUCCESS => parsed {} items",
                  parsed_items_parser.items.len());
        Ok((parsed_items_parser.items, di.generics.clone()))
    }

    #[test]
    fn test_parse_ok() {
        info!("test_parse_ok: START");
        let di: DeriveInput = parse_quote! {
            #[derive(NetworkWire)]
            #[available_operators(op="Foo", op="Bar<Baz>")]
            pub struct MyWire<Baz> {
                _p: std::marker::PhantomData<Baz>,
            }
        };
        let res = parse_available_operators_attribute(&di);
        assert!(res.is_ok());
        let (ops, gens) = res.unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].path().segments[0].ident.to_string(), "Foo");
        assert_eq!(ops[1].path().segments[0].ident.to_string(), "Bar");
        assert_eq!(gens.params.len(), 1); // the generics <Baz>
    }

    #[test]
    fn test_parse_fail_no_attr() {
        info!("test_parse_fail_no_attr: START");
        let di: DeriveInput = parse_quote! {
            #[derive(NetworkWire)]
            pub struct MyWire {}
        };
        let res = parse_available_operators_attribute(&di);
        assert!(res.is_err());
        let err = format!("{}", res.err().unwrap());
        assert!(err.contains("Missing `#[available_operators(...)]`"), "Got: {}", err);
    }

    /// Ensure empty input parses into an empty items list.
    #[test]
    fn parse_empty_input() {
        info!("parse_empty_input: START");
        let tokens = quote::quote! {};
        let parse_res = OperatorItemsParser::parse.parse2(tokens);
        assert!(parse_res.is_ok());
        if let Ok(parser) = parse_res {
            assert_eq!(parser.items().len(), 0);
        }
    }

    /// Single operator specified with `op="Foo"` should parse correctly.
    #[test]
    fn parse_single_op_ok() {
        info!("parse_single_op_ok: START");
        let tokens = quote::quote! { op="Foo" };
        let parse_res = OperatorItemsParser::parse.parse2(tokens);
        assert!(parse_res.is_ok());
        if let Ok(parser) = parse_res {
            let items = parser.items();
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].path().segments.len(), 1);
            assert_eq!(items[0].path().segments[0].ident.to_string(), "Foo");
        }
    }

    /// Multiple operators, each prefixed by `op=...`, separated by commas.
    #[test]
    fn parse_multiple_ops_ok() {
        info!("parse_multiple_ops_ok: START");
        let tokens = quote::quote! { op="Foo", op="Bar<Baz>", op="Qux" };
        let parse_res = OperatorItemsParser::parse.parse2(tokens);
        assert!(parse_res.is_ok());
        if let Ok(parser) = parse_res {
            let items = parser.items();
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].path().segments[0].ident.to_string(), "Foo");
            assert_eq!(items[1].path().segments[0].ident.to_string(), "Bar");
            assert_eq!(items[2].path().segments[0].ident.to_string(), "Qux");
        }
    }

    /// Parsing should fail if the `op` token is missing or replaced.
    #[test]
    fn parse_fails_wrong_keyword() {
        info!("parse_fails_wrong_keyword: START");
        let tokens = quote::quote! { not_op="Foo" };
        let parse_res = OperatorItemsParser::parse.parse2(tokens);
        assert!(parse_res.is_err());
        if let Err(err) = parse_res {
            let msg = err.to_string();
            assert!(
                msg.contains("Expected `op`"),
                "unexpected error message: {}",
                msg
            );
        }
    }

    /// Parsing should fail if the token after `op` isn't an `=`.
    #[test]
    fn parse_fails_missing_eq() {
        info!("parse_fails_missing_eq: START");
        let tokens = quote::quote! { op "Foo" };
        let parse_res = OperatorItemsParser::parse.parse2(tokens);
        assert!(parse_res.is_err());
        if let Err(err) = parse_res {
            let msg = err.to_string();
            assert!(
                msg.contains("expected `=`"),
                "unexpected error message: {}",
                msg
            );
        }
    }

    /// Parsing should fail if the quoted value can't be parsed as a `Path`.
    #[test]
    fn parse_fails_invalid_path() {
        info!("parse_fails_invalid_path: START");
        let tokens = quote::quote! { op="::" };
        let parse_res = OperatorItemsParser::parse.parse2(tokens);
        assert!(parse_res.is_err());
        if let Err(err) = parse_res {
            let msg = err.to_string();
            assert!(
                msg.contains("Could not parse `::` as a Path."),
                "unexpected error message: {}",
                msg
            );
        }
    }

    /// Demonstrate that trailing commas or spacing are handled gracefully.
    #[test]
    fn parse_ok_trailing_comma() {
        info!("parse_ok_trailing_comma: START");
        let tokens = quote::quote! { op="Foo", };
        let parse_res = OperatorItemsParser::parse.parse2(tokens);
        assert!(parse_res.is_ok());
        if let Ok(parser) = parse_res {
            assert_eq!(parser.items().len(), 1);
            assert_eq!(parser.items()[0].path().segments[0].ident.to_string(), "Foo");
        }
    }

    /// Test a case with multiple ops but some tricky spacing or newline usage.
    #[test]
    fn parse_ok_spacing_variations() {
        info!("parse_ok_spacing_variations: START");
        let tokens = quote::quote! {
            op = "AlphaOp"
            ,    op = "BetaOp"   ,
            op="GammaOp"
        };
        let parse_res = OperatorItemsParser::parse.parse2(tokens);
        assert!(parse_res.is_ok());
        if let Ok(parser) = parse_res {
            let items = parser.items();
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].path().segments[0].ident.to_string(), "AlphaOp");
            assert_eq!(items[1].path().segments[0].ident.to_string(), "BetaOp");
            assert_eq!(items[2].path().segments[0].ident.to_string(), "GammaOp");
        }
    }

    /// Confirm that empty string `op=""` fails gracefully when converting to a Path.
    #[test]
    fn parse_fails_empty_string() {
        info!("parse_fails_empty_string: START");
        let tokens = quote::quote! { op="" };
        let parse_res = OperatorItemsParser::parse.parse2(tokens);
        assert!(parse_res.is_err());
        if let Err(err) = parse_res {
            let msg = err.to_string();
            assert!(
                msg.contains("Could not parse `` as a Path."),
                "unexpected error message: {}",
                msg
            );
        }
    }

    /// Parsing with extraneous tokens after valid pairs should fail (leftover tokens).
    #[test]
    fn parse_fails_extra_tokens() {
        info!("parse_fails_extra_tokens: START");
        let tokens = quote::quote! { op="Foo" something_else };
        let parse_res = OperatorItemsParser::parse.parse2(tokens);
        assert!(parse_res.is_err());
        if let Err(err) = parse_res {
            let msg = err.to_string();
            assert!(
                msg.contains("unexpected token"),
                "Got unexpected error message: {}",
                msg
            );
        }
    }
}
