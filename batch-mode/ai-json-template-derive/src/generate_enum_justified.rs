// ---------------- [ File: ai-json-template-derive/src/generate_enum_justified.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_enum_justified(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    span: proc_macro2::Span
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream)
{
    trace!("Beginning generate_enum_justified for enum '{}'", ty_ident);

    let enum_just_ident = syn::Ident::new(&format!("{}Justification", ty_ident), span);
    let enum_conf_ident = syn::Ident::new(&format!("{}Confidence",   ty_ident), span);
    let justified_ident = syn::Ident::new(&format!("Justified{}",    ty_ident), span);

    let (
        just_variants,
        conf_variants,
        first_variant_ident,
        first_variant_just_fields,
        first_variant_conf_fields
    ) = collect_variant_fields_for_just_conf(data_enum, &ty_ident, span, &enum_just_ident, &enum_conf_ident);

    let enum_just_ts = build_enum_justification(
        &enum_just_ident,
        &just_variants,
        first_variant_ident.as_ref(),
        &first_variant_just_fields
    );

    let enum_conf_ts = build_enum_confidence(
        &enum_conf_ident,
        &conf_variants,
        first_variant_ident.as_ref(),
        &first_variant_conf_fields
    );

    let justified_ts = build_justified_enum_struct(
        &ty_ident,
        &enum_just_ident,
        &enum_conf_ident,
        &justified_ident
    );

    trace!("Completed generate_enum_justified for enum '{}'", ty_ident);
    (enum_just_ts, enum_conf_ts, justified_ts)
}

#[cfg(test)]
mod test_generate_enum_justified {
    use super::*;

    #[traced_test]
    fn test_empty_enum() {
        trace!("Testing generate_enum_justified with an empty enum");
        // Parse an entire item enum, then convert its fields to DataEnum
        let item_enum: ItemEnum = parse_quote! {
            enum EmptyEnum {}
        };

        let data_enum = DataEnum {
            enum_token: item_enum.enum_token,
            brace_token: item_enum.brace_token,
            variants: item_enum.variants,
        };

        let (just_ts, conf_ts, just_enum_ts) =
            generate_enum_justified(&item_enum.ident, &data_enum, item_enum.ident.span());

        trace!("Justification tokens: {}", just_ts);
        trace!("Confidence tokens: {}", conf_ts);
        trace!("Justified enum tokens: {}", just_enum_ts);

        let just_str = just_ts.to_string();
        let conf_str = conf_ts.to_string();
        let enum_str = just_enum_ts.to_string();

        assert!(
            just_str.contains("enum EmptyEnumJustification"),
            "Expected Justification enum name not found in output."
        );
        assert!(
            conf_str.contains("enum EmptyEnumConfidence"),
            "Expected Confidence enum name not found in output."
        );
        assert!(
            enum_str.contains("struct JustifiedEmptyEnum"),
            "Expected Justified struct name not found in output."
        );
    }

    #[traced_test]
    fn test_unit_variants_only() {
        trace!("Testing generate_enum_justified with a unit-variants-only enum");

        let item_enum: ItemEnum = parse_quote! {
            enum UnitOnlyEnum {
                Alpha,
                Beta,
                #[justify=false]
                Gamma
            }
        };
        let data_enum = DataEnum {
            enum_token: item_enum.enum_token,
            brace_token: item_enum.brace_token,
            variants: item_enum.variants,
        };

        let (just_ts, conf_ts, just_enum_ts) =
            generate_enum_justified(&item_enum.ident, &data_enum, item_enum.ident.span());

        debug!("Justification tokens: {}", just_ts);
        debug!("Confidence tokens: {}", conf_ts);
        debug!("Justified enum tokens: {}", just_enum_ts);

        let just_str = just_ts.to_string();
        let conf_str = conf_ts.to_string();
        let enum_str = just_enum_ts.to_string();

        // Check if we define variant fields correctly 
        // skip_self_just is false by default, overridden by #[justify=false].
        assert!(
            just_str.contains("Alpha { variant_justification : String }"),
            "Expected justification field in 'Alpha' variant."
        );
        assert!(
            !just_str.contains("Gamma { variant_justification : String }"),
            "Expected no justification field in 'Gamma' due to #[justify=false]."
        );
        assert!(
            conf_str.contains("Beta { variant_confidence : f32 }"),
            "Expected confidence field in 'Beta' variant."
        );
        assert!(
            enum_str.contains("JustifiedUnitOnlyEnum"),
            "Expected a Justified struct name for UnitOnlyEnum."
        );
    }

