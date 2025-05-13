// ---------------- [ File: ai-json-template-derive/src/finalize_from_arm_unnamed_variant_ts.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn finalize_from_arm_unnamed_variant_ts(
    parent_enum_ident:   &syn::Ident,
    variant_ident:       &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident:    &syn::Ident,
    expansions:          &UnnamedVariantExpansion
) -> TokenStream2
{
    trace!(
        "Constructing from-arm expansions for unnamed variant '{}::{}'",
        parent_enum_ident,
        variant_ident
    );

    let flat_parent_ident = syn::Ident::new(
        &format!("FlatJustified{}", parent_enum_ident),
        parent_enum_ident.span()
    );

    // Possibly rename "Unit" => "UnitVariant"
    let raw_vname = variant_ident.to_string();
    let renamed_just_var = if raw_vname == "Unit" {
        syn::Ident::new("UnitVariant", variant_ident.span())
    } else {
        variant_ident.clone()
    };

    let item_exprs   = expansions.item_exprs();
    let just_vals    = expansions.just_vals();
    let conf_vals    = expansions.conf_vals();
    let pattern_vars = expansions.pattern_vars();

    // Construct the final item expression
    let item_ctor = if !item_exprs.is_empty() {
        quote! {
            #parent_enum_ident :: #variant_ident(#(#item_exprs),*)
        }
    } else {
        quote! {
            #parent_enum_ident :: #variant_ident()
        }
    };

    // Justification constructor
    let just_ctor = if !just_vals.is_empty() {
        quote! {
            #justification_ident :: #renamed_just_var {
                #(#just_vals),*
            }
        }
    } else {
        quote! {
            #justification_ident :: #renamed_just_var {}
        }
    };

    // Confidence constructor
    let conf_ctor = if !conf_vals.is_empty() {
        quote! {
            #confidence_ident :: #renamed_just_var {
                #(#conf_vals),*
            }
        }
    } else {
        quote! {
            #confidence_ident :: #renamed_just_var {}
        }
    };

    // Build the match arm pattern
    if !pattern_vars.is_empty() {
        quote! {
            #flat_parent_ident :: #variant_ident { #(#pattern_vars),* } => {
                Self {
                    item: #item_ctor,
                    justification: #just_ctor,
                    confidence: #conf_ctor,
                }
            }
        }
    } else {
        quote! {
            #flat_parent_ident :: #variant_ident {} => {
                Self {
                    item: #parent_enum_ident :: #variant_ident(),
                    justification: #justification_ident :: #renamed_just_var {},
                    confidence: #confidence_ident :: #renamed_just_var {},
                }
            }
        }
    }
}

#[cfg(test)]
mod test_finalize_from_arm_unnamed_variant_ts {
    use super::*;

    #[traced_test]
    fn test_empty_expansion() {
        trace!("Testing the scenario where there are no pattern vars, no item expressions, and no justification/confidence values.");
        let parent_enum_ident = Ident::new("FakeEnum", proc_macro2::Span::call_site());
        let variant_ident = Ident::new("EmptyVar", proc_macro2::Span::call_site());
        let justification_ident = Ident::new("FakeJust", proc_macro2::Span::call_site());
        let confidence_ident = Ident::new("FakeConf", proc_macro2::Span::call_site());

        let expansions = UnnamedVariantExpansionBuilder::default()
            .field_declarations(vec![])
            .pattern_vars(vec![])
            .item_exprs(vec![])
            .just_vals(vec![])
            .conf_vals(vec![])
            .build()
            .expect("Failed building UnnamedVariantExpansion for empty scenario");

        let tokens = finalize_from_arm_unnamed_variant_ts(
            &parent_enum_ident,
            &variant_ident,
            &justification_ident,
            &confidence_ident,
            &expansions
        );
        debug!("Generated tokens: {}", tokens);

        let parsed = parse2::<syn::Item>(tokens.clone());
        assert!(parsed.is_ok(), "Expected valid Rust item syntax.");
        let tokens_str = tokens.to_string();
        debug!("As string: {}", tokens_str);

        // We expect a minimal match arm with an empty block for the variant destructuring
        assert!(tokens_str.contains("FakeEnum :: EmptyVar {} => {"));
        assert!(tokens_str.contains("FakeEnum :: EmptyVar()"));
        trace!("test_empty_expansion passed.");
    }

