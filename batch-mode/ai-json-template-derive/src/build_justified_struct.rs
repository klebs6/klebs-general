// ---------------- [ File: ai-json-template-derive/src/build_justified_struct.rs ]
crate::ix!();

/// Builds a "Justified" struct plus an `impl` with `fn new(...)`.
pub fn build_justified_struct(
    base_ty_ident: &syn::Ident,
    just_ident: &syn::Ident,
    conf_ident: &syn::Ident,
    justified_ident: &syn::Ident,
) -> proc_macro2::TokenStream {
    trace!("Building the main 'Justified' struct type for '{}'", base_ty_ident);

    let struct_def = quote::quote! {
        #[derive(Builder, Debug, Default, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #justified_ident {
            item: #base_ty_ident,
            justification: #just_ident,
            confidence: #conf_ident,
        }
    };

    let impl_block = quote::quote! {
        impl #justified_ident {
            pub fn new(item: #base_ty_ident) -> Self {
                Self {
                    item,
                    justification: Default::default(),
                    confidence: Default::default(),
                }
            }
        }
    };

    let final_ts = quote::quote! {
        #struct_def
        #impl_block
    };

    debug!("Constructed 'Justified' struct definition for '{}'", justified_ident);
    final_ts
}

#[cfg(test)]
mod test_build_justified_struct_exhaustive {
    use super::*;
    use quote::ToTokens;
    use syn::{
        parse2, File, Item, ItemStruct, ImplItem, ImplItemFn, Fields, Visibility,
        Attribute, Path, token, punctuated::Punctuated,
    };
    use tracing::{trace, debug, info, error};

    /// Finds exactly one `struct` in the file AST, or panics.
    fn find_single_struct(ast: &File) -> &ItemStruct {
        let mut found_struct = None;
        for item in &ast.items {
            if let Item::Struct(s) = item {
                if found_struct.is_some() {
                    panic!("Found more than one struct in the tokens!");
                }
                found_struct = Some(s);
            }
        }
        found_struct.expect("No struct found in the tokens!")
    }

    /// Checks that the struct has exactly the three fields: item, justification, confidence,
    /// all private (no `pub`).
    fn assert_struct_has_expected_fields_and_no_pub(
        s: &ItemStruct,
        base_ty_name: &str,
        just_ty_name: &str,
        conf_ty_name: &str
    ) {
        let fields = match &s.fields {
            Fields::Named(n) => &n.named,
            _ => panic!("Expected named fields in the Justified struct"),
        };
        assert_eq!(
            fields.len(),
            3,
            "Justified struct should have exactly 3 fields: item, justification, confidence"
        );

        // We'll gather the names to ensure we have item/justification/confidence
        let mut names_found = Vec::new();

        for f in fields {
            let name = f.ident.as_ref().unwrap().to_string();
            names_found.push(name.clone());

            // check non-pub
            match f.vis {
                Visibility::Inherited => {},
                _ => panic!("Field '{}' is unexpectedly pub or otherwise", name),
            }

            // check type substring
            let type_str = f.ty.to_token_stream().to_string();
            match name.as_str() {
                "item" => {
                    assert!(
                        type_str.contains(base_ty_name),
                        "Expected `item: {}`, got '{}'",
                        base_ty_name,
                        type_str
                    );
                }
                "justification" => {
                    assert!(
                        type_str.contains(just_ty_name),
                        "Expected `justification: {}`, got '{}'",
                        just_ty_name,
                        type_str
                    );
                }
                "confidence" => {
                    assert!(
                        type_str.contains(conf_ty_name),
                        "Expected `confidence: {}`, got '{}'",
                        conf_ty_name,
                        type_str
                    );
                }
                _ => panic!("Unexpected field '{}' in Justified struct", name),
            }
        }

        // Now confirm we have exactly those three:
        for needed in &["item", "justification", "confidence"] {
            // Compare needed as String
            if !names_found.iter().any(|nf| nf == needed) {
                panic!("Expected field '{}' not found in struct fields", needed);
            }
        }
    }

    /// Check there is an `impl JustifiedFoo { fn new(...) -> Self { ... } }` with
    /// justification/confidence set to Default::default().
    fn assert_impl_has_new(ast: &File, justified_name: &str, base_ty_name: &str) {
        // find `impl <something> for JustifiedName`
        let mut found_impl = None;
        for item in &ast.items {
            if let Item::Impl(ii) = item {
                // Check if `impl Foo for Bar`
                if let syn::Type::Path(tp) = &*ii.self_ty {
                    if let Some(seg) = tp.path.segments.last() {
                        if seg.ident == justified_name {
                            found_impl = Some(ii);
                            break;
                        }
                    }
                }
            }
        }
        let the_impl = found_impl.unwrap_or_else(|| {
            panic!("Could not find `impl {}` in the tokens", justified_name);
        });

        // within that impl, find a function named `new`
        let mut found_new = None;
        for itm in &the_impl.items {
            if let ImplItem::Fn(ImplItemFn { sig, block, .. }) = itm {
                if sig.ident == "new" {
                    found_new = Some((sig, block));
                    break;
                }
            }
        }
        let (sig, block) = found_new.expect("No `fn new(...)` found in the impl block");

        // signature: fn new(item: BaseTy) -> Self
        let sig_str = sig.to_token_stream().to_string();
        assert!(
            sig_str.contains("fn new")
            && sig_str.contains(base_ty_name)
            && sig_str.contains("-> Self"),
            "Expected signature like `fn new(item: {base_ty_name}) -> Self`, got: {sig_str}"
        );

        // body: justification: Default::default(), confidence: Default::default()
        let block_str = block.to_token_stream().to_string();
        assert!(
            block_str.contains("justification : Default :: default ()")
            && block_str.contains("confidence : Default :: default ()"),
            "Expected new() body to set justification/confidence to Default::default(), got:\n{block_str}"
        );
    }