    #[traced_test]
    fn test_named_variants() {
        trace!("Testing generate_enum_justified with named variants");

        let item_enum: ItemEnum = parse_quote! {
            enum NamedVariantsEnum {
                FirstVariant {
                    alpha: i32,
                    #[justify=false]
                    beta: bool
                },
                SecondVariant {
                    gamma: String,
                    delta: Vec<u64>
                }
            }
        };
        let data_enum = DataEnum {
            enum_token: item_enum.enum_token,
            brace_token: item_enum.brace_token,
            variants: item_enum.variants,
        };

        let (just_ts, conf_ts, just_enum_ts) =
            generate_enum_justified(&item_enum.ident, &data_enum, item_enum.ident.span());

        info!("Justification tokens: {}", just_ts);
        info!("Confidence tokens: {}", conf_ts);
        info!("Justified enum tokens: {}", just_enum_ts);

        let just_str = just_ts.to_string();
        let conf_str = conf_ts.to_string();

        // "FirstVariant" => alpha_justification, skip beta_justification
        assert!(
            just_str.contains("FirstVariant { variant_justification : String , alpha_justification : String }"),
            "Expected 'alpha_justification' in FirstVariant"
        );
        assert!(
            !just_str.contains("beta_justification"),
            "Did not expect 'beta_justification' in FirstVariant due to #[justify=false]"
        );

        // "SecondVariant" => gamma_justification, delta_justification
        assert!(
            just_str.contains("SecondVariant { variant_justification : String , gamma_justification : String , delta_justification : String }"),
            "Expected 'gamma_justification' and 'delta_justification' in SecondVariant"
        );
        // Confidence expansions
        assert!(
            conf_str.contains("alpha_confidence : f32"),
            "Expected 'alpha_confidence' in FirstVariant"
        );
        assert!(
            conf_str.contains("delta_confidence : f32"),
            "Expected 'delta_confidence' in SecondVariant"
        );
    }

    #[traced_test]
    fn test_unnamed_variants() {
        trace!("Testing generate_enum_justified with tuple variants (Fields::Unnamed)");

        let item_enum: ItemEnum = parse_quote! {
            enum UnnamedVariantsEnum {
                Single(u8),
                Pair(#[justify=false] String, i64),
            }
        };
        let data_enum = DataEnum {
            enum_token: item_enum.enum_token,
            brace_token: item_enum.brace_token,
            variants: item_enum.variants,
        };

        let (just_ts, conf_ts, just_enum_ts) =
            generate_enum_justified(&item_enum.ident, &data_enum, item_enum.ident.span());

        warn!("Justification tokens: {}", just_ts);
        warn!("Confidence tokens: {}", conf_ts);
        warn!("Justified enum tokens: {}", just_enum_ts);

        let just_str = just_ts.to_string();
        let conf_str = conf_ts.to_string();

        // "Single" => single unnamed field => field_0_justification
        assert!(
            just_str.contains("Single { variant_justification : String , field_0_justification : String }"),
            "Expected field_0_justification in Single variant"
        );
        // "Pair" => first field is #[justify=false], second is i64 => only second gets justification
        assert!(
            just_str.contains("Pair { variant_justification : String , field_1_justification : String }"),
            "Expected only field_1_justification in Pair variant"
        );
        assert!(
            !just_str.contains("field_0_justification : String , field_1_justification : String , field_2_justification"),
            "Did not expect a justification field for Pair's field_0"
        );
        // Confidence is analogous
        assert!(
            conf_str.contains("field_1_confidence"),
            "Expected second field confidence in Pair variant"
        );
        assert!(
            !conf_str.contains("field_0_confidence : f32 , field_1_confidence"),
            "Did not expect confidence for Pair's first field"
        );
    }

    #[traced_test]
    fn test_mixed_variant_types() {
        trace!("Testing generate_enum_justified with a mix of unit, named, and unnamed variants");

        let item_enum: ItemEnum = parse_quote! {
            enum MixedVariantsEnum {
                Unit,
                Named {
                    x: String,
                    #[justify=false]
                    y: u32
                },
                Unnamed(bool, i16)
            }
        };
        let data_enum = DataEnum {
            enum_token: item_enum.enum_token,
            brace_token: item_enum.brace_token,
            variants: item_enum.variants,
        };

        let (just_ts, conf_ts, just_enum_ts) =
            generate_enum_justified(&item_enum.ident, &data_enum, item_enum.ident.span());

        error!("Justification tokens: {}", just_ts);
        error!("Confidence tokens: {}", conf_ts);
        error!("Justified enum tokens: {}", just_enum_ts);

        let just_str = just_ts.to_string();
        let conf_str = conf_ts.to_string();

        // "Unit" => variant_justification
        assert!(
            just_str.contains("Unit { variant_justification : String }"),
            "Expected top-level justification in Unit variant"
        );
        // "Named" => x gets justification, y is skipped
        assert!(
            just_str.contains("Named { variant_justification : String , x_justification : String }"),
            "Expected x_justification in Named variant"
        );
        assert!(
            !just_str.contains("y_justification"),
            "Did not expect y_justification in Named variant"
        );
        // "Unnamed" => two fields => both get justification unless specify otherwise
        assert!(
            just_str.contains("Unnamed { variant_justification : String , field_0_justification : String , field_1_justification : String }"),
            "Expected field_0_justification and field_1_justification in Unnamed variant"
        );

        // Confidence checks:
        assert!(
            conf_str.contains("Unit { variant_confidence : f32 }"),
            "Expected top-level variant_confidence for Unit"
        );
        assert!(
            conf_str.contains("x_confidence"),
            "Expected x_confidence in Named variant"
        );
        assert!(
            conf_str.contains("field_0_confidence") && conf_str.contains("field_1_confidence"),
            "Expected field_0_confidence and field_1_confidence in Unnamed variant"
        );
    }
}
