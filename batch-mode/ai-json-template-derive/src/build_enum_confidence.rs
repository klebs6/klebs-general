// ---------------- [ File: ai-json-template-derive/src/build_enum_confidence.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_enum_confidence(
    enum_conf_ident: &syn::Ident,
    conf_variants: &[proc_macro2::TokenStream],
    first_variant_ident: Option<&syn::Ident>,
    first_variant_conf_fields: &[String],
) -> proc_macro2::TokenStream {
    info!(
        "Starting build_enum_confidence => enum_conf_ident='{}', #conf_variants={}, first_variant_ident={:?}, #first_variant_conf_fields={}",
        enum_conf_ident,
        conf_variants.len(),
        first_variant_ident,
        first_variant_conf_fields.len(),
    );

    for (i, variant_ts) in conf_variants.iter().enumerate() {
        trace!("  conf_variants[{}] => {}", i, variant_ts);
    }
    if !first_variant_conf_fields.is_empty() {
        trace!(
            "The first variant's fields => {:?}",
            first_variant_conf_fields
        );
    }

    // Parse into a real `syn::ItemEnum`
    let enum_def: syn::ItemEnum = syn::parse_quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        enum #enum_conf_ident {
            #( #conf_variants ),*
        }
    };
    debug!("Successfully constructed ItemEnum for '{}'", enum_conf_ident);

    // Optionally build a Default impl if we have a first_variant_ident
    let maybe_impl: Option<syn::ItemImpl> = if let Some(first_variant) = first_variant_ident {
        trace!(
            "Generating Default impl for '{}' with first_variant='{}' ({} fields)",
            enum_conf_ident,
            first_variant,
            first_variant_conf_fields.len()
        );

        // Build each field initializer as: `field_name: Default::default()`
        let init_fields: Vec<proc_macro2::TokenStream> = first_variant_conf_fields
            .iter()
            .map(|field_name| {
                let f = syn::Ident::new(field_name, proc_macro2::Span::call_site());
                quote::quote! { #f: Default::default() }
            })
            .collect();

        // Parse that impl into a real `syn::ItemImpl`
        let impl_def: syn::ItemImpl = syn::parse_quote! {
            impl Default for #enum_conf_ident {
                fn default() -> Self {
                    #enum_conf_ident::#first_variant { #( #init_fields ),* }
                }
            }
        };
        debug!("Successfully constructed Default impl for '{}'", enum_conf_ident);
        Some(impl_def)
    } else {
        trace!("No first_variant_ident provided; skipping Default impl");
        None
    };

    // Combine into final output
    let output = if let Some(impl_def) = maybe_impl {
        debug!("Emitting enum + Default impl for '{}'", enum_conf_ident);
        quote::quote! {
            #enum_def
            #impl_def
        }
    } else {
        debug!("Emitting only enum '{}'", enum_conf_ident);
        quote::quote! {
            #enum_def
        }
    };

    debug!("Final output for '{}':\n{}", enum_conf_ident, output);
    output
}

#[cfg(test)]
mod test_build_enum_confidence_exhaustive {
    use super::*;
    use syn::{parse2, File, Item, ItemEnum, ItemImpl};
    use tracing::*;

    /// Locate exactly one `Item::Enum` in the parsed file, plus gather all `Item::Impl`s.
    /// Panics if there's no enum or if there's more than one enum.
    fn find_enum_and_impls(ast_file: &File) -> (&ItemEnum, Vec<&ItemImpl>) {
        let mut maybe_enum = None;
        let mut impls = Vec::new();
        for item in &ast_file.items {
            match item {
                Item::Enum(e) => {
                    if maybe_enum.is_some() {
                        panic!("Multiple enums found, but test expects exactly one enum item.");
                    }
                    maybe_enum = Some(e);
                }
                Item::Impl(i) => {
                    impls.push(i);
                }
                _ => {
                    // ignore
                }
            }
        }
        let the_enum = maybe_enum.expect("No enum found in generated code.");
        (the_enum, impls)
    }

    #[traced_test]
    fn test_no_variants_no_first_variant_ident() {
        trace!("Starting test_no_variants_no_first_variant_ident");

        let enum_ident = syn::Ident::new("MyConfidence", proc_macro2::Span::call_site());
        let generated = build_enum_confidence(&enum_ident, &[], None, &[]);
        debug!("Generated TokenStream: {}", generated);

        let parsed: syn::File = parse2(generated.clone())
            .expect("Failed to parse generated TokenStream for no-variants test");

        let (the_enum, the_impls) = find_enum_and_impls(&parsed);
        assert_eq!(the_enum.ident, "MyConfidence", "Enum ident mismatch");
        assert!(
            the_enum.variants.is_empty(),
            "Expected zero variants for 'MyConfidence'"
        );
        // No Default impl expected
        assert!(
            the_impls.is_empty(),
            "Should not have generated a Default impl if first_variant_ident is None"
        );

        info!("Finished test_no_variants_no_first_variant_ident");
    }

