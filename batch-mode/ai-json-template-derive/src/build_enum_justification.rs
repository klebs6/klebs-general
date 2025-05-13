// ---------------- [ File: ai-json-template-derive/src/build_enum_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_enum_justification(
    enum_just_ident:           &syn::Ident,
    just_variants:             &[proc_macro2::TokenStream],
    first_variant_ident:       Option<&syn::Ident>,
    first_variant_just_fields: &[String]
) -> proc_macro2::TokenStream
{
    trace!(
        "ENTER build_enum_justification(enum_just_ident='{}', #just_variants={}, first_variant_ident={:?}, #first_variant_just_fields={})",
        enum_just_ident,
        just_variants.len(),
        first_variant_ident,
        first_variant_just_fields.len()
    );

    // Step 1: Prepare each variant's token stream
    trace!("Preparing tokens for each just_variants entry...");
    for (i, variant_ts) in just_variants.iter().enumerate() {
        debug!("  just_variants[{}]: {}", i, variant_ts);
    }
    let variants_ts = quote::quote! { #( #just_variants ),* };

    // Step 2: Possibly build a Default impl if we have a first variant
    let default_impl = if let Some(first_var_ident) = first_variant_ident {
        trace!(
            "We have first_variant_ident='{}'; building a Default impl that defaults to that variant",
            first_var_ident
        );

        // For each string in first_variant_just_fields, produce e.g. `field_name: Default::default()`
        let field_inits: Vec<_> = first_variant_just_fields
            .iter()
            .map(|field_name_str| {
                let f = syn::Ident::new(field_name_str, proc_macro2::Span::call_site());
                trace!("  Default initializer => '{}': Default::default()", field_name_str);
                quote::quote! {
                    #f: Default::default()
                }
            })
            .collect();

        let default_impl_ts = quote::quote! {
            impl Default for #enum_just_ident {
                fn default() -> Self {
                    #enum_just_ident::#first_var_ident {
                        #( #field_inits ),*
                    }
                }
            }
        };
        debug!("Generated Default impl snippet:\n{}", default_impl_ts);
        default_impl_ts
    } else {
        trace!("No first_variant_ident => skipping Default impl entirely.");
        quote::quote! {}
    };

    // Step 3: Combine the enum + (optionally) the impl as separate AST items
    // because the test suite expects them as separate items.
    let expanded = quote::quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        enum #enum_just_ident {
            #variants_ts
        }

        #default_impl
    };

    debug!(
        "Final expanded output for '{}':\n{}",
        enum_just_ident,
        expanded
    );
    trace!("EXIT build_enum_justification for '{}'", enum_just_ident);
    expanded
}

#[cfg(test)]
mod test_build_enum_justification_exhaustively {
    use super::*;

    #[traced_test]
    fn test_empty_variants_no_first_variant() {
        trace!("Starting test: test_empty_variants_no_first_variant");
        let enum_ident = syn::Ident::new("EmptyJustification", proc_macro2::Span::call_site());
        let just_variants: Vec<proc_macro2::TokenStream> = vec![];
        let first_variant_ident: Option<&syn::Ident> = None;
        let first_variant_just_fields: &[String] = &[];

        debug!("Invoking build_enum_justification with empty variant list and no first variant ident...");
        let output_ts = build_enum_justification(
            &enum_ident,
            &just_variants,
            first_variant_ident,
            first_variant_just_fields
        );

        info!("Attempting to parse resulting TokenStream as an enum definition...");
        let ast = parse2::<syn::File>(output_ts.clone()).expect("Failed to parse output TokenStream into a syn::File");

        // We should have exactly 1 enum item, with no variants, and no default impl
        let mut enum_count = 0;
        let mut impl_count = 0;
        for item in ast.items {
            match item {
                syn::Item::Enum(item_enum) => {
                    enum_count += 1;
                    // Check name
                    assert_eq!(item_enum.ident, "EmptyJustification");
                    // Check no variants
                    assert!(item_enum.variants.is_empty(), "Expected no variants, found some");
                    debug!("Found correct enum with zero variants.");
                }
                syn::Item::Impl(_) => {
                    impl_count += 1;
                }
                _ => {}
            }
        }
        assert_eq!(enum_count, 1, "Expected exactly one enum in output");
        assert_eq!(impl_count, 0, "Did not expect a Default impl when no first_variant_ident");
        info!("test_empty_variants_no_first_variant succeeded.");
    }

