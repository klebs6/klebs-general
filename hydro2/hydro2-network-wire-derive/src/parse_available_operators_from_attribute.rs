// ---------------- [ File: hydro2-network-wire-derive/src/parse_available_operators_from_attribute.rs ]
//! parse_available_operators_from_attribute.rs
//! This module extracts `#[available_operators(op="Foo", op="Bar<Z>")]` from a struct.
crate::ix!();

/// The top-level function for reading `#[available_operators(...)]`.
/// Returns `(Vec<OperatorSpecItem>, Generics)` or an error TokenStream.
pub fn parse_available_operators_attribute(
    input_ast: &DeriveInput
) -> Result<(Vec<OperatorSpecItem>, Generics), TokenStream>
{
    info!(
        "[parse_available_operators_attribute] Entered, struct name: {}",
        input_ast.ident
    );

    // 1) find `#[available_operators(...)]` attribute
    let attr = match input_ast.attrs.iter().find(|a| a.path().is_ident("available_operators")) {
        Some(a) => {
            info!(
                "[parse_available_operators_attribute] Found `available_operators` attribute for {}",
                input_ast.ident
            );
            a
        },
        None => {
            let msg = "Missing `#[available_operators(...)]` on struct with `#[derive(NetworkWire)]`.";
            info!(
                "[parse_available_operators_attribute] ERROR: {} (struct: {})",
                msg,
                input_ast.ident
            );
            let err_ts = SynError::new(input_ast.ident.span(), msg).to_compile_error();
            return Err(err_ts);
        }
    };

    // 2) parse the inside => e.g. `op="Foo", op="Bar<Baz>"`
    let op_items = match parse_operator_items(attr) {
        Ok(ops) => {
            info!(
                "[parse_available_operators_attribute] Successfully parsed operator items: {:?}",
                ops.iter().map(|oi| oi.path().to_token_stream().to_string()).collect::<Vec<_>>()
            );
            ops
        },
        Err(e) => {
            info!(
                "[parse_available_operators_attribute] ERROR: parse_operator_items failed: {:?}",
                e
            );
            return Err(e.to_compile_error());
        }
    };

    // The wire struct's own generics
    let gens = input_ast.generics.clone();
    info!(
        "[parse_available_operators_attribute] Generics for struct {}: {:#?}",
        input_ast.ident, gens
    );

    info!("[parse_available_operators_attribute] Exiting normally.");
    Ok((op_items, gens))
}

/// parse_operator_items just calls `OperatorItemsParser`
pub fn parse_operator_items(attr: &Attribute) -> syn::Result<Vec<OperatorSpecItem>> {
    info!("[parse_operator_items] Entered.");

    let parsed = attr.parse_args::<OperatorItemsParser>();
    match &parsed {
        Ok(_) => info!("[parse_operator_items] OperatorItemsParser returned Ok."),
        Err(err) => info!("[parse_operator_items] OperatorItemsParser returned Err: {:?}", err),
    };

    let parsed = parsed?;
    let items = parsed.items().to_vec();
    info!(
        "[parse_operator_items] Successfully extracted items: {:?}",
        items.iter().map(|i| i.path().to_token_stream().to_string()).collect::<Vec<_>>()
    );

    info!("[parse_operator_items] Exiting with Ok(..).");
    Ok(items)
}

#[cfg(test)]
mod parse_ops_test {
    use super::*;
    use quote::quote;
    use syn::parse2;
    use syn::parse_quote;

    #[test]
    fn test_operator_spec_item_basics() {
        info!("[test_operator_spec_item_basics] Entered.");
        let path: Path = parse_quote!(MyOp1 < T >);
        let item = OperatorSpecItem::new(path);
        info!(
            "[test_operator_spec_item_basics] Item path first segment: {}",
            item.path().segments[0].ident
        );
        assert_eq!(item.path().segments[0].ident.to_string(), "MyOp1");
    }

