// ---------------- [ File: src/split_wire_generics.rs ]
crate::ix!();

fn ensure_brackets(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        "< >".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn split_wire_generics(wire_generics: &syn::Generics) -> (TokenStream, TokenStream, Option<&syn::WhereClause>) {
    info!("split_wire_generics: START");
    info!("  input = {}", wire_generics.into_token_stream());

    let (impl_gen, ty_gen, wc) = wire_generics.split_for_impl();
    info!("  split_for_impl() results =>");
    info!("    impl_gen = {}", impl_gen.to_token_stream());
    info!("    ty_gen   = {}", ty_gen.to_token_stream());
    if let Some(ref wc_actual) = wc {
        info!("    where_cl = {}", wc_actual.to_token_stream());
    } else {
        info!("    where_cl = None");
    }

    // Convert to strings
    let impl_str = impl_gen.to_token_stream().to_string();
    let ty_str   = ty_gen.to_token_stream().to_string();

    // If empty, produce "< >"
    let final_impl_str = ensure_brackets(&impl_str);
    let final_ty_str   = ensure_brackets(&ty_str);

    info!("  returning => impl_ts = {}, ty_ts = {}, wc = {:?}", 
              final_impl_str, final_ty_str, wc);

    let impl_ts = syn::parse_str::<proc_macro2::TokenStream>(&final_impl_str)
        .expect("should parse back to TokenStream");
    let ty_ts = syn::parse_str::<proc_macro2::TokenStream>(&final_ty_str)
        .expect("should parse back to TokenStream");

    info!("split_wire_generics: FINISH\n");
    (impl_ts, ty_ts, wc)
}

#[cfg(test)]
mod test_split_wire_generics {
    use super::*;

    /// A small helper for parsing a `struct` with given generics and
    /// optional where-clause. Returns the `Generics` component of the AST.
    fn parse_generics(struct_text: &str) -> Generics {
        info!("parse_generics: START => '{}'", struct_text);
        match parse_str::<DeriveInput>(struct_text) {
            Ok(derive_input) => {
                info!("  parse_generics succeeded => {}", derive_input.generics.to_token_stream());
                derive_input.generics
            }
            Err(e) => {
                info!("  parse_generics failed => {}", e);
                panic!("Failed to parse test struct: {}", e);
            }
        }
    }

    /// Normalize whitespace to facilitate robust string comparison.
    fn normalize_ws(s: &str) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn test_no_generics() {
        info!("test_no_generics: START");
        let generics = parse_generics(r#"struct Foo;"#);
        let (impl_ts, ty_ts, wc_opt) = split_wire_generics(&generics);

        let impl_str = normalize_ws(&impl_ts.to_string());
        let ty_str   = normalize_ws(&ty_ts.to_string());
        info!("  impl_str = {}", impl_str);
        info!("  ty_str   = {}", ty_str);
        info!("  wc_opt   = {:?}", wc_opt);

        assert_eq!(impl_str, "< >", "Expected empty impl generics '< >'");
        assert_eq!(ty_str, "< >",   "Expected empty ty generics '< >'");
        assert!(wc_opt.is_none(),   "Expected no where-clause");
    }

    #[test]
    fn test_lifetime_only() {
        info!("test_lifetime_only: START");
        let generics = parse_generics(r#"struct Foo<'a> { bar: &'a str }"#);
        let (impl_ts, ty_ts, wc_opt) = split_wire_generics(&generics);

        let impl_str = normalize_ws(&impl_ts.to_string());
        let ty_str   = normalize_ws(&ty_ts.to_string());
        info!("  impl_str = {}", impl_str);
        info!("  ty_str   = {}", ty_str);
        info!("  wc_opt   = {:?}", wc_opt);

        assert_eq!(impl_str, "< 'a >", "Expected <'a> in impl generics");
        assert_eq!(ty_str,   "< 'a >", "Expected <'a> in ty generics");
        assert!(wc_opt.is_none());
    }

    #[test]
    fn test_single_type_param() {
        info!("test_single_type_param: START");
        let generics = parse_generics(r#"struct Foo<T> { value: T }"#);
        let (impl_ts, ty_ts, wc_opt) = split_wire_generics(&generics);

        let impl_str = normalize_ws(&impl_ts.to_string());
        let ty_str   = normalize_ws(&ty_ts.to_string());
        info!("  impl_str = {}", impl_str);
        info!("  ty_str   = {}", ty_str);

        assert_eq!(impl_str, "< T >", "Expected <T> in impl generics");
        assert_eq!(ty_str,   "< T >", "Expected <T> in ty generics");
        assert!(wc_opt.is_none());
    }

    #[test]
    fn test_multiple_type_and_lifetime_params() {
        info!("test_multiple_type_and_lifetime_params: START");
        let generics = parse_generics(
            r#"
            struct Bar<'x, A, B> 
            where 
                A: Clone,
                B: Send,
            {
                a: &'x A,
                b: B
            }
            "#
        );
        let (impl_ts, ty_ts, wc_opt) = split_wire_generics(&generics);

        let impl_str = normalize_ws(&impl_ts.to_string());
        let ty_str   = normalize_ws(&ty_ts.to_string());
        info!("  impl_str = {}", impl_str);
        info!("  ty_str   = {}", ty_str);
        info!("  wc_opt   = {:?}", wc_opt.map(|w| w.to_token_stream().to_string()));

        assert!(impl_str.contains("'x") && impl_str.contains("A") && impl_str.contains("B"));
        assert!(ty_str.contains("'x") && ty_str.contains("A") && ty_str.contains("B"));

        let wc = wc_opt.expect("Expected a where-clause, got None");
        let wc_tokens = normalize_ws(&wc.to_token_stream().to_string());
        assert!(wc_tokens.contains("A : Clone"));
        assert!(wc_tokens.contains("B : Send"));
    }

    #[test]
    fn test_generic_with_bounds() {
        info!("test_generic_with_bounds: START");
        let generics = parse_generics(r#"
            struct Baz<T: std::fmt::Debug + 'static>
            {
                t: T
            }
        "#);
        let (impl_ts, ty_ts, wc_opt) = split_wire_generics(&generics);

        let impl_str = normalize_ws(&impl_ts.to_string());
        let ty_str   = normalize_ws(&ty_ts.to_string());
        info!("  impl_str = {}", impl_str);
        info!("  ty_str   = {}", ty_str);

        assert!(impl_str.contains("T"));
        assert!(ty_str.contains("T"));
        assert!(wc_opt.is_none(), "Expected no explicit where-clause");
    }

    #[test]
    fn test_where_clause_only() {
        info!("test_where_clause_only: START");
        let generics = parse_generics(r#"
            struct Qux<T>
            where
                T: Clone + Sync,
            {
                field: T
            }
        "#);
        let (impl_ts, ty_ts, wc_opt) = split_wire_generics(&generics);

        let impl_str = normalize_ws(&impl_ts.to_string());
        let ty_str   = normalize_ws(&ty_ts.to_string());
        info!("  impl_str = {}", impl_str);
        info!("  ty_str   = {}", ty_str);

        assert_eq!(impl_str, "< T >", "Expected <T> in impl generics");
        assert_eq!(ty_str,   "< T >", "Expected <T> in ty generics");
        
        let wc = wc_opt.expect("Expected a where-clause for Qux, got None");
        let wc_tokens = normalize_ws(&wc.to_token_stream().to_string());
        info!("  wc_tokens = {}", wc_tokens);

        // Relaxed check: verifying that T has both Clone and Sync
        // combined into T : Clone + Sync
        assert!(wc_tokens.contains("T : Clone + Sync"), "Got: {}", wc_tokens);
    }

    #[test]
    fn test_split_wire_generics() {
        info!("test_split_wire_generics: START");
        let gens: syn::Generics = parse_quote! { <T:Clone, U:Debug> };
        info!("  input generics = {:?}", gens);

        let (impl_ts, ty_ts, wc) = split_wire_generics(&gens);
        info!("  impl_ts = {}", impl_ts.to_string());
        info!("  ty_ts   = {}", ty_ts.to_string());
        if let Some(w) = &wc {
            info!("  where-clause = {}", w.to_token_stream());
        } else {
            info!("  where-clause = None");
        }

        assert!(impl_ts.to_string().contains("< T : Clone , U : Debug >"));
        assert!(ty_ts.to_string().contains("< T , U >"));
        assert!(wc.is_none());
    }
}