    #[traced_test]
    fn test_multiple_variants_no_first_variant() {
        trace!("Starting test: test_multiple_variants_no_first_variant");
        let enum_ident = syn::Ident::new("MultiJustNoDefault", proc_macro2::Span::call_site());
        let just_variants = vec![
            quote! { VarOne },
            quote! { VarTwo { field_a: String } },
        ];
        let first_variant_ident: Option<&syn::Ident> = None;
        let first_variant_just_fields: &[String] = &[];

        debug!("Invoking build_enum_justification with multiple variants and no first_variant_ident...");
        let output_ts = build_enum_justification(
            &enum_ident,
            &just_variants,
            first_variant_ident,
            first_variant_just_fields
        );

        info!("Attempting to parse resulting TokenStream as an enum definition...");
        let ast = parse2::<syn::File>(output_ts.clone()).expect("Failed to parse output TokenStream into a syn::File");

        let mut enum_count = 0;
        let mut impl_count = 0;
        for item in ast.items {
            match item {
                syn::Item::Enum(item_enum) => {
                    enum_count += 1;
                    assert_eq!(item_enum.ident, "MultiJustNoDefault");
                    debug!("Checking that we have exactly 2 variants in the enum...");
                    assert_eq!(item_enum.variants.len(), 2, "Expected exactly 2 variants in the output enum");
                    debug!("Found correct number of variants with no default impl variant.");
                }
                syn::Item::Impl(_) => {
                    impl_count += 1;
                }
                _ => {}
            }
        }
        assert_eq!(enum_count, 1, "Expected exactly one enum in output");
        assert_eq!(impl_count, 0, "Did not expect a Default impl for multi-variant w/o first_variant_ident");
        info!("test_multiple_variants_no_first_variant succeeded.");
    }

    #[traced_test]
    fn test_with_first_variant_but_no_fields() {
        trace!("Starting test: test_with_first_variant_but_no_fields");
        let enum_ident = syn::Ident::new("EnumJustWithDefault", proc_macro2::Span::call_site());
        let just_variants = vec![
            quote::quote! { VarA },
            quote::quote! { VarB { some_field: String } },
        ];
        let first_variant_ident = Some(syn::Ident::new("VarA", proc_macro2::Span::call_site()));
        let first_variant_just_fields: &[String] = &[];

        debug!("Invoking build_enum_justification with a recognized first variant that has no fields...");
        let output_ts = build_enum_justification(
            &enum_ident,
            &just_variants,
            first_variant_ident.as_ref(),
            first_variant_just_fields
        );

        info!("Attempting to parse resulting TokenStream as an enum definition plus default impl...");
        let ast = syn::parse2::<syn::File>(output_ts.clone())
            .expect("Failed to parse output TokenStream");

        let mut enum_count = 0;
        let mut impl_count = 0;

        for item in ast.items {
            match item {
                syn::Item::Enum(item_enum) => {
                    enum_count += 1;
                    assert_eq!(item_enum.ident, "EnumJustWithDefault");
                    debug!("We expect two variants here: 'VarA' and 'VarB'");
                    assert_eq!(
                        item_enum.variants.len(),
                        2,
                        "Expected exactly 2 variants in the output enum"
                    );
                }
                // FIX: remove `trait_: None` so we correctly match trait impls (like `impl Default for ...`)
                syn::Item::Impl(syn::ItemImpl {
                    self_ty: box syn::Type::Path(type_path),
                    ..
                }) => {
                    impl_count += 1;
                    debug!("Found impl block for {:?}", type_path.path.segments.last().unwrap().ident);
                }
                _ => {}
            }
        }

        assert_eq!(enum_count, 1, "Expected exactly one enum in output");
        assert_eq!(
            impl_count, 1,
            "Expected one default impl for 'EnumJustWithDefault'"
        );
        info!("test_with_first_variant_but_no_fields succeeded.");
    }