    #[traced_test]
    fn test_some_variants_no_first_variant_ident() {
        trace!("Starting test_some_variants_no_first_variant_ident");
        let enum_ident = syn::Ident::new("AnotherConfidence", proc_macro2::Span::call_site());
        let variants = vec![
            quote::quote! { AlphaConf { a_conf : f32 } },
            quote::quote! { BetaConf { b_conf : f32 } },
        ];
        let generated = build_enum_confidence(&enum_ident, &variants, None, &[]);
        debug!("Generated TokenStream: {}", generated);

        let parsed: syn::File = parse2(generated.clone())
            .expect("Failed to parse generated tokens in some-variants test");

        let (the_enum, the_impls) = find_enum_and_impls(&parsed);
        assert_eq!(
            the_enum.ident, "AnotherConfidence",
            "Enum ident mismatch for 'AnotherConfidence'"
        );
        assert_eq!(the_enum.variants.len(), 2, "Should have exactly 2 variants");
        assert_eq!(the_enum.variants[0].ident, "AlphaConf");
        assert_eq!(the_enum.variants[1].ident, "BetaConf");
        // No Default impl since first_variant_ident is None
        assert!(
            the_impls.is_empty(),
            "No Default impl expected when first_variant_ident is None"
        );

        info!("Finished test_some_variants_no_first_variant_ident");
    }

