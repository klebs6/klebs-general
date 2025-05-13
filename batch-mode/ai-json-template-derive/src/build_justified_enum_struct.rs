// ---------------- [ File: ai-json-template-derive/src/build_justified_enum_struct.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_justified_enum_struct(
    ty_ident: &syn::Ident,
    enum_just_ident: &syn::Ident,
    enum_conf_ident: &syn::Ident,
    justified_ident: &syn::Ident
) -> TokenStream2
{
    debug!(
        "Building the final Justified struct '{}' for enum '{}'",
        justified_ident,
        ty_ident
    );

    quote::quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Getters, Setters)]
        #[getset(get="pub", set="pub")]
        struct #justified_ident {
            item: #ty_ident,
            justification: #enum_just_ident,
            confidence: #enum_conf_ident,
        }

        impl #justified_ident {
            fn new(item: #ty_ident) -> Self {
                Self {
                    item,
                    justification: ::core::default::Default::default(),
                    confidence: ::core::default::Default::default(),
                }
            }
        }
    }
}

#[cfg(test)]
mod test_build_justified_enum_struct {
    use super::*;

    #[traced_test]
    fn it_generates_a_struct_with_the_correct_name() {
        trace!("Starting test: it_generates_a_struct_with_the_correct_name");

        let ty_ident = syn::Ident::new("MyEnum", proc_macro2::Span::call_site());
        let enum_just_ident = syn::Ident::new("MyEnumJustification", proc_macro2::Span::call_site());
        let enum_conf_ident = syn::Ident::new("MyEnumConfidence", proc_macro2::Span::call_site());
        let justified_ident = syn::Ident::new("JustifiedMyEnum", proc_macro2::Span::call_site());

        debug!("Calling build_justified_enum_struct(...) with ident='{}'", justified_ident);
        let tokens: TokenStream2 = build_justified_enum_struct(
            &ty_ident,
            &enum_just_ident,
            &enum_conf_ident,
            &justified_ident
        );
        trace!("Generated token stream: {}", tokens);

        let file_syntax: syn::File = parse2(tokens).expect("Could not parse token stream into a syn::File");
        let mut found_struct = false;

        for item in file_syntax.items {
            if let Item::Struct(item_struct) = item {
                debug!("Found struct: {}", item_struct.ident);
                if item_struct.ident == justified_ident {
                    found_struct = true;
                    // Confirm the fields
                    match &item_struct.fields {
                        Fields::Named(named) => {
                            let mut field_names = vec![];
                            for field in &named.named {
                                field_names.push(field.ident.as_ref().unwrap().to_string());
                            }
                            trace!("Struct fields = {:?}", field_names);
                            assert!(field_names.contains(&"item".to_string()));
                            assert!(field_names.contains(&"justification".to_string()));
                            assert!(field_names.contains(&"confidence".to_string()));
                        }
                        _ => {
                            error!("Expected named fields on the generated struct, but did not find them.");
                            panic!("Generated struct did not have named fields.");
                        }
                    }
                }
            }
        }
        assert!(found_struct, "Expected to find struct '{}'", justified_ident);
    }

