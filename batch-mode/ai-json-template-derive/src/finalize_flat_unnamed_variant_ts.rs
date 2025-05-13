// ---------------- [ File: ai-json-template-derive/src/finalize_flat_unnamed_variant_ts.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn finalize_flat_unnamed_variant_ts(
    variant_ident: &syn::Ident,
    expansions: &UnnamedVariantExpansion
) -> TokenStream2
{
    trace!(
        "Constructing flattened unnamed variant definition for variant '{}'",
        variant_ident
    );

    let field_decls = expansions.field_declarations();

    if !field_decls.is_empty() {
        quote! {
            #variant_ident {
                #(#field_decls),*
            },
        }
    } else {
        quote! {
            #variant_ident {},
        }
    }
}

#[cfg(test)]
mod test_finalize_flat_unnamed_variant_ts {
    use super::*;

    #[traced_test]
    fn test_no_fields_no_top_level_justification_or_confidence() {
        trace!("Starting test_no_fields_no_top_level_justification_or_confidence");

        let variant_ident: Ident = parse_quote! { MyEmptyVariant };
        debug!("Created a variant_ident = {:?}", variant_ident);

        let expansions = UnnamedVariantExpansionBuilder::default()
            .field_declarations(vec![])
            .pattern_vars(vec![])
            .item_exprs(vec![])
            .just_vals(vec![])
            .conf_vals(vec![])
            .build()
            .expect("Failed to build UnnamedVariantExpansion");

        info!("Calling finalize_flat_unnamed_variant_ts with empty expansions");
        let generated = finalize_flat_unnamed_variant_ts(&variant_ident, &expansions);
        debug!("Received generated TokenStream: {}", generated.to_token_stream());

        // We verify the code compiles to something minimal (e.g. `MyEmptyVariant {},`).
        let as_string = generated.to_string();
        info!("Generated tokens as string => {}", as_string);
        assert!(
            as_string.contains("MyEmptyVariant { } ,"),
            "Expected an empty variant definition with braces."
        );

        trace!("test_no_fields_no_top_level_justification_or_confidence completed successfully");
    }

