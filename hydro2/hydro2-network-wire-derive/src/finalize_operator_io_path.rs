// ---------------- [ File: src/finalize_operator_io_path.rs ]
crate::ix!();

/// Renames the final segment of `op_path` => e.g. FooOp => FooOpIO
/// Returns (mutated_path, new_ident). If no last segment, returns error ident.
pub fn finalize_operator_io_path(mut op_path: syn::Path) -> (syn::Path, syn::Ident) {
    info!("finalize_operator_io_path: START");
    info!("  original op_path = {}", quote::ToTokens::to_token_stream(&op_path));

    if let Some(last_seg) = op_path.segments.last_mut() {
        let old_ident = &last_seg.ident;
        let new_io_ident = syn::Ident::new(
            &format!("{}IO", old_ident),
            old_ident.span(),
        );
        info!("  found last segment '{}'; renaming to '{}'", old_ident, new_io_ident);

        last_seg.ident = new_io_ident.clone();
        info!("  op_path now = {}", quote::ToTokens::to_token_stream(&op_path));

        info!("finalize_operator_io_path: FINISH (renamed segment)");
        (op_path, new_io_ident)
    } else {
        info!("  no last segment found, using error ident");
        let err_ident = syn::Ident::new("_____Error_NoLastSeg", op_path.span());
        info!("finalize_operator_io_path: FINISH (error ident = {})", err_ident);
        (op_path, err_ident)
    }
}

#[cfg(test)]
mod test_finalize_operator_io_path {
    use super::*;
    use syn::{parse_str, parse_quote, DeriveInput, Path, punctuated::Punctuated, GenericParam};
    use syn::token::Comma;
    use quote::ToTokens;

    /// Attempts to parse a `syn::Path` from a given string.
    /// Returns an error instead of panicking.
    fn try_parse_path(s: &str) -> Result<Path, syn::Error> {
        info!("try_parse_path: attempting to parse '{}'", s);
        match parse_str::<Path>(s) {
            Ok(p) => {
                info!("  parse succeeded => {}", p.to_token_stream());
                Ok(p)
            }
            Err(e) => {
                info!("  parse failed => {}", e);
                Err(e)
            }
        }
    }

    #[test]
    fn test_finalize_operator_io_path_valid() -> Result<(), syn::Error> {
        info!("test_finalize_operator_io_path_valid: START");
        let original = try_parse_path("FooOp")?;
        let (mutated_path, new_ident) = finalize_operator_io_path(original.clone());

        assert_eq!(new_ident.to_string(), "FooOpIO");
        let last_seg = mutated_path
            .segments
            .last()
            .ok_or_else(|| syn::Error::new(
                original.span(),
                "Expected at least one segment after finalization"
            ))?;
        assert_eq!(last_seg.ident.to_string(), "FooOpIO");
        Ok(())
    }

    #[test]
    fn test_finalize_operator_io_path_no_segments() -> Result<(), syn::Error> {
        info!("test_finalize_operator_io_path_no_segments: START");
        let empty_path = Path {
            leading_colon: None,
            segments: Punctuated::new(),
        };
        let (_, new_ident) = finalize_operator_io_path(empty_path.clone());
        assert_eq!(new_ident.to_string(), "_____Error_NoLastSeg");
        Ok(())
    }

    // The following tests for parse_available_operators_attribute, operator_items_parser,
    // build_network_io_enum, etc. are shown for completeness in the same file, but 
    // typically you'd split them into their own modules. We'll just add minor 
    // eprintln statements for demonstration.

    #[test]
    fn test_parse_available_operators_attribute_ok() -> Result<(), syn::Error> {
        info!("test_parse_available_operators_attribute_ok: START");
        let derive_input: DeriveInput = parse_quote! {
            #[available_operators(op="AddOp", op="SubOp<X>", op="MulOp<A,B>")]
            pub struct MyWire<T> { field: T }
        };
        let (ops, gens) = match parse_available_operators_attribute(&derive_input) {
            Ok((o, g)) => {
                info!("  parse_available_operators_attribute => Ok");
                (o, g)
            }
            Err(ts) => {
                info!("  parse_available_operators_attribute => Error: {}", ts.to_string());
                return Err(syn::Error::new(derive_input.ident.span(), ts.to_string()));
            }
        };
        // We expect 3 operator items
        assert_eq!(ops.len(), 3);
        assert!(gens.params.iter().any(|p| match p {
            GenericParam::Type(t) => t.ident == "T",
            _ => false,
        }), "Expected generic param T in struct MyWire<T>");
        Ok(())
    }

