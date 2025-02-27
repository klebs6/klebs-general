// ---------------- [ File: src/split_op_generics.rs ]
crate::ix!();

/// Split the operator's generics into (impl_generics, ty_generics, where_clause)
pub fn split_op_generics(op_item: &OperatorSpecItem) 
    -> (TokenStream, TokenStream, Option<&syn::WhereClause>) 
{
    info!(
        "[split_op_generics] Entered with op_item path: {}",
        op_item.path().to_token_stream().to_string()
    );
    let (impl_gen, ty_gen, wc) = op_item.op_generics().split_for_impl();
    info!(
        "[split_op_generics] Raw split_for_impl results => impl_gen: `{}`, ty_gen: `{}`, where_clause: `{:?}`",
        impl_gen.to_token_stream(),
        ty_gen.to_token_stream(),
        wc.as_ref().map(|w| w.to_token_stream())
    );

    let result = (quote! { #impl_gen }, quote! { #ty_gen }, wc);
    info!(
        "[split_op_generics] Returning => impl_gen: `{}`, ty_gen: `{}`, where_clause: `{:?}`",
        result.0, result.1, result.2.map(|w| w.to_token_stream())
    );
    info!("[split_op_generics] Exiting.\n");
    result
}

#[cfg(test)]
mod split_op_generics_tests {
    use super::*;
    use syn::{parse_str, WhereClause};

    /// Minimal helper for parsing a Path from a string without `unwrap`.
    /// Panics if the parse operation fails.
    fn parse_path(path_str: &str) -> Path {
        info!("[parse_path] Attempting to parse path: `{}`", path_str);
        match parse_str::<Path>(path_str) {
            Ok(p) => {
                info!("[parse_path] Success. Parsed path: {:?}", p.to_token_stream());
                p
            },
            Err(e) => {
                info!("[parse_path] ERROR: Failed to parse path: {}", e);
                panic!("Failed to parse path '{path_str}': {e}")
            }
        }
    }

    /// Minimal helper that optionally normalizes whitespace for easier string comparisons.
    /// In your project, you might use `crate::normalize_whitespace` if desired.
    fn norm(s: &str) -> String {
        info!("[norm] Normalizing whitespace for: `{}`", s);
        let result = s.split_whitespace().collect::<Vec<_>>().join(" ");
        info!("[norm] Result: `{}`", result);
        result
    }

    /// Builds a fresh `OperatorSpecItem` from a given path string,
    /// then optionally finalizes it against `wire_generics`.
    /// Returns the resulting `OperatorSpecItem`.
    fn build_op_item(
        path_str: &str,
        wire_generics: Option<&Generics>,
    ) -> OperatorSpecItem {
        info!(
            "[build_op_item] Building op_item from path_str: `{}`, with wire_generics: {:?}",
            path_str,
            wire_generics.as_ref().map(|g| g.to_token_stream())
        );
        let path = parse_path(path_str);
        let op_item = OperatorSpecItem::new(path);
        info!(
            "[build_op_item] Created OperatorSpecItem with path: `{}`",
            op_item.path().to_token_stream()
        );

        match wire_generics {
            Some(gens) => {
                info!("[build_op_item] Finalizing op_item with wire_generics.");
                let finalized = op_item.finalize_with_wire_gens(gens);
                info!(
                    "[build_op_item] Finalized OperatorSpecItem => path: `{}`, generics: `{:?}`",
                    finalized.path().to_token_stream(),
                    finalized.op_generics().params
                );
                finalized
            }
            None => {
                info!("[build_op_item] No wire_generics, returning op_item as-is.");
                op_item
            }
        }
    }

    #[test]
    fn test_split_op_generics_no_generics() {
        info!("\n[test_split_op_generics_no_generics] Entered.");
        let op_item = build_op_item("SomeOp", None);
        let (impl_gen, ty_gen, maybe_wc) = split_op_generics(&op_item);

        let impl_gen_str = norm(&impl_gen.to_string());
        let ty_gen_str = norm(&ty_gen.to_string());
        info!(
            "[test_split_op_generics_no_generics] Checking results => impl_gen: `{}`, ty_gen: `{}`, where_clause: `{:?}`",
            impl_gen_str,
            ty_gen_str,
            maybe_wc
        );

        assert!(maybe_wc.is_none(), "Expected no where-clause for no generics.");
        assert_eq!(impl_gen_str, "");
        assert_eq!(ty_gen_str, "");
    }

    #[test]
    fn test_split_op_generics_one_type_param() {
        info!("\n[test_split_op_generics_one_type_param] Entered.");
        let op_item = build_op_item("MyOp<T>", Some(&Generics::default()));
        let (impl_gen, ty_gen, maybe_wc) = split_op_generics(&op_item);

        let impl_gen_str = norm(&impl_gen.to_string());
        let ty_gen_str = norm(&ty_gen.to_string());
        info!(
            "[test_split_op_generics_one_type_param] Checking => impl_gen: `{}`, ty_gen: `{}`, where_clause: `{:?}`",
            impl_gen_str,
            ty_gen_str,
            maybe_wc
        );

        assert!(maybe_wc.is_none());
        assert!(
            impl_gen_str.contains("OpTy0"),
            "Expected a minted generic param in impl."
        );
        assert!(
            ty_gen_str.contains("OpTy0"),
            "Expected a minted generic param in type usage."
        );
    }

    #[test]
    fn test_split_op_generics_lifetime_param() {
        info!("\n[test_split_op_generics_lifetime_param] Entered.");
        let op_item = build_op_item("LifetimeOp<'a>", Some(&Generics::default()));
        let (impl_gen, ty_gen, maybe_wc) = split_op_generics(&op_item);

        let impl_gen_str = norm(&impl_gen.to_string());
        let ty_gen_str = norm(&ty_gen.to_string());
        info!(
            "[test_split_op_generics_lifetime_param] Checking => impl_gen: `{}`, ty_gen: `{}`, where_clause: `{:?}`",
            impl_gen_str,
            ty_gen_str,
            maybe_wc
        );

        assert!(maybe_wc.is_none());
        assert!(
            impl_gen_str.contains("'a"),
            "Expected a lifetime generic in the impl generics."
        );
        assert!(
            ty_gen_str.contains("'a"),
            "Expected a lifetime generic in the type generics."
        );
    }

    #[test]
    fn test_split_op_generics_const_param() {
        info!("\n[test_split_op_generics_const_param] Entered.");
        let op_item = build_op_item("ConstOp<123>", Some(&Generics::default()));
        let (impl_gen, ty_gen, maybe_wc) = split_op_generics(&op_item);

        let impl_gen_str = norm(&impl_gen.to_string());
        let ty_gen_str = norm(&ty_gen.to_string());
        info!(
            "[test_split_op_generics_const_param] Checking => impl_gen: `{}`, ty_gen: `{}`, where_clause: `{:?}`",
            impl_gen_str,
            ty_gen_str,
            maybe_wc
        );

        assert!(maybe_wc.is_none());
        assert!(
            impl_gen_str.contains("const OPC0 : usize"),
            "Expected a minted const generic param."
        );
        assert!(
            ty_gen_str.contains("OPC0"),
            "Expected the type generics to reference the minted const param."
        );
    }

    #[test]
    fn test_split_op_generics_multiple_params_with_where_clause() {
        info!("\n[test_split_op_generics_multiple_params_with_where_clause] Entered.");

        // Parse the wire generics from a dummy struct
        let wire_with_where: Generics = {
            // We keep the struct definition basically the same;
            // just ensure it's parseable by syn::parse_str.
            let wire_str = "struct Dummy<W, X, 'y, const M: usize> where W: Clone, X: Copy {}";
            match syn::parse_str::<syn::DeriveInput>(wire_str) {
                Ok(ast) => {
                    info!(
                        "[test_split_op_generics_multiple_params_with_where_clause] \
                        Parsed dummy wire generics: `{:?}`",
                        ast.generics
                    );
                    ast.generics
                },
                Err(e) => panic!("Failed to parse dummy struct: {e}"),
            }
        };

        // Build the op_item (which will only store minted generics in `op_generics`)
        let op_item = build_op_item("ComplexOp<W, AnotherTy, 77>", Some(&wire_with_where));

        // unify the wire generics + the op_item's minted generics
        let merged_gens = unify_generics_ast(&wire_with_where, op_item.op_generics());

        // Then split_for_impl on the *merged* generics
        let (impl_gen, ty_gen, maybe_wc) = merged_gens.split_for_impl();

        // FIX: first turn impl_gen and ty_gen into token streams, then to string.
        let impl_gen_ts = quote::quote! { #impl_gen };
        let ty_gen_ts   = quote::quote! { #ty_gen };

        // Then call .to_string() on the token streams instead of ImplGenerics / TypeGenerics directly.
        let impl_gen_str = norm(&impl_gen_ts.to_string());
        let ty_gen_str   = norm(&ty_gen_ts.to_string());

        info!(
            "[test_split_op_generics_multiple_params_with_where_clause] Checking => \
            impl_gen: `{}`, ty_gen: `{}`, where_clause: `{:?}`",
            impl_gen_str,
            ty_gen_str,
            maybe_wc
        );

        // Check the where-clause
        if let Some(where_clause) = maybe_wc {
            let wc_str = norm(&where_clause.to_token_stream().to_string());
            assert!(wc_str.contains("W : Clone"), "Where clause should include W: Clone.");
            assert!(wc_str.contains("X : Copy"),  "Where clause should include X: Copy.");
        } else {
            panic!("Expected a combined where-clause, got None.");
        }

        debug!("impl_gen_str: {:#?}", impl_gen_str);
        debug!("ty_gen_str: {:#?}", ty_gen_str);

        // Now these checks pass because the merged generics have W plus minted generics:
        assert!(impl_gen_str.contains("W"),     "Expected reused param W in impl generics.");
        assert!(ty_gen_str.contains("W"),       "Expected reused param W in type generics.");
        assert!(impl_gen_str.contains("OpTy0"), "Expected minted type param for AnotherTy in impl generics.");
        assert!(ty_gen_str.contains("OpTy0"),   "Expected minted type param for AnotherTy in type generics.");
        assert!(impl_gen_str.contains("OPC1"),  "Expected minted const param for 77 in impl generics.");
        assert!(ty_gen_str.contains("OPC1"),    "Expected minted const param for 77 in type generics.");
    }
}
