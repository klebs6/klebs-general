// ---------------- [ File: ai-json-template-derive/src/collect_variant_fields_for_just_conf.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn collect_variant_fields_for_just_conf(
    data_enum: &syn::DataEnum,
    parent_ident: &syn::Ident,
    span: proc_macro2::Span,
    enum_just_ident: &syn::Ident,
    enum_conf_ident: &syn::Ident,
) -> (
    Vec<proc_macro2::TokenStream>, // just_variants
    Vec<proc_macro2::TokenStream>, // conf_variants
    Option<syn::Ident>,            // first_variant_ident
    Vec<String>,                   // first_variant_just_fields
    Vec<String>                    // first_variant_conf_fields
)
{
    debug!("Collecting variant fields for justification/conf in '{}'", parent_ident);

    let mut first_variant_ident       = None;
    let mut first_variant_just_fields = Vec::<String>::new();
    let mut first_variant_conf_fields = Vec::<String>::new();

    let mut just_variants = Vec::new();
    let mut conf_variants = Vec::new();

    for (i, variant) in data_enum.variants.iter().enumerate() {
        let var_ident = &variant.ident;
        if i == 0 {
            first_variant_ident = Some(var_ident.clone());
        }

        let skip_self_just = is_justification_disabled_for_variant(variant);
        trace!(
            "Variant '{}' -> skip_self_just={}",
            var_ident,
            skip_self_just
        );

        match &variant.fields {
            syn::Fields::Unit => {
                let (jvar, cvar, maybe_j, maybe_c) =
                    handle_unit_variant(var_ident, skip_self_just);
                just_variants.push(jvar);
                conf_variants.push(cvar);

                if i == 0 {
                    if let Some(j) = maybe_j { first_variant_just_fields.push(j); }
                    if let Some(c) = maybe_c { first_variant_conf_fields.push(c); }
                }
            }

            syn::Fields::Named(named_fields) => {
                let (jvar, cvar, jfields, cfields) =
                    handle_named_variant(var_ident, named_fields, skip_self_just, i == 0);
                just_variants.push(jvar);
                conf_variants.push(cvar);

                if i == 0 {
                    first_variant_just_fields.extend(jfields);
                    first_variant_conf_fields.extend(cfields);
                }
            }

            syn::Fields::Unnamed(unnamed_fields) => {
                let (jvar, cvar, jfields, cfields) =
                    handle_unnamed_variant(var_ident, unnamed_fields, skip_self_just, i == 0);
                just_variants.push(jvar);
                conf_variants.push(cvar);

                if i == 0 {
                    first_variant_just_fields.extend(jfields);
                    first_variant_conf_fields.extend(cfields);
                }
            }
        }
    }

    (
        just_variants,
        conf_variants,
        first_variant_ident,
        first_variant_just_fields,
        first_variant_conf_fields
    )
}

#[cfg(test)]
mod exhaustive_test_collect_variant_fields_for_just_conf {
    use super::*;

    #[traced_test]
    fn it_handles_empty_enum() {
        trace!("Starting test: it_handles_empty_enum");

        // Parse an empty enum
        let parsed: ItemEnum = syn::parse2(quote! {
            enum EmptyEnum {}
        }).expect("Failed to parse empty enum");

        // Synthesize a DataEnum from the parsed item
        let data_enum = DataEnum {
            enum_token: parsed.enum_token,
            brace_token: parsed.brace_token,
            variants: parsed.variants,
        };

        let span = proc_macro2::Span::call_site();
        let parent_ident = syn::Ident::new("ParentEnum", span);
        let enum_just_ident = syn::Ident::new("EmptyJust", span);
        let enum_conf_ident = syn::Ident::new("EmptyConf", span);

        let (just_variants, conf_variants, first_variant_ident, first_just_fields, first_conf_fields)
            = collect_variant_fields_for_just_conf(&data_enum, &parent_ident, span, &enum_just_ident, &enum_conf_ident);

        debug!("just_variants len: {}", just_variants.len());
        debug!("conf_variants len: {}", conf_variants.len());
        debug!("first_variant_ident: {:?}", first_variant_ident);
        debug!("first_variant_just_fields: {:?}", first_just_fields);
        debug!("first_variant_conf_fields: {:?}", first_conf_fields);

        assert_eq!(just_variants.len(), 0, "Expected 0 justification variants for an empty enum");
        assert_eq!(conf_variants.len(), 0, "Expected 0 confidence variants for an empty enum");
        assert_eq!(first_variant_ident, None, "No first variant for empty enum");
        assert!(first_just_fields.is_empty(), "No fields for empty enum");
        assert!(first_conf_fields.is_empty(), "No fields for empty enum");

        info!("Finished test: it_handles_empty_enum");
    }