    #[traced_test]
    fn test_only_top_level_justification_confidence_no_fields() {
        trace!("Starting test_only_top_level_justification_confidence_no_fields");

        let variant_ident: Ident = parse_quote! { WithJustification };
        debug!("Created variant_ident = {:?}", variant_ident);

        // Emulate skip_self_just = false => we have top-level just/conf, but no fields
        let expansions = UnnamedVariantExpansionBuilder::default()
            .field_declarations(vec![
                parse_quote! { #[serde(default)] enum_variant_justification : String },
                parse_quote! { #[serde(default)] enum_variant_confidence : f32 },
            ])
            .pattern_vars(vec![
                parse_quote! { enum_variant_justification },
                parse_quote! { enum_variant_confidence },
            ])
            .item_exprs(vec![])
            .just_vals(vec![
                parse_quote! { variant_justification : enum_variant_justification },
            ])
            .conf_vals(vec![
                parse_quote! { variant_confidence : enum_variant_confidence },
            ])
            .build()
            .expect("Failed to build UnnamedVariantExpansion");

        info!("Calling finalize_flat_unnamed_variant_ts with only top-level justification/confidence");
        let generated = finalize_flat_unnamed_variant_ts(&variant_ident, &expansions);
        debug!("Received generated TokenStream: {}", generated.to_token_stream());

        // Check basic structure
        let as_string = generated.to_string();
        info!("Generated tokens => {}", as_string);

        assert!(
            as_string.contains("WithJustification { #[serde ( default )] enum_variant_justification : String , #[serde ( default )] enum_variant_confidence : f32 } ,"),
            "Expected to see the top-level justification/confidence fields in the variant definition"
        );
        assert!(
            as_string.contains("WithJustification { enum_variant_justification , enum_variant_confidence } =>"),
            "Expected to see the match arm pattern for top-level justification/confidence"
        );
        trace!("test_only_top_level_justification_confidence_no_fields completed successfully");
    }

    #[traced_test]
    fn test_multiple_fields_with_and_without_justification() {
        trace!("Starting test_multiple_fields_with_and_without_justification");

        let variant_ident: Ident = parse_quote! { ComplexTuple };
        debug!("Created variant_ident = {:?}", variant_ident);

        // Suppose we have 3 fields:
        //    f0: i32 (skip_self_just=false => includes justification/confidence)
        //    f1: String (skip_self_just=true => no justification/confidence)
        //    f2: f64 (skip_self_just=false => includes justification/confidence)
        // plus top-level justification is also enabled (skip_self_just=false)

        let expansions = UnnamedVariantExpansionBuilder::default()
            .field_declarations(vec![
                // top-level
                parse_quote! { #[serde(default)] enum_variant_justification : String },
                parse_quote! { #[serde(default)] enum_variant_confidence : f32 },

                // f0
                parse_quote! { #[serde(default)] f0 : i32 },
                parse_quote! { #[serde(default)] f0_justification : String },
                parse_quote! { #[serde(default)] f0_confidence : f32 },

                // f1
                parse_quote! { #[serde(default)] f1 : String },
                // skip self justification => no f1_justification, no f1_confidence

                // f2
                parse_quote! { #[serde(default)] f2 : f64 },
                parse_quote! { #[serde(default)] f2_justification : String },
                parse_quote! { #[serde(default)] f2_confidence : f32 },
            ])
            .pattern_vars(vec![
                // top-level
                parse_quote! { enum_variant_justification },
                parse_quote! { enum_variant_confidence },

                // f0
                parse_quote! { f0 },
                parse_quote! { f0_justification },
                parse_quote! { f0_confidence },

                // f1
                parse_quote! { f1 },
                // skip justification

                // f2
                parse_quote! { f2 },
                parse_quote! { f2_justification },
                parse_quote! { f2_confidence },
            ])
            .item_exprs(vec![
                parse_quote! { f0 },
                parse_quote! { f1 },
                parse_quote! { f2 },
            ])
            .just_vals(vec![
                parse_quote! { variant_justification : enum_variant_justification },
                parse_quote! { f0_justification : f0_justification },
                parse_quote! { f2_justification : f2_justification },
            ])
            .conf_vals(vec![
                parse_quote! { variant_confidence : enum_variant_confidence },
                parse_quote! { f0_confidence : f0_confidence },
                parse_quote! { f2_confidence : f2_confidence },
            ])
            .build()
            .expect("Failed to build UnnamedVariantExpansion");

        info!("Calling finalize_flat_unnamed_variant_ts for multiple fields scenario");
        let generated = finalize_flat_unnamed_variant_ts(&variant_ident, &expansions);
        debug!("Received generated TokenStream: {}", generated.to_token_stream());

        let as_string = generated.to_string();
        info!("Generated tokens => {}", as_string);

        // We check the presence of relevant fields and match arms:
        assert!(
            as_string.contains("ComplexTuple { #[serde ( default )] enum_variant_justification : String , #[serde ( default )] enum_variant_confidence : f32 , #[serde ( default )] f0 : i32 , #[serde ( default )] f0_justification : String , #[serde ( default )] f0_confidence : f32 , #[serde ( default )] f1 : String , #[serde ( default )] f2 : f64 , #[serde ( default )] f2_justification : String , #[serde ( default )] f2_confidence : f32 } ,"),
            "Expected all field declarations, including top-level and f0/f2 justification/conf"
        );
        assert!(
            as_string.contains("ComplexTuple { enum_variant_justification , enum_variant_confidence , f0 , f0_justification , f0_confidence , f1 , f2 , f2_justification , f2_confidence } =>"),
            "Expected correct pattern matching in final match arm"
        );

        trace!("test_multiple_fields_with_and_without_justification completed successfully");
    }

    #[traced_test]
    fn test_only_fields_no_top_level_justification() {
        trace!("Starting test_only_fields_no_top_level_justification");

        let variant_ident: Ident = parse_quote! { OnlyFields };
        debug!("Created variant_ident = {:?}", variant_ident);

        // skip_self_just=true => no top-level justification
        // fields => let's say we have two fields, each with skip_self_just=false
        let expansions = UnnamedVariantExpansionBuilder::default()
            .field_declarations(vec![
                parse_quote! { #[serde(default)] f0 : bool },
                parse_quote! { #[serde(default)] f0_justification : String },
                parse_quote! { #[serde(default)] f0_confidence : f32 },
                parse_quote! { #[serde(default)] f1 : u32 },
                parse_quote! { #[serde(default)] f1_justification : String },
                parse_quote! { #[serde(default)] f1_confidence : f32 },
            ])
            .pattern_vars(vec![
                parse_quote! { f0 },
                parse_quote! { f0_justification },
                parse_quote! { f0_confidence },
                parse_quote! { f1 },
                parse_quote! { f1_justification },
                parse_quote! { f1_confidence },
            ])
            .item_exprs(vec![
                parse_quote! { f0 },
                parse_quote! { f1 },
            ])
            .just_vals(vec![
                parse_quote! { f0_justification : f0_justification },
                parse_quote! { f1_justification : f1_justification },
            ])
            .conf_vals(vec![
                parse_quote! { f0_confidence : f0_confidence },
                parse_quote! { f1_confidence : f1_confidence },
            ])
            .build()
            .expect("Failed to build UnnamedVariantExpansion");

        info!("Calling finalize_flat_unnamed_variant_ts for no top-level justification scenario");
        let generated = finalize_flat_unnamed_variant_ts(&variant_ident, &expansions);
        debug!("Received generated TokenStream: {}", generated.to_token_stream());

        let as_string = generated.to_string();
        info!("Generated tokens => {}", as_string);

        // We check that we do NOT have top-level justification or confidence fields
        assert!(!as_string.contains("enum_variant_justification"),
            "Should not have top-level justification field");
        assert!(!as_string.contains("enum_variant_confidence"),
            "Should not have top-level confidence field");
        assert!(
            as_string.contains("OnlyFields { #[serde ( default )] f0 : bool , #[serde ( default )] f0_justification : String , #[serde ( default )] f0_confidence : f32 , #[serde ( default )] f1 : u32 , #[serde ( default )] f1_justification : String , #[serde ( default )] f1_confidence : f32 } ,"),
            "Expected only fields with their just/conf expansions"
        );
        assert!(
            as_string.contains("OnlyFields { f0 , f0_justification , f0_confidence , f1 , f1_justification , f1_confidence } =>"),
            "Expected only fields in the match arm"
        );

        trace!("test_only_fields_no_top_level_justification completed successfully");
    }
}