    /// Check that we see `#[derive(Builder, Debug, Default, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]`
    fn assert_derive_builder_and_serde(attrs: &[Attribute]) {
        use syn::{Meta, Path, token, punctuated::Punctuated};
        let mut found_derive = false;
        let mut found_traits = vec![];

        for a in attrs {
            if a.path().is_ident("derive") {
                found_derive = true;
                // parse e.g. #[derive(Debug, Clone, etc.)]
                let parsed: syn::Result<Punctuated<Path, token::Comma>> =
                    a.parse_args_with(Punctuated::parse_terminated);
                if let Ok(paths) = parsed {
                    for p in paths {
                        if let Some(seg) = p.segments.last() {
                            found_traits.push(seg.ident.to_string());
                        }
                    }
                }
            }
        }

        assert!(found_derive, "Expected a #[derive(...)] attribute on the struct");
        for needed in &["Builder","Debug","Default","Clone","PartialEq","Serialize","Deserialize","Getters","Setters"] {
            assert!(
                found_traits.contains(&needed.to_string()),
                "Trait '{needed}' missing from #[derive(...)]"
            );
        }
    }

    //
    // ===================== TESTS =====================
    //

    #[test]
    fn test_generates_struct_with_correct_name() {
        let base_ty  = syn::Ident::new("MyType", proc_macro2::Span::call_site());
        let just_ty  = syn::Ident::new("MyTypeJustification", proc_macro2::Span::call_site());
        let conf_ty  = syn::Ident::new("MyTypeConfidence",    proc_macro2::Span::call_site());
        let just_str = syn::Ident::new("JustifiedMyType",     proc_macro2::Span::call_site());

        let tokens = build_justified_struct(&base_ty, &just_ty, &conf_ty, &just_str);
        let ast: File = parse2(tokens).expect("Could not parse as syn::File");

        let s = find_single_struct(&ast);
        assert_eq!(s.ident.to_string(), "JustifiedMyType");
    }

    #[test]
    fn test_struct_has_expected_fields() {
        let base_ty  = syn::Ident::new("Foo", proc_macro2::Span::call_site());
        let just_ty  = syn::Ident::new("FooJustification", proc_macro2::Span::call_site());
        let conf_ty  = syn::Ident::new("FooConfidence", proc_macro2::Span::call_site());
        let just_str = syn::Ident::new("JustifiedFoo", proc_macro2::Span::call_site());

        let tokens = build_justified_struct(&base_ty, &just_ty, &conf_ty, &just_str);
        let ast: File = parse2(tokens).expect("Parse error in test_struct_has_expected_fields");
        let s = find_single_struct(&ast);

        assert_struct_has_expected_fields_and_no_pub(s, "Foo", "FooJustification", "FooConfidence");
    }

    #[test]
    fn test_struct_has_no_pub_fields() {
        let base_ty  = syn::Ident::new("NoPubType", proc_macro2::Span::call_site());
        let just_ty  = syn::Ident::new("NoPubTypeJustification", proc_macro2::Span::call_site());
        let conf_ty  = syn::Ident::new("NoPubTypeConfidence",    proc_macro2::Span::call_site());
        let just_str = syn::Ident::new("JustifiedNoPub",         proc_macro2::Span::call_site());

        let tokens = build_justified_struct(&base_ty, &just_ty, &conf_ty, &just_str);
        let ast: File = parse2(tokens).expect("Parse error in test_struct_has_no_pub_fields");
        let s = find_single_struct(&ast);

        assert_struct_has_expected_fields_and_no_pub(s,
            "NoPubType",
            "NoPubTypeJustification",
            "NoPubTypeConfidence"
        );
    }

    #[test]
    fn test_builder_and_serde_derives_present() {
        let base_ty  = syn::Ident::new("SerdeType",  proc_macro2::Span::call_site());
        let just_ty  = syn::Ident::new("SerdeTypeJustification", proc_macro2::Span::call_site());
        let conf_ty  = syn::Ident::new("SerdeTypeConfidence",    proc_macro2::Span::call_site());
        let just_str = syn::Ident::new("JustifiedSerde",         proc_macro2::Span::call_site());

        let tokens = build_justified_struct(&base_ty, &just_ty, &conf_ty, &just_str);
        let ast: File = parse2(tokens).expect("Parse error for builder+serde test");
        let s = find_single_struct(&ast);

        assert_derive_builder_and_serde(&s.attrs);
    }

    #[test]
    fn test_impl_new_provides_defaults() {
        let base_ty  = syn::Ident::new("SomeType", proc_macro2::Span::call_site());
        let just_ty  = syn::Ident::new("SomeTypeJustification", proc_macro2::Span::call_site());
        let conf_ty  = syn::Ident::new("SomeTypeConfidence", proc_macro2::Span::call_site());
        let just_str = syn::Ident::new("JustifiedWithNew", proc_macro2::Span::call_site());

        let tokens = build_justified_struct(&base_ty, &just_ty, &conf_ty, &just_str);
        let ast: File = parse2(tokens).expect("Could not parse the Justified struct+impl");
        // check the impl block
        assert_impl_has_new(&ast, "JustifiedWithNew", "SomeType");
    }
}
