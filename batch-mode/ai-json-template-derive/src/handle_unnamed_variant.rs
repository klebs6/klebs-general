// ---------------- [ File: ai-json-template-derive/src/handle_unnamed_variant.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn handle_unnamed_variant(
    var_ident: &syn::Ident,
    unnamed_fields: &syn::FieldsUnnamed,
    skip_self_just: bool,
    is_first_variant: bool
) -> (
    proc_macro2::TokenStream, // variant in Justification enum
    proc_macro2::TokenStream, // variant in Confidence enum
    Vec<String>,              // justification fields (if first variant)
    Vec<String>               // confidence fields (if first variant)
)
{
    debug!(
        "Handling unnamed variant '{}', skip_self_just={}, is_first_variant={}",
        var_ident,
        skip_self_just,
        is_first_variant
    );

    let mut j_fields = Vec::new();
    let mut c_fields = Vec::new();
    let mut out_just_names = Vec::new();
    let mut out_conf_names = Vec::new();

    if !skip_self_just {
        j_fields.push(quote::quote! { variant_justification: String, });
        c_fields.push(quote::quote! { variant_confidence: f32, });

        if is_first_variant {
            out_just_names.push("variant_justification".to_string());
            out_conf_names.push("variant_confidence".to_string());
        }
    }

    for (idx, field) in unnamed_fields.unnamed.iter().enumerate() {
        if is_justification_enabled(field) {
            let j_id = syn::Ident::new(
                &format!("field_{}_justification", idx),
                field.span()
            );
            let c_id = syn::Ident::new(
                &format!("field_{}_confidence", idx),
                field.span()
            );
            j_fields.push(quote::quote! { #j_id: String, });
            c_fields.push(quote::quote! { #c_id: f32, });

            if is_first_variant {
                out_just_names.push(format!("field_{}_justification", idx));
                out_conf_names.push(format!("field_{}_confidence", idx));
            }
        }
    }

    let just_variant = quote::quote! {
        #var_ident { #(#j_fields)* }
    };
    let conf_variant = quote::quote! {
        #var_ident { #(#c_fields)* }
    };

    (just_variant, conf_variant, out_just_names, out_conf_names)
}

#[cfg(test)]
mod test_handle_unnamed_variant {
    use super::*;

    #[traced_test]
    fn test_no_fields_skip_self_just_false_first_variant_false() {
        trace!("Starting: test_no_fields_skip_self_just_false_first_variant_false");
        // Build a variant ident
        let var_ident = syn::Ident::new("NoFieldsVariant", proc_macro2::Span::call_site());

        // Build an empty FieldsUnnamed
        let unnamed_fields = FieldsUnnamed {
            paren_token: Default::default(),
            unnamed: syn::punctuated::Punctuated::new(),
        };

        let skip_self_just = false;
        let is_first_variant = false;

        debug!("Calling handle_unnamed_variant with no fields...");
        let (just_enum_variant, conf_enum_variant, just_fields, conf_fields) = handle_unnamed_variant(
            &var_ident,
            &unnamed_fields,
            skip_self_just,
            is_first_variant
        );

        info!("Asserting that top-level justification/conf exist because skip_self_just=false");
        let just_str = just_enum_variant.to_string();
        let conf_str = conf_enum_variant.to_string();

        // Since skip_self_just=false, we expect "variant_justification" and "variant_confidence" in each.
        assert!(just_str.contains("variant_justification"), "Expected top-level justification field in: {}", just_str);
        assert!(conf_str.contains("variant_confidence"), "Expected top-level confidence field in: {}", conf_str);

        debug!("Verifying field name arrays are empty (no actual data fields) but top-level fields exist");
        // Because it's not the first variant, we do not add them to the "first variant field" lists, except if they come from the top-level logic.
        // handle_unnamed_variant always adds them to the struct if skip_self_just=false,
        // but if we rely on them being "the first variant's fields," it only pushes them if is_first_variant=true.
        assert_eq!(just_fields, Vec::<String>::new(), "No data fields, so no additional justification fields beyond the top-level should appear for the 'first variant' set");
        assert_eq!(conf_fields, Vec::<String>::new(), "No data fields, so no additional confidence fields beyond the top-level should appear for the 'first variant' set");
    }

    #[traced_test]
    fn test_one_field_skip_self_just_false_first_variant_true() {
        trace!("Starting: test_one_field_skip_self_just_false_first_variant_true");
        let var_ident = syn::Ident::new("OneFieldVariant", proc_macro2::Span::call_site());

        // Build an unnamed field
        let field_type: Type = parse_quote! { String };
        let field = Field {
            attrs: vec![],
            vis: parse_quote! { },
            ident: None,
            colon_token: None,
            ty: field_type,
            mutability: FieldMutability::None,
        };

        let mut unnamed_fields_punct = syn::punctuated::Punctuated::new();
        unnamed_fields_punct.push(field);

        let unnamed_fields = FieldsUnnamed {
            paren_token: Default::default(),
            unnamed: unnamed_fields_punct,
        };

        let skip_self_just = false;
        let is_first_variant = true;

        debug!("Calling handle_unnamed_variant with 1 field...");
        let (just_enum_variant, conf_enum_variant, just_fields, conf_fields) = handle_unnamed_variant(
            &var_ident,
            &unnamed_fields,
            skip_self_just,
            is_first_variant
        );

        let just_str = just_enum_variant.to_string();
        let conf_str = conf_enum_variant.to_string();

        info!("Expecting the top-level justification/conf fields AND the field_0_justification/confidence because skip_self_just=false");
        assert!(just_str.contains("variant_justification"), "Expected variant_justification in: {}", just_str);
        assert!(conf_str.contains("variant_confidence"), "Expected variant_confidence in: {}", conf_str);

        debug!("Ensuring we see references to 'field_0_justification' and 'field_0_confidence' in the expansions");
        assert!(
            just_str.contains("field_0_justification"),
            "Expected field_0_justification in: {}",
            just_str
        );
        assert!(
            conf_str.contains("field_0_confidence"),
            "Expected field_0_confidence in: {}",
            conf_str
        );

        debug!("Checking that first variant added these justification/conf fields to the returned lists");
        assert_eq!(
            just_fields,
            vec!["variant_justification".to_string(), "field_0_justification".to_string()],
            "Expected top-level plus the single field for the first variant"
        );
        assert_eq!(
            conf_fields,
            vec!["variant_confidence".to_string(), "field_0_confidence".to_string()],
            "Expected top-level plus the single field for the first variant"
        );
    }

    #[traced_test]
    fn test_two_fields_skip_self_just_true_first_variant_false() {
        trace!("Starting: test_two_fields_skip_self_just_true_first_variant_false");
        let var_ident = syn::Ident::new("TwoFieldsVariant", proc_macro2::Span::call_site());

        // Build two unnamed fields
        let field_one: Type = parse_quote! { i32 };
        let field_two: Type = parse_quote! { bool };

        let f1 = Field {
            attrs: vec![],
            vis: parse_quote! { },
            ident: None,
            colon_token: None,
            ty: field_one,
            mutability: FieldMutability::None,
        };
        let f2 = Field {
            attrs: vec![],
            vis: parse_quote! { },
            ident: None,
            colon_token: None,
            ty: field_two,
            mutability: FieldMutability::None,
        };

        let mut unnamed_fields_punct = syn::punctuated::Punctuated::new();
        unnamed_fields_punct.push(f1);
        unnamed_fields_punct.push(f2);

        let unnamed_fields = FieldsUnnamed {
            paren_token: Default::default(),
            unnamed: unnamed_fields_punct,
        };

        let skip_self_just = true;
        let is_first_variant = false;

        debug!("Calling handle_unnamed_variant with 2 fields, skip_self_just=true...");
        let (just_enum_variant, conf_enum_variant, just_fields, conf_fields) = handle_unnamed_variant(
            &var_ident,
            &unnamed_fields,
            skip_self_just,
            is_first_variant
        );

        let just_str = just_enum_variant.to_string();
        let conf_str = conf_enum_variant.to_string();

        info!("Since skip_self_just=true, we do NOT expect top-level variant_justification/confidence in expansions");
        assert!(
            !just_str.contains("variant_justification"),
            "Did not expect variant_justification in: {}",
            just_str
        );
        assert!(
            !conf_str.contains("variant_confidence"),
            "Did not expect variant_confidence in: {}",
            conf_str
        );

        debug!("We do expect the expansions to reflect no justification/conf fields for each unnamed field (since skip_self_just=true means no field_just/conf either)");
        assert!(
            !just_str.contains("field_0_justification"),
            "skip_self_just=true => no justification placeholders for field_0"
        );
        assert!(
            !just_str.contains("field_1_justification"),
            "skip_self_just=true => no justification placeholders for field_1"
        );
        assert!(
            !conf_str.contains("field_0_confidence"),
            "skip_self_just=true => no confidence placeholders for field_0"
        );
        assert!(
            !conf_str.contains("field_1_confidence"),
            "skip_self_just=true => no confidence placeholders for field_1"
        );

        debug!("Neither the top-level nor the field-level expansions should exist => returned lists are empty");
        assert!(just_fields.is_empty(), "We do not push anything if skip_self_just=true");
        assert!(conf_fields.is_empty(), "We do not push anything if skip_self_just=true");
    }

    #[traced_test]
    fn test_multiple_fields_skip_self_just_false_first_variant_true() {
        trace!("Starting: test_multiple_fields_skip_self_just_false_first_variant_true");
        let var_ident = syn::Ident::new("MultiFieldsVariant", proc_macro2::Span::call_site());

        // Build multiple unnamed fields
        let field_types: Vec<Type> = vec![
            parse_quote! { String },
            parse_quote! { u32 },
            parse_quote! { bool },
        ];
        let fields: Vec<_> = field_types
            .into_iter()
            .map(|t| Field {
                attrs: vec![],
                vis: parse_quote! { },
                ident: None,
                colon_token: None,
                ty: t,
                mutability: FieldMutability::None,
            })
            .collect();

        let mut unnamed_fields_punct = syn::punctuated::Punctuated::new();
        for f in fields {
            unnamed_fields_punct.push(f);
        }

        let unnamed_fields = FieldsUnnamed {
            paren_token: Default::default(),
            unnamed: unnamed_fields_punct,
        };

        let skip_self_just = false;
        let is_first_variant = true;

        debug!("Calling handle_unnamed_variant with multiple fields...");
        let (just_enum_variant, conf_enum_variant, just_fields, conf_fields) = handle_unnamed_variant(
            &var_ident,
            &unnamed_fields,
            skip_self_just,
            is_first_variant
        );

        let just_str = just_enum_variant.to_string();
        let conf_str = conf_enum_variant.to_string();

        info!("We expect top-level variant_justification/confidence plus field_0_justification, field_1_justification, field_2_justification, etc.");
        assert!(just_str.contains("variant_justification"), "Should have top-level justification field");
        assert!(conf_str.contains("variant_confidence"), "Should have top-level confidence field");
        assert!(just_str.contains("field_0_justification"), "Should have field_0_justification");
        assert!(just_str.contains("field_1_justification"), "Should have field_1_justification");
        assert!(just_str.contains("field_2_justification"), "Should have field_2_justification");
        assert!(conf_str.contains("field_0_confidence"), "Should have field_0_confidence");
        assert!(conf_str.contains("field_1_confidence"), "Should have field_1_confidence");
        assert!(conf_str.contains("field_2_confidence"), "Should have field_2_confidence");

        debug!("Since is_first_variant=true, we expect the top-level plus every field to appear in just_fields/conf_fields");
        let expected_j = vec![
            "variant_justification".to_string(),
            "field_0_justification".to_string(),
            "field_1_justification".to_string(),
            "field_2_justification".to_string(),
        ];
        let expected_c = vec![
            "variant_confidence".to_string(),
            "field_0_confidence".to_string(),
            "field_1_confidence".to_string(),
            "field_2_confidence".to_string(),
        ];

        assert_eq!(just_fields, expected_j, "Mismatch in justification fields for first variant with multiple fields");
        assert_eq!(conf_fields, expected_c, "Mismatch in confidence fields for first variant with multiple fields");
    }
}