    #[traced_test]
    fn it_derives_necessary_traits() {
        trace!("Starting test: it_derives_necessary_traits");

        let ty_ident = syn::Ident::new("FooEnum", proc_macro2::Span::call_site());
        let just_ident = syn::Ident::new("FooEnumJustification", proc_macro2::Span::call_site());
        let conf_ident = syn::Ident::new("FooEnumConfidence", proc_macro2::Span::call_site());
        let justified_ident = syn::Ident::new("JustifiedFooEnum", proc_macro2::Span::call_site());

        let tokens = build_justified_enum_struct(&ty_ident, &just_ident, &conf_ident, &justified_ident);
        trace!("Token stream: {}", tokens);

        let file_syntax: syn::File = parse2(tokens).expect("Could not parse token stream");

        let mut found_struct_with_derives = false;

        for item in file_syntax.items {
            if let Item::Struct(item_struct) = item {
                if item_struct.ident == justified_ident {
                    found_struct_with_derives = true;

                    let mut found_derive_debug = false;
                    let mut found_derive_clone = false;
                    let mut found_derive_partial_eq = false;
                    let mut found_derive_serialize = false;
                    let mut found_derive_deserialize = false;
                    let mut found_derive_default = false;
                    let mut found_derive_getters = false;
                    let mut found_derive_setters = false;

                    for attr in &item_struct.attrs {
                        if attr.path().is_ident("derive") {
                            let parsed = attr.parse_nested_meta(|meta| {
                                if let Some(ident) = meta.path.get_ident() {
                                    let derived_str = ident.to_string();
                                    trace!("Derived: {}", derived_str);

                                    match derived_str.as_str() {
                                        "Debug" => found_derive_debug = true,
                                        "Clone" => found_derive_clone = true,
                                        "PartialEq" => found_derive_partial_eq = true,
                                        "Serialize" => found_derive_serialize = true,
                                        "Deserialize" => found_derive_deserialize = true,
                                        "Default" => found_derive_default = true,
                                        "Getters" => found_derive_getters = true,
                                        "Setters" => found_derive_setters = true,
                                        _ => (),
                                    }
                                }
                                Ok(())
                            });

                            if let Err(e) = parsed {
                                error!("Failed to parse nested meta: {:?}", e);
                                panic!("Parsing nested meta for derive attribute failed: {e}");
                            }
                        }
                    }

                    assert!(found_derive_debug,         "Expected Debug derive");
                    assert!(found_derive_clone,         "Expected Clone derive");
                    assert!(found_derive_partial_eq,    "Expected PartialEq derive");
                    assert!(found_derive_serialize,     "Expected Serialize derive");
                    assert!(found_derive_deserialize,   "Expected Deserialize derive");
                    assert!(found_derive_default,       "Expected Default derive");
                    assert!(found_derive_getters,       "Expected Getters derive");
                    assert!(found_derive_setters,       "Expected Setters derive");
                }
            }
        }

        assert!(found_struct_with_derives, "Expected to find a struct with the correct derives.");
    }