    #[traced_test]
    fn test_with_first_variant_and_fields() {
        trace!("Starting test: test_with_first_variant_and_fields");
        let enum_ident = syn::Ident::new("EnumJustFieldsDefault", proc_macro2::Span::call_site());
        let just_variants = vec![
            quote::quote! { VariantOne { alpha: String, beta: f32 } },
            quote::quote! { VariantTwo { gamma: bool } },
        ];
        let first_variant_ident = Some(syn::Ident::new("VariantOne", proc_macro2::Span::call_site()));
        let first_variant_just_fields = vec!["alpha".to_string(), "beta".to_string()];

        debug!("Invoking build_enum_justification with a recognized first variant that has fields 'alpha' and 'beta'...");
        let output_ts = build_enum_justification(
            &enum_ident,
            &just_variants,
            first_variant_ident.as_ref(),
            &first_variant_just_fields
        );

        info!("Attempting to parse resulting TokenStream as an enum definition plus default impl...");
        let ast = syn::parse2::<syn::File>(output_ts.clone())
            .expect("Failed to parse output TokenStream");

        let mut enum_count = 0;
        let mut impl_count = 0;

        for item in ast.items {
            match item {
                syn::Item::Enum(item_enum) => {
                    enum_count += 1;
                    assert_eq!(item_enum.ident, "EnumJustFieldsDefault");
                    debug!("We expect two variants: 'VariantOne' and 'VariantTwo'");
                    assert_eq!(
                        item_enum.variants.len(),
                        2,
                        "Wrong number of variants in the generated enum!"
                    );
                }
                // FIX: remove `trait_: None` so we correctly detect our trait impl
                syn::Item::Impl(syn::ItemImpl {
                    self_ty: box syn::Type::Path(type_path),
                    ..
                }) => {
                    impl_count += 1;
                    debug!(
                        "Found impl block for {:?}",
                        type_path.path.segments.last().unwrap().ident
                    );
                }
                _ => {}
            }
        }

        assert_eq!(enum_count, 1, "Expected exactly one enum definition in the output");
        assert_eq!(impl_count, 1, "Expected exactly one default impl in the output");
        info!("test_with_first_variant_and_fields succeeded.");
    }

    #[traced_test]
    fn test_default_impl_content_for_first_variant_fields() {
        trace!("Starting test: test_default_impl_content_for_first_variant_fields");
        let enum_ident = syn::Ident::new("EnumJustCheckInit", proc_macro2::Span::call_site());
        let just_variants = vec![
            quote::quote! { VarX { alpha: String, beta: f32 } },
            quote::quote! { VarY {} },
        ];
        let first_variant_ident = Some(syn::Ident::new("VarX", proc_macro2::Span::call_site()));
        let first_variant_just_fields = vec!["alpha".to_string(), "beta".to_string()];

        debug!("Invoking build_enum_justification with first variant 'VarX {{ alpha, beta }}'...");
        let output_ts = build_enum_justification(
            &enum_ident,
            &just_variants,
            first_variant_ident.as_ref(),
            &first_variant_just_fields
        );

        info!("Parsing the output to find the 'impl Default for EnumJustCheckInit' block...");
        let ast = syn::parse2::<syn::File>(output_ts.clone())
            .expect("Failed to parse output TokenStream");

        let mut found_impl_block = false;
        let mut found_correct_init = false;

        for item in ast.items {
            match item {
                syn::Item::Impl(item_impl) => {
                    found_impl_block = true;
                    // We expect something like `impl Default for EnumJustCheckInit`
                    // that references VarX { alpha: Default::default(), beta: Default::default() }
                    if let Some((_, path, _)) = &item_impl.trait_ {
                        if path.segments.last().map(|s| s.ident.to_string()) == Some("Default".to_string()) {
                            // Check if method text references the correct field defaults:
                            for impl_item in &item_impl.items {
                                if let syn::ImplItem::Fn(m) = impl_item {
                                    if m.sig.ident == "default" {
                                        let method_str = quote::quote!(#m).to_string();
                                        // minimal check:
                                        if method_str.contains("VarX")
                                            && method_str.contains("alpha : Default :: default ()")
                                            && method_str.contains("beta : Default :: default ()")
                                        {
                                            found_correct_init = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        assert!(
            found_impl_block,
            "Expected an impl Default block for 'EnumJustCheckInit' but did not see one"
        );
        assert!(
            found_correct_init,
            "Did not find correct default init referencing 'alpha' and 'beta' fields set to ::core::default::Default::default()"
        );
        info!("test_default_impl_content_for_first_variant_fields succeeded.");
    }
}