    #[traced_test]
    fn test_no_just_conf_with_item_exprs() {
        trace!("Testing scenario with item expressions but no top-level justification/conf expansions.");
        let parent_enum_ident = Ident::new("AnotherEnum", proc_macro2::Span::call_site());
        let variant_ident = Ident::new("TupleVar", proc_macro2::Span::call_site());
        let justification_ident = Ident::new("AnotherJust", proc_macro2::Span::call_site());
        let confidence_ident = Ident::new("AnotherConf", proc_macro2::Span::call_site());

        // Suppose there's one field declared => the user sets an item expression but no just/conf
        let expansions = UnnamedVariantExpansionBuilder::default()
            .field_declarations(vec![
                quote! { #[serde(default)] f0: i32, },
            ])
            .pattern_vars(vec![
                quote! { f0 },
            ])
            .item_exprs(vec![
                quote! { f0 },
            ])
            .just_vals(vec![])
            .conf_vals(vec![])
            .build()
            .expect("Failed building expansions for single field scenario");

        let tokens = finalize_from_arm_unnamed_variant_ts(
            &parent_enum_ident,
            &variant_ident,
            &justification_ident,
            &confidence_ident,
            &expansions
        );
        debug!("Generated tokens: {}", tokens);

        let parsed = parse2::<syn::Item>(tokens.clone());
        assert!(parsed.is_ok(), "Expected valid Rust item syntax.");
        let tokens_str = tokens.to_string();
        debug!("As string: {}", tokens_str);

        // We expect the pattern to capture f0, then create AnotherEnum::TupleVar(f0)
        assert!(tokens_str.contains("AnotherEnum :: TupleVar(#f0)"));
        // Because top-level just/conf is missing
        assert!(!tokens_str.contains("variant_justification"));
        assert!(!tokens_str.contains("variant_confidence"));
        trace!("test_no_just_conf_with_item_exprs passed.");
    }

    #[traced_test]
    fn test_top_level_just_conf_no_fields() {
        trace!("Testing scenario with no field declarations, but with top-level justification/conf expansions present.");
        let parent_enum_ident = Ident::new("TopEnum", proc_macro2::Span::call_site());
        let variant_ident = Ident::new("NoFieldsVar", proc_macro2::Span::call_site());
        let justification_ident = Ident::new("TopJust", proc_macro2::Span::call_site());
        let confidence_ident = Ident::new("TopConf", proc_macro2::Span::call_site());

        // top-level fields => variant_justification, variant_confidence
        let expansions = UnnamedVariantExpansionBuilder::default()
            .field_declarations(vec![
                quote! { #[serde(default)] enum_variant_justification: String },
                quote! { #[serde(default)] enum_variant_confidence: f32 },
            ])
            .pattern_vars(vec![
                quote! { enum_variant_justification },
                quote! { enum_variant_confidence },
            ])
            .item_exprs(vec![])
            .just_vals(vec![
                quote! { variant_justification: enum_variant_justification },
            ])
            .conf_vals(vec![
                quote! { variant_confidence: enum_variant_confidence },
            ])
            .build()
            .expect("Failed building expansions for top-level justification/conf no fields scenario");

        let tokens = finalize_from_arm_unnamed_variant_ts(
            &parent_enum_ident,
            &variant_ident,
            &justification_ident,
            &confidence_ident,
            &expansions
        );
        debug!("Generated tokens: {}", tokens);

        let parsed = parse2::<syn::Item>(tokens.clone());
        assert!(parsed.is_ok(), "Expected valid Rust item syntax.");
        let tokens_str = tokens.to_string();
        debug!("As string: {}", tokens_str);

        // Expect presence of match arm pattern capturing enum_variant_justification, enum_variant_confidence
        assert!(tokens_str.contains("TopEnum :: NoFieldsVar { enum_variant_justification, enum_variant_confidence } => {"));
        // Expect top-level justification/conf usage
        assert!(tokens_str.contains("variant_justification: enum_variant_justification"));
        assert!(tokens_str.contains("variant_confidence: enum_variant_confidence"));
        trace!("test_top_level_just_conf_no_fields passed.");
    }

    #[traced_test]
    fn test_multiple_fields_and_just_conf() {
        trace!("Testing scenario with multiple fields + top-level justification/conf expansions.");
        let parent_enum_ident = Ident::new("MultiEnum", proc_macro2::Span::call_site());
        let variant_ident = Ident::new("ComplexVar", proc_macro2::Span::call_site());
        let justification_ident = Ident::new("MultiJust", proc_macro2::Span::call_site());
        let confidence_ident = Ident::new("MultiConf", proc_macro2::Span::call_site());

        // We'll create 2 unnamed fields + top-level justification/conf
        let expansions = UnnamedVariantExpansionBuilder::default()
            .field_declarations(vec![
                // top-level
                quote! { #[serde(default)] enum_variant_justification:String },
                quote! { #[serde(default)] enum_variant_confidence:f32 },
                // field 1
                quote! { #[serde(default)] f0: i32, },
                quote! { #[serde(default)] f0_justification:String, },
                quote! { #[serde(default)] f0_confidence:f32, },
                // field 2
                quote! { #[serde(default)] f1: String, },
                quote! { #[serde(default)] f1_justification:String, },
                quote! { #[serde(default)] f1_confidence:f32, },
            ])
            .pattern_vars(vec![
                quote! { enum_variant_justification },
                quote! { enum_variant_confidence },
                quote! { f0 },
                quote! { f0_justification },
                quote! { f0_confidence },
                quote! { f1 },
                quote! { f1_justification },
                quote! { f1_confidence },
            ])
            .item_exprs(vec![
                quote! { f0 },
                quote! { f1 },
            ])
            .just_vals(vec![
                quote! { variant_justification: enum_variant_justification },
                quote! { f0_justification: f0_justification },
                quote! { f1_justification: f1_justification },
            ])
            .conf_vals(vec![
                quote! { variant_confidence: enum_variant_confidence },
                quote! { f0_confidence: f0_confidence },
                quote! { f1_confidence: f1_confidence },
            ])
            .build()
            .expect("Failed building expansions for multiple fields + top-level just/conf");

        let tokens = finalize_from_arm_unnamed_variant_ts(
            &parent_enum_ident,
            &variant_ident,
            &justification_ident,
            &confidence_ident,
            &expansions
        );
        debug!("Generated tokens: {}", tokens);

        let parsed = parse2::<syn::Item>(tokens.clone());
        assert!(parsed.is_ok(), "Expected valid Rust item syntax.");
        let tokens_str = tokens.to_string();
        debug!("As string: {}", tokens_str);

        // Check presence of the pattern capturing top-level justification/conf and both fields plus their just/conf
        assert!(tokens_str.contains("MultiEnum :: ComplexVar { enum_variant_justification, enum_variant_confidence, f0, f0_justification, f0_confidence, f1, f1_justification, f1_confidence } =>"));
        // Check that item includes MultiEnum::ComplexVar(f0, f1)
        assert!(tokens_str.contains("item: MultiEnum :: ComplexVar("));
        // Check justification/conf expansions
        assert!(tokens_str.contains("variant_justification: enum_variant_justification"));
        assert!(tokens_str.contains("f0_justification: f0_justification"));
        assert!(tokens_str.contains("f1_justification: f1_justification"));
        assert!(tokens_str.contains("variant_confidence: enum_variant_confidence"));
        assert!(tokens_str.contains("f0_confidence: f0_confidence"));
        assert!(tokens_str.contains("f1_confidence: f1_confidence"));

        trace!("test_multiple_fields_and_just_conf passed.");
    }
}