    #[test]
    fn test_parse_available_operators_attribute_missing() -> Result<(), syn::Error> {
        info!("test_parse_available_operators_attribute_missing: START");
        let derive_input: DeriveInput = parse_quote! {
            pub struct MyWire {}
        };
        let result = parse_available_operators_attribute(&derive_input);
        assert!(result.is_err(), "Expected parse failure for missing attribute.");
        Ok(())
    }

    #[test]
    fn test_parse_available_operators_attribute_malformed_path() -> Result<(), syn::Error> {
        info!("test_parse_available_operators_attribute_malformed_path: START");
        let derive_input: DeriveInput = parse_quote! {
            #[available_operators(op="Not<A<>")]
            pub struct MyWire {}
        };
        let result = parse_available_operators_attribute(&derive_input);
        assert!(result.is_err(), "Expected parse failure for malformed path in attribute.");
        Ok(())
    }

    #[test]
    fn test_operator_items_parser_multiple() -> Result<(), syn::Error> {
        info!("test_operator_items_parser_multiple: START");
        let attr_tokens = quote! { op="Foo", op="Bar<Baz>", op="Spam<Z,Q>" };
        let parser = parse_quote! { #attr_tokens };
        let items_parser: OperatorItemsParser = parser;
        let items = items_parser.items();
        assert_eq!(items.len(), 3);
        Ok(())
    }

    #[test]
    fn test_operator_items_parser_invalid_key() -> Result<(), syn::Error> {
        info!("test_operator_items_parser_invalid_key: START");
        let attr_tokens = quote! { something_else="Foo" };
        let attempt = syn::parse2::<OperatorItemsParser>(attr_tokens);
        assert!(attempt.is_err());
        Ok(())
    }

    #[test]
    fn test_build_network_io_enum_normal() -> Result<(), syn::Error> {
        info!("test_build_network_io_enum_normal: START");
        let enum_ident = syn::Ident::new("MyWireIO", proc_macro2::Span::call_site());
        let op_items = vec![
            OperatorSpecItem::new(parse_str::<Path>("AddOp")?),
            OperatorSpecItem::new(parse_str::<Path>("MulOp<X>")?),
        ];
        let (enum_ts,_) = build_network_io_enum(
            &enum_ident,
            &op_items,
            &quote! { <T> },
            &quote! { <T> },
            &quote! {}
        );
        let enum_str = enum_ts.to_string();
        info!("  resulting enum ts: {}", enum_str);
        assert!(enum_str.contains("enum MyWireIO < T >"));
        assert!(enum_str.contains("None"));
        assert!(enum_str.contains("AddOpIO"));
        assert!(enum_str.contains("MulOpIO"));
        Ok(())
    }

    #[test]
    fn test_build_network_io_enum_empty_ops() -> Result<(), syn::Error> {
        info!("test_build_network_io_enum_empty_ops: START");
        let enum_ident = syn::Ident::new("EmptyWireIO", proc_macro2::Span::call_site());
        let op_items = vec![];
        let (enum_ts,_) = build_network_io_enum(
            &enum_ident,
            &op_items,
            &quote! {},
            &quote! {},
            &quote! {}
        );
        let enum_str = enum_ts.to_string();
        info!("  resulting enum ts: {}", enum_str);
        assert!(enum_str.contains("enum EmptyWireIO"));
        assert!(enum_str.contains("None"));
        Ok(())
    }

    #[test]
    fn test_build_operator_signature_map_ok() -> Result<(), syn::Error> {
        info!("test_build_operator_signature_map_ok: START");
        let items = vec![
            OperatorSpecItem::new(try_parse_path("AddOp")?),
            OperatorSpecItem::new(try_parse_path("SubOp<T>")?),
            OperatorSpecItem::new(try_parse_path("CustomOp<X>")?),
        ];
        let sig_map = build_operator_signature_map(&items);
        info!("  sig_map keys: {:?}", sig_map.keys());
        assert_eq!(sig_map["AddOp"].to_string(), "AddOpOperatorSignature");
        assert_eq!(sig_map["SubOp"].to_string(), "SubOpOperatorSignature");
        assert_eq!(sig_map["CustomOp"].to_string(), "CustomOpOperatorSignature");
        Ok(())
    }

    #[test]
    fn test_merge_generics() -> Result<(), syn::Error> {
        info!("test_merge_generics: START");

        // parse the struct wrapper for wire
        let wire_ast: syn::DeriveInput = syn::parse_quote! {
            struct Wire<T, U> {}
        };
        let wire_gens = wire_ast.generics;

        // parse the struct wrapper for op
        let op_ast: syn::DeriveInput = syn::parse_quote! {
            struct Op<V, W> where V: Clone, W: Copy {}
        };
        let op_gens = op_ast.generics;

        let merged = merge_generics(&wire_gens, &op_gens);
        // Instead of `merged.to_token_stream()`, do:
        let (impl_gen, _, wc_opt) = merged.split_for_impl();
        // Combine the pieces
        let merged_str = if let Some(wc) = wc_opt {
            quote::quote! { #impl_gen #wc }.to_string()
        } else {
            quote::quote! { #impl_gen }.to_string()
        };

        info!("  merged_str = {}", merged_str);

        // Now the string should look like "< T , U , V , W > where V : Clone , W : Copy"
        assert!(merged_str.contains("T , U , V , W"), "Got: {}", merged_str);
        assert!(merged_str.contains("where V : Clone , W : Copy"), "Got: {}", merged_str);

        Ok(())
    }

    #[test]
    fn test_combine_where_clauses() -> Result<(), syn::Error> {
        info!("test_combine_where_clauses: START");
        let clause_a: syn::WhereClause = parse_quote! { where T: Clone };
        let clause_b: syn::WhereClause = parse_quote! { where U: Copy };
        let combined = combine_where_clauses(Some(&clause_a), Some(&clause_b));
        let combined_str = combined.to_string();
        info!("  combined_str = {}", combined_str);
        assert!(combined_str.contains("T : Clone , U : Copy"));
        Ok(())
    }

    #[test]
    fn test_build_single_operator_impl_basic() -> Result<(), syn::Error> {
        info!("test_build_single_operator_impl_basic: START");
        let wire_gens: Generics = parse_quote! { <T> };
        let mut op_item = OperatorSpecItem::new(parse_quote! { AddOp<Z> });
        let sig_map = {
            let mut m = std::collections::HashMap::new();
            m.insert("AddOp".to_string(), parse_quote! { AddOpOperatorSignature });
            m
        };
        info!("  finalizing op_item with wire_gens <T>");
        op_item = op_item.finalize_with_wire_gens(&wire_gens);

        let bridging_ts = build_single_operator_impl(
            &syn::Ident::new("MyWireIO", proc_macro2::Span::call_site()),
            &op_item,
            &sig_map,
            &wire_gens
        );
        let bridging_str = bridging_ts.to_string();
        info!("  bridging_str = {}", bridging_str);
        assert!(bridging_str.contains("impl < T , OpTy0 >"));
        assert!(bridging_str.contains("Operator < MyWireIO < T > >"));
        assert!(bridging_str.contains("AddOp < OpTy0 >"));
        Ok(())
    }

    #[test]
    fn test_finalize_operator_io_path() {
        info!("test_finalize_operator_io_path: START");
        // normal path => "AddOp" => "AddOpIO"
        let p: syn::Path = parse_quote!(AddOp);
        info!("  original path = {}", p.to_token_stream());

        let (new_path, new_ident) = finalize_operator_io_path(p);
        info!("  new path = {}", new_path.to_token_stream());
        info!("  new ident = {}", new_ident);

        assert_eq!(new_ident.to_string(), "AddOpIO");
        let last_seg = new_path.segments.last().unwrap();
        assert_eq!(last_seg.ident.to_string(), "AddOpIO");
    }

    #[test]
    fn test_finalize_operator_io_path_no_last_seg() {
        info!("test_finalize_operator_io_path_no_last_seg: START");
        // If we somehow have an empty path => no last seg
        let p = syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::new(),
        };
        info!("  original path is empty");

        let (new_path, new_ident) = finalize_operator_io_path(p);
        info!("  new path = {}", new_path.to_token_stream());
        info!("  new ident = {}", new_ident);

        assert_eq!(new_ident.to_string(), "_____Error_NoLastSeg");
        assert_eq!(new_path.segments.len(), 0, "Still no segments");
    }
}