    #[traced_test]
    fn it_handles_single_unit_variant() {
        trace!("Starting test: it_handles_single_unit_variant");

        let parsed: ItemEnum = syn::parse2(quote! {
            enum SingleVariantEnum {
                Unit
            }
        }).expect("Failed to parse single-variant enum");

        let data_enum = DataEnum {
            enum_token: parsed.enum_token,
            brace_token: parsed.brace_token,
            variants: parsed.variants,
        };

        let span = proc_macro2::Span::call_site();
        let parent_ident = syn::Ident::new("ParentEnum", span);
        let enum_just_ident = syn::Ident::new("SingleJust", span);
        let enum_conf_ident = syn::Ident::new("SingleConf", span);

        let (just_variants, conf_variants, first_variant_ident, first_just_fields, first_conf_fields)
            = collect_variant_fields_for_just_conf(&data_enum, &parent_ident, span, &enum_just_ident, &enum_conf_ident);

        debug!("just_variants: {:?}", just_variants);
        debug!("conf_variants: {:?}", conf_variants);
        debug!("first_variant_ident: {:?}", first_variant_ident);
        debug!("first_just_fields: {:?}", first_just_fields);
        debug!("first_conf_fields: {:?}", first_conf_fields);

        assert_eq!(just_variants.len(), 1, "Should have exactly 1 justification variant");
        assert_eq!(conf_variants.len(), 1, "Should have exactly 1 confidence variant");
        assert_eq!(first_variant_ident.as_ref().unwrap().to_string(), "Unit", "First variant should be 'Unit'");
        // If we don't skip justification, we expect "variant_justification" and "variant_confidence" 
        // unless an attribute turned them off. By default we expect them to exist for the first variant.
        assert_eq!(first_just_fields, vec!["variant_justification"], "Expected top-level justification field for unit variant");
        assert_eq!(first_conf_fields, vec!["variant_confidence"], "Expected top-level confidence field for unit variant");

        info!("Finished test: it_handles_single_unit_variant");
    }

