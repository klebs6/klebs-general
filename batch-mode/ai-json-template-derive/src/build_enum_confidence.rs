// ---------------- [ File: ai-json-template-derive/src/build_enum_confidence.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_enum_confidence(
    enum_conf_ident: &syn::Ident,
    conf_variants: &[proc_macro2::TokenStream],
    first_variant_ident: Option<&syn::Ident>,
    first_variant_conf_fields: &[String]
) -> proc_macro2::TokenStream
{
    debug!(
        "Building confidence enum '{}' with {} variant(s)",
        enum_conf_ident,
        conf_variants.len()
    );

    let variants_ts = quote::quote! { #( #conf_variants ),* };
    let default_impl = if let Some(first_variant) = first_variant_ident {
        let init_fields: Vec<_> = first_variant_conf_fields.iter().map(|f_str| {
            let f_id = syn::Ident::new(f_str, proc_macro2::Span::call_site());
            quote::quote! { #f_id: ::core::default::Default::default() }
        }).collect();

        quote::quote! {
            impl ::core::default::Default for #enum_conf_ident {
                fn default() -> Self {
                    #enum_conf_ident::#first_variant { #( #init_fields ),* }
                }
            }
        }
    } else {
        quote::quote!()
    };

    quote::quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        enum #enum_conf_ident {
            #variants_ts
        }
        #default_impl
    }
}

#[cfg(test)]
mod test_build_enum_confidence_exhaustive {
    use super::*;