    #[traced_test]
    fn test_default_impl_with_first_variant_no_fields() {
        trace!("Starting test_default_impl_with_first_variant_no_fields");
        let enum_ident = syn::Ident::new("NoFieldConfidence", proc_macro2::Span::call_site());
        let variants = vec![
            // first variant has no fields
            quote::quote! { EmptyVariant {} },
            // second variant with something
            quote::quote! { NotEmptyVariant { x_conf : f32 } },
        ];
        let first_variant = syn::Ident::new("EmptyVariant", proc_macro2::Span::call_site());
        let generated = build_enum_confidence(&enum_ident, &variants, Some(&first_variant), &[]);
        debug!("Generated TokenStream: {}", generated);

        let parsed: syn::File =
            parse2(generated.clone()).expect("Could not parse generated code for no-field variant");
        let (the_enum, the_impls) = find_enum_and_impls(&parsed);

        // Check enum ident & structure
        assert_eq!(the_enum.ident, "NoFieldConfidence");
        assert_eq!(the_enum.variants.len(), 2);
        assert_eq!(the_enum.variants[0].ident, "EmptyVariant");
        assert_eq!(the_enum.variants[1].ident, "NotEmptyVariant");

        // Check Default impl
        assert_eq!(
            the_impls.len(),
            1,
            "Expected exactly 1 impl item (Default) for 'NoFieldConfidence'"
        );
        let impl_item = the_impls[0];
        // ensure it's "impl Default for NoFieldConfidence { ... }"
        assert!(
            impl_item.trait_.is_some(),
            "We expected an impl of some trait (Default) but trait_ is None"
        );
        let trait_path = impl_item.trait_.as_ref().unwrap().1.clone();
        let trait_name = quote::quote!(#trait_path).to_string();
        assert!(
            trait_name.contains("Default"),
            "Expected an impl of 'Default' trait, got: {}",
            trait_name
        );

        // check the self type
        if let syn::Type::Path(tp) = &*impl_item.self_ty {
            assert_eq!(
                tp.path.segments.last().unwrap().ident, "NoFieldConfidence",
                "Expected impl self type to be NoFieldConfidence"
            );
        } else {
            panic!("Expected the self type to be a named type path!");
        }

        info!("Finished test_default_impl_with_first_variant_no_fields");
    }

    #[traced_test]
    fn test_default_impl_with_fields_in_first_variant() {
        trace!("Starting test_default_impl_with_fields_in_first_variant");
        let enum_ident = syn::Ident::new("FieldfulConfidence", proc_macro2::Span::call_site());
        let variants = vec![
            quote::quote! { FirstVar { alpha_conf : f32 , gamma_conf : f32 } },
            quote::quote! { SecondVar { beta_conf : f32 } },
        ];
        let first_variant = syn::Ident::new("FirstVar", proc_macro2::Span::call_site());
        let fields = vec!["alpha_conf".to_owned(), "gamma_conf".to_owned()];
        let generated =
            build_enum_confidence(&enum_ident, &variants, Some(&first_variant), &fields);
        debug!("Generated TokenStream: {}", generated);

        let parsed: syn::File = parse2(generated.clone())
            .expect("Failed to parse tokens for fields-in-first-variant test");
        let (the_enum, the_impls) = find_enum_and_impls(&parsed);

        // Check enum ident & structure
        assert_eq!(the_enum.ident, "FieldfulConfidence");
        assert_eq!(the_enum.variants.len(), 2);
        assert_eq!(the_enum.variants[0].ident, "FirstVar");
        assert_eq!(the_enum.variants[1].ident, "SecondVar");

        // We expect exactly 1 impl => "impl Default for FieldfulConfidence"
        assert_eq!(
            the_impls.len(),
            1,
            "Expected 1 impl item (Default) for 'FieldfulConfidence'"
        );
        let impl_item = the_impls[0];
        // check trait is Default
        assert!(
            impl_item.trait_.is_some(),
            "Expected an impl of 'Default' trait but found none"
        );
        let trait_path = impl_item.trait_.as_ref().unwrap().1.clone();
        let trait_name = quote::quote!(#trait_path).to_string();
        assert!(
            trait_name.contains("Default"),
            "Expected an impl of Default trait, got: {}",
            trait_name
        );
        // check the self type
        if let syn::Type::Path(tp) = &*impl_item.self_ty {
            assert_eq!(
                tp.path.segments.last().unwrap().ident, "FieldfulConfidence",
                "Expected impl self type to be 'FieldfulConfidence'"
            );
        } else {
            panic!("Self type is not a named type path!");
        }

        info!("Finished test_default_impl_with_fields_in_first_variant");
    }

    #[traced_test]
    fn test_multiple_variants_some_fields_in_first_none_in_second() {
        trace!("Starting test_multiple_variants_some_fields_in_first_none_in_second");
        let enum_ident = syn::Ident::new("MixedConfidence", proc_macro2::Span::call_site());
        let variants = vec![
            quote::quote! { First { one_conf : f32 , two_conf : f32 } },
            quote::quote! { Second { } },
        ];
        let first_variant = syn::Ident::new("First", proc_macro2::Span::call_site());
        let fields = vec!["one_conf".to_owned(), "two_conf".to_owned()];
        let generated =
            build_enum_confidence(&enum_ident, &variants, Some(&first_variant), &fields);
        debug!("Generated TokenStream: {}", generated);

        let parsed: syn::File = parse2(generated.clone())
            .expect("Failed to parse tokens in multiple-variants test");
        let (the_enum, the_impls) = find_enum_and_impls(&parsed);

        // Check the enum structure
        assert_eq!(the_enum.ident, "MixedConfidence");
        assert_eq!(the_enum.variants.len(), 2);
        assert_eq!(the_enum.variants[0].ident, "First");
        assert_eq!(the_enum.variants[1].ident, "Second");

        // We expect exactly 1 impl => "impl Default for MixedConfidence"
        assert_eq!(
            the_impls.len(),
            1,
            "Expected exactly 1 impl item for 'MixedConfidence'"
        );
        let impl_item = the_impls[0];
        // check trait is Default
        assert!(
            impl_item.trait_.is_some(),
            "Expected an impl of the Default trait but found none"
        );
        let trait_path = impl_item.trait_.as_ref().unwrap().1.clone();
        let trait_name = quote::quote!(#trait_path).to_string();
        assert!(
            trait_name.contains("Default"),
            "Expected an impl of Default trait, got: {}",
            trait_name
        );
        // check the self type
        if let syn::Type::Path(tp) = &*impl_item.self_ty {
            assert_eq!(
                tp.path.segments.last().unwrap().ident, "MixedConfidence",
                "Expected impl self type to be 'MixedConfidence'"
            );
        } else {
            panic!("Self type is not a named type path!");
        }

        info!("Finished test_multiple_variants_some_fields_in_first_none_in_second");
    }

    #[traced_test]
    fn test_first_variant_ident_is_none_but_fields_are_provided() {
        trace!("Starting test_first_variant_ident_is_none_but_fields_are_provided");
        let enum_ident = syn::Ident::new("OrphanConfidence", proc_macro2::Span::call_site());
        let variants = vec![
            quote::quote! { VarA { a_conf : f32 } },
            quote::quote! { VarB { b_conf : f32 } },
        ];
        // fields but no first_variant_ident
        let generated = build_enum_confidence(&enum_ident, &variants, None, &["a_conf".to_owned()]);
        debug!("Generated TokenStream: {}", generated);

        let parsed: syn::File = parse2(generated.clone())
            .expect("Failed to parse tokens for orphan test");
        let (the_enum, the_impls) = find_enum_and_impls(&parsed);

        // Check the enum
        assert_eq!(the_enum.ident, "OrphanConfidence");
        assert_eq!(the_enum.variants.len(), 2);
        assert_eq!(the_enum.variants[0].ident, "VarA");
        assert_eq!(the_enum.variants[1].ident, "VarB");

        // first_variant_ident is None => no default impl
        assert!(
            the_impls.is_empty(),
            "Should NOT have a default impl if first_variant_ident is None"
        );

        info!("Finished test_first_variant_ident_is_none_but_fields_are_provided");
    }
}