    #[traced_test]
    fn it_handles_multiple_named_variants_with_justify_attributes() {
        trace!("Starting test: it_handles_multiple_named_variants_with_justify_attributes");

        // This enum has two variants: 
        // - One with no skip attributes
        // - Another with #[justify=false] and a named field with/without justification
        let parsed: ItemEnum = syn::parse2(quote! {
            enum MultiVariantEnum {
                /// A named variant with justification
                Alpha {
                    x: i32,
                    #[justify=false]
                    y: String
                },

                /// A named variant with skip for the variant itself
                #[justify=false]
                Beta {
                    z: bool
                }
            }
        }).expect("Failed to parse multiple named-variants enum");

        let data_enum = DataEnum {
            enum_token: parsed.enum_token,
            brace_token: parsed.brace_token,
            variants: parsed.variants,
        };

        let span = proc_macro2::Span::call_site();
        let parent_ident = syn::Ident::new("ParentEnum", span);
        let enum_just_ident = syn::Ident::new("MultiJust", span);
        let enum_conf_ident = syn::Ident::new("MultiConf", span);

        let (just_variants, conf_variants, first_variant_ident, first_just_fields, first_conf_fields)
            = collect_variant_fields_for_just_conf(&data_enum, &parent_ident, span, &enum_just_ident, &enum_conf_ident);

        debug!("just_variants: {:?}", just_variants);
        debug!("conf_variants: {:?}", conf_variants);
        debug!("first_variant_ident: {:?}", first_variant_ident);
        debug!("first_just_fields: {:?}", first_just_fields);
        debug!("first_conf_fields: {:?}", first_conf_fields);

        // We have two variants: 'Alpha' (no skip_self) and 'Beta' (skip_self).
        // So for justification, 'Alpha' has top-level variant_justification, plus x_justification (since x is i32).
        // But y is skip justification => no y_justification. 
        // 'Beta' is skip_self => no top-level fields (variant_justification) 
        // but it does have a field 'z' => we check if it's justify-enabled by default => yes => so it has z_justification ?

        assert_eq!(just_variants.len(), 2, "Expected 2 justification variants (Alpha and Beta)");
        assert_eq!(conf_variants.len(), 2, "Expected 2 confidence variants (Alpha and Beta)");
        assert_eq!(first_variant_ident.as_ref().unwrap().to_string(), "Alpha", "First variant is 'Alpha'");
        
        // The first variant's justification fields should contain 'variant_justification' and 'x_justification' 
        // but not 'y_justification' since y has #[justify=false].
        assert_eq!(
            first_just_fields,
            vec!["variant_justification", "x_justification"],
            "Alpha should have top-level justification plus x_justification"
        );
        assert_eq!(
            first_conf_fields,
            vec!["variant_confidence", "x_confidence"],
            "Alpha should have top-level confidence plus x_confidence"
        );

        info!("Finished test: it_handles_multiple_named_variants_with_justify_attributes");
    }

    #[traced_test]
    fn it_handles_unnamed_variants_and_first_variant_logic() {
        trace!("Starting test: it_handles_unnamed_variants_and_first_variant_logic");

        // Four variants:
        // - First = Unnamed with no skip
        // - Second = Unnamed with skip_self
        // - Third = Unit with skip_self
        // - Fourth = Unit with no skip

        let parsed: ItemEnum = syn::parse2(quote! {
            enum MixedUnnamedEnum {
                One(i32, bool),
                #[justify=false]
                Two(String, i64),
                #[justify=false]
                Three,
                Four
            }
        }).expect("Failed to parse unnamed variants enum");

        let data_enum = DataEnum {
            enum_token: parsed.enum_token,
            brace_token: parsed.brace_token,
            variants: parsed.variants,
        };

        let span = proc_macro2::Span::call_site();
        let parent_ident = syn::Ident::new("ParentEnum", span);
        let enum_just_ident = syn::Ident::new("MixedJust", span);
        let enum_conf_ident = syn::Ident::new("MixedConf", span);

        let (just_variants, conf_variants, first_variant_ident, first_just_fields, first_conf_fields)
            = collect_variant_fields_for_just_conf(&data_enum, &parent_ident, span, &enum_just_ident, &enum_conf_ident);

        debug!("just_variants: {:?}", just_variants);
        debug!("conf_variants: {:?}", conf_variants);
        debug!("first_variant_ident: {:?}", first_variant_ident);
        debug!("first_just_fields: {:?}", first_just_fields);
        debug!("first_conf_fields: {:?}", first_conf_fields);

        assert_eq!(just_variants.len(), 4, "We expect 4 variants in the Justification enum");
        assert_eq!(conf_variants.len(), 4, "We expect 4 variants in the Confidence enum");
        assert_eq!(first_variant_ident.as_ref().unwrap().to_string(), "One", "First variant should be 'One'");

        // For the first variant "One", we have top-level variant_justification + field_0_justification + field_1_justification
        // because there's no skip on the variant or fields.
        assert_eq!(
            first_just_fields,
            vec!["variant_justification", "field_0_justification", "field_1_justification"],
            "Unnamed variant 'One' should have top-level justification plus two fields"
        );
        assert_eq!(
            first_conf_fields,
            vec!["variant_confidence", "field_0_confidence", "field_1_confidence"],
            "Unnamed variant 'One' should have top-level confidence plus two fields"
        );

        // Summaries for second variant "Two": skip_self => no variant_justification, 
        // but each field is presumably justification-enabled by default if there's no `#[justify=false]` on them individually 
        // (we have none specified, but let's not assert in detail here; 
        // we only do the top-level coverage for the first variant in the function's return).

        // Summaries for third variant "Three": skip_self => it's a Unit variant => so no justification fields at all.

        // Summaries for fourth variant "Four": a normal unit variant => top-level justification fields exist if not skip_self.

        info!("Finished test: it_handles_unnamed_variants_and_first_variant_logic");
    }