    #[test]
    fn test_operator_items_parser_single() {
        info!("[test_operator_items_parser_single] Entered.");
        // e.g. `op="MyOpFoo<T>"`
        let tokens = quote! { op="MyOpFoo<T>" };
        let parsed = parse2::<OperatorItemsParser>(tokens);
        info!(
            "[test_operator_items_parser_single] parse2 result: {:?}",
            parsed
        );
        assert!(parsed.is_ok(), "Expected single operator parse to succeed");
        let parser_obj = parsed.unwrap();
        assert_eq!(parser_obj.items().len(), 1);
        assert_eq!(parser_obj.items()[0].path().segments[0].ident.to_string(), "MyOpFoo");
    }

    #[test]
    fn test_operator_items_parser_multiple() {
        info!("[test_operator_items_parser_multiple] Entered.");
        // e.g. `op="Foo", op="Bar<Baz>"`
        let tokens = quote! { op="Foo", op="Bar<Baz>" };
        let parsed = parse2::<OperatorItemsParser>(tokens);
        info!(
            "[test_operator_items_parser_multiple] parse2 result: {:?}",
            parsed
        );
        assert!(parsed.is_ok());
        let parser_obj = parsed.unwrap();
        let items_count = parser_obj.items().len();
        info!(
            "[test_operator_items_parser_multiple] Got {} items after parse.",
            items_count
        );
        assert_eq!(items_count, 2);
        assert_eq!(parser_obj.items()[0].path().segments[0].ident.to_string(), "Foo");
        assert_eq!(parser_obj.items()[1].path().segments[0].ident.to_string(), "Bar");
    }

    #[test]
    fn test_operator_items_parser_fail_nonstring() {
        info!("[test_operator_items_parser_fail_nonstring] Entered.");
        // e.g. `op=123` => must fail
        let tokens = quote! { op=123 };
        let parsed = parse2::<OperatorItemsParser>(tokens);
        info!(
            "[test_operator_items_parser_fail_nonstring] parse2 result: {:?}",
            parsed
        );
        assert!(parsed.is_err());
        let msg = format!("{:?}", parsed.err().unwrap());
        assert!(msg.contains("expected string literal"), "Got: {}", msg);
    }

    #[test]
    fn test_operator_items_parser_fail_not_op() {
        info!("[test_operator_items_parser_fail_not_op] Entered.");
        // e.g. `foo="Bar"`
        let tokens = quote! { foo="Bar" };
        let parsed = parse2::<OperatorItemsParser>(tokens);
        info!(
            "[test_operator_items_parser_fail_not_op] parse2 result: {:?}",
            parsed
        );
        assert!(parsed.is_err());
        let msg = format!("{:?}", parsed.err().unwrap());
        assert!(msg.contains("Expected `op`"), "Got: {}", msg);
    }

    #[test]
    fn test_parse_available_operators_attribute_missing_attr() {
        info!("[test_parse_available_operators_attribute_missing_attr] Entered.");
        let di: DeriveInput = parse_quote! {
            #[derive(NetworkWire)]
            // no `#[available_operators]`
            pub struct MyFakeWire;
        };
        let res = parse_available_operators_attribute(&di);
        info!(
            "[test_parse_available_operators_attribute_missing_attr] parse result: {:?}",
            res
        );
        assert!(res.is_err());
        let err_ts = res.err().unwrap();
        let err_str = err_ts.to_string();
        assert!(
            err_str.contains("Missing `#[available_operators(...)]`"),
            "Got: {}",
            err_str
        );
    }

    #[test]
    fn test_parse_available_operators_attribute_ok() {
        info!("[test_parse_available_operators_attribute_ok] Entered.");
        // We'll define a pseudo-struct with the attribute
        let di: DeriveInput = parse_quote! {
            #[derive(NetworkWire)]
            #[available_operators(op="Foo", op="Bar<Baz>")]
            pub struct MyWire<Baz> {
                _p: std::marker::PhantomData<Baz>,
            }
        };
        let res = parse_available_operators_attribute(&di);
        info!(
            "[test_parse_available_operators_attribute_ok] parse result: {:?}",
            res
        );
        assert!(res.is_ok());
        let (ops, gens) = res.unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].path().segments[0].ident.to_string(), "Foo");
        assert_eq!(ops[1].path().segments[0].ident.to_string(), "Bar");
        assert_eq!(gens.params.len(), 1); // the generics <Baz>
    }
}