    #[traced_test]
    fn it_includes_impl_new_method() {
        trace!("Starting test: it_includes_impl_new_method");

        let ty_ident = syn::Ident::new("BarEnum", proc_macro2::Span::call_site());
        let just_ident = syn::Ident::new("BarEnumJustification", proc_macro2::Span::call_site());
        let conf_ident = syn::Ident::new("BarEnumConfidence", proc_macro2::Span::call_site());
        let justified_ident = syn::Ident::new("JustifiedBarEnum", proc_macro2::Span::call_site());

        let tokens = build_justified_enum_struct(&ty_ident, &just_ident, &conf_ident, &justified_ident);
        trace!("Token stream: {}", tokens);

        let syntax: syn::File = parse2(tokens).expect("Could not parse token stream");
        let mut impl_found = false;
        let mut new_method_found = false;

        for item in syntax.items {
            if let Item::Impl(item_impl) = item {
                trace!("Found an impl block");
                // Check if it's `impl JustifiedBarEnum`
                if let Some((_, path, _)) = &item_impl.trait_ {
                    warn!("Found an impl with a trait? Not expecting it. The trait path is {:?}", path);
                } else {
                    // Not a trait impl => presumably `impl JustifiedBarEnum`
                    // check the self type
                    if let Type::Path(tp) = &*item_impl.self_ty {
                        let last_seg = tp.path.segments.last().expect("No segments in type path");
                        if last_seg.ident == justified_ident {
                            impl_found = true;
                            // look for `fn new(...)`
                            for inner_item in &item_impl.items {
                                if let syn::ImplItem::Fn(method) = inner_item {
                                    if method.sig.ident == "new" {
                                        new_method_found = true;
                                        // Ensure it has a param matching `item: BarEnum`
                                        let params = &method.sig.inputs;
                                        // expecting 1 param after &self?
                                        // Actually "fn new(item: #ty_ident) -> Self"
                                        // so there's no &self param at all, just &Self
                                        // Actually it's static, so no 'self param' => first param is the item
                                        let mut param_count = 0;
                                        for param in params {
                                            if let syn::FnArg::Typed(pat_type) = param {
                                                param_count += 1;
                                                // check the type
                                                if let Type::Path(tpt) = &*pat_type.ty {
                                                    let seg_ident = &tpt.path.segments.last().unwrap().ident;
                                                    assert_eq!(seg_ident, &ty_ident, "The 'new' method param type did not match the input enum ident");
                                                }
                                            }
                                        }
                                        assert_eq!(param_count, 1, "Expected exactly one parameter for 'new' method");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        assert!(impl_found, "Expected to find an impl block for '{}'", justified_ident);
        assert!(new_method_found, "Expected to find a 'new' method in the impl for '{}'", justified_ident);
    }

    #[traced_test]
    fn it_is_free_of_public_fields() {
        trace!("Starting test: it_is_free_of_public_fields");

        let ty_ident = syn::Ident::new("PublessEnum", proc_macro2::Span::call_site());
        let just_ident = syn::Ident::new("PublessEnumJustification", proc_macro2::Span::call_site());
        let conf_ident = syn::Ident::new("PublessEnumConfidence", proc_macro2::Span::call_site());
        let justified_ident = syn::Ident::new("JustifiedPublessEnum", proc_macro2::Span::call_site());

        let tokens = build_justified_enum_struct(&ty_ident, &just_ident, &conf_ident, &justified_ident);
        trace!("Token stream: {}", tokens);

        let syntax: syn::File = parse2(tokens).expect("Could not parse token stream");

        let mut tested_any_struct_fields = false;
        for item in syntax.items {
            if let Item::Struct(item_struct) = item {
                if item_struct.ident == justified_ident {
                    // Check each field for "pub"
                    if let Fields::Named(named_fields) = &item_struct.fields {
                        for field in &named_fields.named {
                            tested_any_struct_fields = true;
                            assert!(field.vis.to_token_stream().to_string() != "pub", "Expected no 'pub' fields, but found one");
                        }
                    }
                }
            }
        }

        assert!(tested_any_struct_fields, "No struct fields were found, or unexpected parse structure. Could not confirm lack of 'pub' fields.");
    }

    #[traced_test]
    fn it_produces_valid_syntax_for_arbitrary_idents() {
        trace!("Starting test: it_produces_valid_syntax_for_arbitrary_idents");

        // Trying an enum name with underscores or digits
        let ty_ident = syn::Ident::new("Enum_42", proc_macro2::Span::call_site());
        let just_ident = syn::Ident::new("Enum_42Justification", proc_macro2::Span::call_site());
        let conf_ident = syn::Ident::new("Enum_42Confidence", proc_macro2::Span::call_site());
        let justified_ident = syn::Ident::new("JustifiedEnum_42", proc_macro2::Span::call_site());

        let tokens = build_justified_enum_struct(&ty_ident, &just_ident, &conf_ident, &justified_ident);
        trace!("Generated: {}", tokens);

        // Ensure it parses cleanly
        let syntax: syn::File = parse2(tokens).expect("Failed to parse token stream from arbitrary idents");
        let mut found_any_struct = false;
        for item in syntax.items {
            if let Item::Struct(s) = item {
                if s.ident == justified_ident {
                    found_any_struct = true;
                    assert!(s.fields.len() == 3, "Expected three fields in the struct (item, justification, confidence)");
                }
            }
        }
        assert!(found_any_struct, "Did not find expected struct for 'JustifiedEnum_42'");
    }
}