    #[traced_test]
    fn it_handles_complex_attributes_on_first_variant() {
        trace!("Starting test: it_handles_complex_attributes_on_first_variant");
        
        // The very first variant is heavily annotated with skip 
        // to see if we do or don't gather them as "first variant."
        let parsed: ItemEnum = syn::parse2(quote! {
            enum ComplexEnum {
                #[justify=false]
                /// A doc comment
                SkippedNamed {
                    alpha: i32,
                    beta: bool
                },
                SimpleUnit,
                AnotherUnit
            }
        }).expect("Failed to parse complex attribute enum");

        let data_enum = DataEnum {
            enum_token: parsed.enum_token,
            brace_token: parsed.brace_token,
            variants: parsed.variants,
        };

        let span = proc_macro2::Span::call_site();
        let parent_ident = syn::Ident::new("ParentEnum", span);
        let enum_just_ident = syn::Ident::new("ComplexJust", span);
        let enum_conf_ident = syn::Ident::new("ComplexConf", span);

        let (just_variants, conf_variants, first_variant_ident, first_just_fields, first_conf_fields)
            = collect_variant_fields_for_just_conf(&data_enum, &parent_ident, span, &enum_just_ident, &enum_conf_ident);

        debug!("just_variants: {:?}", just_variants);
        debug!("conf_variants: {:?}", conf_variants);
        debug!("first_variant_ident: {:?}", first_variant_ident);
        debug!("first_just_fields: {:?}", first_just_fields);
        debug!("first_conf_fields: {:?}", first_conf_fields);

        // We do have 3 variants. The first encountered variant is "SkippedNamed",
        // but it is skip_self => there's no top-level justification. 
        // The code picks that as "first_variant_ident" anyway (the function doesn't skip it in the sense of ignoring it),
        // so we expect first_variant_ident = Some("SkippedNamed").
        // But the top-level fields from "SkippedNamed" might be minimal or none if skip_self is true.

        assert_eq!(
            just_variants.len(), 
            3, 
            "Expected 3 justification variants total"
        );
        assert_eq!(
            conf_variants.len(), 
            3, 
            "Expected 3 confidence variants total"
        );
        assert_eq!(
            first_variant_ident.as_ref().unwrap().to_string(), 
            "SkippedNamed", 
            "The function picks the literal first variant even if skip_self is true"
        );
        // Because skip_self is true, we expect zero top-level justification or confidence fields for the first variant
        // but it might or might not gather fields if they are not individually skip_just. 
        // However, the function picks the top-level fields for the first variant's 
        // "first_variant_just_fields" and "first_variant_conf_fields" 
        // only if skip_self is false. So we expect them empty here.
        assert!(
            first_just_fields.is_empty(),
            "Skipped variant => no top-level fields"
        );
        assert!(
            first_conf_fields.is_empty(),
            "Skipped variant => no top-level fields"
        );

        info!("Finished test: it_handles_complex_attributes_on_first_variant");
    }
}
