// ---------------- [ File: ai-json-template-derive/src/build_enum_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_enum_justification(
    enum_just_ident: &syn::Ident,
    just_variants: &[proc_macro2::TokenStream],
    first_variant_ident: Option<&syn::Ident>,
    first_variant_just_fields: &[String]
) -> proc_macro2::TokenStream
{
    debug!(
        "Building justification enum '{}' with {} variant(s)",
        enum_just_ident,
        just_variants.len()
    );

    let variants_ts = quote::quote! { #( #just_variants ),* };
    let default_impl = if let Some(first_variant) = first_variant_ident {
        let init_fields: Vec<_> = first_variant_just_fields.iter().map(|f_str| {
            let f_id = syn::Ident::new(f_str, proc_macro2::Span::call_site());
            quote::quote! { #f_id: ::core::default::Default::default() }
        }).collect();

        quote::quote! {
            impl ::core::default::Default for #enum_just_ident {
                fn default() -> Self {
                    #enum_just_ident::#first_variant { #( #init_fields ),* }
                }
            }
        }
    } else {
        quote::quote!()
    };

    quote::quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        enum #enum_just_ident {
            #variants_ts
        }
        #default_impl
    }
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
            quote! { VarA },
            quote! { VarB { some_field: String } },
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
        let ast = parse2::<syn::File>(output_ts.clone()).expect("Failed to parse output TokenStream");
        let mut enum_count = 0;
        let mut impl_count = 0;

        let mut found_default_impl = false;
        for item in ast.items {
            match item {
                syn::Item::Enum(item_enum) => {
                    enum_count += 1;
                    assert_eq!(item_enum.ident, "EnumJustWithDefault");
                    debug!("We expect two variants here: 'VarA' and 'VarB'");
                    assert_eq!(item_enum.variants.len(), 2);
                }
                syn::Item::Impl(ItemImpl {
                    self_ty: box syn::Type::Path(type_path),
                    trait_: None,
                    ..
                }) => {
                    // This is a plain impl block => likely 'impl Default for EnumJustWithDefault'
                    impl_count += 1;
                    debug!("Checking that this impl is for 'EnumJustWithDefault'...");
                    assert_eq!(type_path.path.segments.last().unwrap().ident, "EnumJustWithDefault");
                    found_default_impl = true;
                }
                _ => {}
            }
        }

        assert_eq!(enum_count, 1, "Expected exactly one enum definition");
        assert_eq!(impl_count, 1, "Expected one default impl for 'EnumJustWithDefault'");
        assert!(found_default_impl, "Did not find an impl block for Default in the parsed tokens");

        info!("test_with_first_variant_but_no_fields succeeded.");
    }

    #[traced_test]
    fn test_with_first_variant_and_fields() {
        trace!("Starting test: test_with_first_variant_and_fields");
        let enum_ident = syn::Ident::new("EnumJustFieldsDefault", proc_macro2::Span::call_site());
        let just_variants = vec![
            quote! { VariantOne { alpha: String, beta: f32 } },
            quote! { VariantTwo { gamma: bool } },
        ];
        let first_variant_ident = Some(syn::Ident::new("VariantOne", proc_macro2::Span::call_site()));
        let first_variant_just_fields = vec![ "alpha".to_string(), "beta".to_string() ];

        debug!("Invoking build_enum_justification with a recognized first variant that has fields 'alpha' and 'beta'...");
        let output_ts = build_enum_justification(
            &enum_ident,
            &just_variants,
            first_variant_ident.as_ref(),
            &first_variant_just_fields
        );

        info!("Attempting to parse resulting TokenStream as an enum definition plus default impl...");
        let ast = parse2::<syn::File>(output_ts.clone()).expect("Failed to parse output TokenStream");
        let mut enum_count = 0;
        let mut impl_count = 0;

        let mut found_impl_for_this_enum = false;
        for item in ast.items {
            match item {
                syn::Item::Enum(item_enum) => {
                    enum_count += 1;
                    assert_eq!(item_enum.ident, "EnumJustFieldsDefault");
                    debug!("We expect two variants: 'VariantOne' and 'VariantTwo'");
                    assert_eq!(item_enum.variants.len(), 2, "Wrong number of variants in the generated enum!");
                }
                syn::Item::Impl(ItemImpl {
                    self_ty: box syn::Type::Path(type_path),
                    trait_: None,
                    ..
                }) => {
                    impl_count += 1;
                    debug!("Checking that this impl is for 'EnumJustFieldsDefault'...");
                    assert_eq!(type_path.path.segments.last().unwrap().ident, "EnumJustFieldsDefault");
                    found_impl_for_this_enum = true;
                }
                _ => {}
            }
        }

        assert_eq!(enum_count, 1, "Expected exactly one enum definition in the output");
        assert_eq!(impl_count, 1, "Expected exactly one default impl in the output");
        assert!(found_impl_for_this_enum, "Expected the default impl to be for EnumJustFieldsDefault, but didn't see it");
        info!("test_with_first_variant_and_fields succeeded.");
    }

    #[traced_test]
    fn test_default_impl_content_for_first_variant_fields() {
        trace!("Starting test: test_default_impl_content_for_first_variant_fields");
        let enum_ident = syn::Ident::new("EnumJustCheckInit", proc_macro2::Span::call_site());
        let just_variants = vec![
            quote! { VarX { alpha: String, beta: f32 } },
            quote! { VarY {} },
        ];
        let first_variant_ident = Some(syn::Ident::new("VarX", proc_macro2::Span::call_site()));
        let first_variant_just_fields = vec![ "alpha".to_string(), "beta".to_string() ];

        debug!("Invoking build_enum_justification with first variant 'VarX {{ alpha, beta }}'...");
        let output_ts = build_enum_justification(
            &enum_ident,
            &just_variants,
            first_variant_ident.as_ref(),
            &first_variant_just_fields
        );

        info!("Parsing the output to find the 'impl Default for EnumJustCheckInit' block...");
        let ast = parse2::<syn::File>(output_ts.clone()).expect("Failed to parse output TokenStream");
        let mut found_impl_block = false;
        let mut found_correct_init = false;

        for item in ast.items {
            if let syn::Item::Impl(item_impl) = item {
                // we are expecting: impl Default for EnumJustCheckInit { fn default() -> Self { EnumJustCheckInit::VarX { alpha: ..., beta: ... } } }
                if let Some((_, trait_path, _)) = &item_impl.trait_ {
                    // This means it's "impl <Trait> for Type", but we want no trait -> skip
                    continue;
                }
                if let syn::Type::Path(tp) = &*item_impl.self_ty {
                    if tp.path.segments.last().map(|s| s.ident.to_string()) == Some("EnumJustCheckInit".to_string()) {
                        found_impl_block = true;
                        // Now let's search the block for the function default
                        for item in &item_impl.items {
                            if let syn::ImplItem::Fn(method) = item {
                                if method.sig.ident == "default" {
                                    debug!("Found 'default' function in impl block. Checking if it references 'VarX { alpha: ::core::default::Default::default(), beta: ...}'");
                                    // We can do a simplistic string check
                                    let method_str = quote! { #method }.to_string();
                                    if method_str.contains("VarX") &&
                                       method_str.contains("alpha: ::core::default::Default::default()") &&
                                       method_str.contains("beta: ::core::default::Default::default()") {
                                        found_correct_init = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        assert!(found_impl_block, "Expected an impl Default block for 'EnumJustCheckInit' but did not see one");
        assert!(found_correct_init, "Did not find correct default init referencing 'alpha' and 'beta' fields set to ::core::default::Default::default()");
        info!("test_default_impl_content_for_first_variant_fields succeeded.");
    }
}