    #[traced_test]
    fn test_no_variants_no_first_variant_ident() {
        trace!("Starting test_no_variants_no_first_variant_ident");
        let enum_ident = syn::Ident::new("MyConfidence", proc_macro2::Span::call_site());
        let generated = build_enum_confidence(&enum_ident, &[], None, &[]);
        debug!("Generated TokenStream: {}", generated);

        // Parse into a syn::File to ensure it compiles as valid Rust
        let parsed: File = match parse2(generated.clone()) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to parse generated TokenStream: {:?}", e);
                panic!("Could not parse code: {}", e);
            }
        };

        // We'll check there's an enum named `MyConfidence` with 0 variants and no default impl
        let file_str = format!("{:?}", parsed);
        assert!(file_str.contains("enum MyConfidence"));
        assert!(!file_str.contains("impl Default for MyConfidence"));
        info!("Finished test_no_variants_no_first_variant_ident");
    }

    #[traced_test]
    fn test_some_variants_no_first_variant_ident() {
        trace!("Starting test_some_variants_no_first_variant_ident");
        let enum_ident = syn::Ident::new("AnotherConfidence", proc_macro2::Span::call_site());
        let variants = vec![
            quote! { AlphaConf { a_conf: f32 } },
            quote! { BetaConf { b_conf: f32 } },
        ];

        let generated = build_enum_confidence(&enum_ident, &variants, None, &[]);
        debug!("Generated TokenStream: {}", generated);

        let parsed: File = parse2(generated.clone()).expect("Failed to parse generated tokens");
        let file_str = format!("{:?}", parsed);
        // check the variants
        assert!(file_str.contains("AlphaConf"));
        assert!(file_str.contains("BetaConf"));
        // no default impl
        assert!(!file_str.contains("impl Default for AnotherConfidence"));
        info!("Finished test_some_variants_no_first_variant_ident");
    }

    #[traced_test]
    fn test_default_impl_with_first_variant_no_fields() {
        trace!("Starting test_default_impl_with_first_variant_no_fields");
        let enum_ident = syn::Ident::new("NoFieldConfidence", proc_macro2::Span::call_site());
        let variants = vec![
            // first variant has no fields
            quote! { EmptyVariant {} },
            // second variant with something
            quote! { NotEmptyVariant { x_conf: f32 } },
        ];

        // We pass the first variant ident as Some, but no fields in `first_variant_conf_fields`
        let first_variant = syn::Ident::new("EmptyVariant", proc_macro2::Span::call_site());
        let generated = build_enum_confidence(
            &enum_ident,
            &variants,
            Some(&first_variant),
            &[], // no fields
        );
        debug!("Generated TokenStream: {}", generated);

        let parsed: File = parse2(generated.clone()).expect("Could not parse generated code");
        let file_str = format!("{:?}", parsed);

        assert!(file_str.contains("impl Default for NoFieldConfidence"));
        // The default should choose `EmptyVariant { }`
        assert!(file_str.contains("NoFieldConfidence :: EmptyVariant { }"));
        info!("Finished test_default_impl_with_first_variant_no_fields");
    }

    #[traced_test]
    fn test_default_impl_with_fields_in_first_variant() {
        trace!("Starting test_default_impl_with_fields_in_first_variant");
        let enum_ident = syn::Ident::new("FieldfulConfidence", proc_macro2::Span::call_site());
        let variants = vec![
            quote! { FirstVar { alpha_conf: f32, gamma_conf: f32 } },
            quote! { SecondVar { beta_conf: f32 } },
        ];

        // We'll specify the first variant "FirstVar" and a few fields
        let first_variant = syn::Ident::new("FirstVar", proc_macro2::Span::call_site());
        let fields = vec!["alpha_conf".to_owned(), "gamma_conf".to_owned()];

        let generated = build_enum_confidence(&enum_ident, &variants, Some(&first_variant), &fields);
        debug!("Generated TokenStream: {}", generated);

        let parsed: File = parse2(generated.clone()).expect("Failed to parse tokens");
        let file_str = format!("{:?}", parsed);
        // Check that the Default impl is present
        assert!(file_str.contains("impl Default for FieldfulConfidence"));
        // Check that it references FirstVar
        assert!(file_str.contains("FieldfulConfidence :: FirstVar { alpha_conf"));
        assert!(file_str.contains("gamma_conf: ::core::default::Default::default()"));
        info!("Finished test_default_impl_with_fields_in_first_variant");
    }

    #[traced_test]
    fn test_multiple_variants_some_fields_in_first_none_in_second() {
        trace!("Starting test_multiple_variants_some_fields_in_first_none_in_second");
        let enum_ident = syn::Ident::new("MixedConfidence", proc_macro2::Span::call_site());
        let variants = vec![
            quote! { First { one_conf: f32, two_conf: f32 } },
            quote! { Second {} },
        ];
        let first_variant = syn::Ident::new("First", proc_macro2::Span::call_site());
        let fields = vec!["one_conf".to_owned(), "two_conf".to_owned()];

        let generated = build_enum_confidence(
            &enum_ident,
            &variants,
            Some(&first_variant),
            &fields
        );
        debug!("Generated TokenStream: {}", generated);

        let parsed: File = parse2(generated.clone()).expect("Failed to parse tokens");
        let file_str = format!("{:?}", parsed);
        // Ensure the second variant is present
        assert!(file_str.contains("Second"));
        // Ensure default picks the first
        assert!(file_str.contains("impl Default for MixedConfidence"));
        assert!(file_str.contains("MixedConfidence :: First"));
        info!("Finished test_multiple_variants_some_fields_in_first_none_in_second");
    }

    #[traced_test]
    fn test_first_variant_ident_is_none_but_fields_are_provided() {
        trace!("Starting test_first_variant_ident_is_none_but_fields_are_provided");
        // This scenario is weird: we pass fields in, but no first_variant_ident => no default impl
        let enum_ident = syn::Ident::new("OrphanConfidence", proc_macro2::Span::call_site());
        let variants = vec![
            quote! { VarA { a_conf: f32 } },
            quote! { VarB { b_conf: f32 } },
        ];
        let generated = build_enum_confidence(&enum_ident, &variants, None, &["a_conf".to_owned()]);
        debug!("Generated TokenStream: {}", generated);

        let parsed: File = parse2(generated.clone()).expect("Failed to parse tokens");
        let file_str = format!("{:?}", parsed);

        // No default impl since first_variant_ident is None
        assert!(file_str.contains("enum OrphanConfidence"));
        assert!(!file_str.contains("impl Default for OrphanConfidence"));
        info!("Finished test_first_variant_ident_is_none_but_fields_are_provided");
    }
}
