// ---------------- [ File: ai-json-template-derive/src/build_enum_variant_expr_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_enum_variant_expr_with_justification(
    variant: &syn::Variant,
    variant_name_str: &str,
    variant_docs: &str,
    variant_kind_str: &str,
    fields_insertion_ts: proc_macro2::TokenStream,
    skip_self_just: bool
) -> proc_macro2::TokenStream {
    trace!(
        "Building enum variant expr with justification => variant: '{}', kind: '{}'",
        variant_name_str,
        variant_kind_str
    );

    // Optionally add top-level "variant_justification" & "variant_confidence"
    let top_level_just_conf = if !skip_self_just {
        debug!("Adding top-level variant_justification and variant_confidence for '{}'", variant_name_str);
        quote::quote! {
            {
                let mut j_obj = serde_json::Map::new();
                j_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                j_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                variant_map.insert("variant_justification".to_string(), serde_json::Value::Object(j_obj));

                let mut c_obj = serde_json::Map::new();
                c_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                c_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                variant_map.insert("variant_confidence".to_string(), serde_json::Value::Object(c_obj));
            }
        }
    } else {
        trace!("skip_self_just=true => No variant_justification/confidence for '{}'", variant_name_str);
        quote::quote! {}
    };

    quote::quote! {
        {
            let mut variant_map = serde_json::Map::new();
            variant_map.insert("variant_name".to_string(), serde_json::Value::String(#variant_name_str.to_string()));
            variant_map.insert("variant_docs".to_string(), serde_json::Value::String(#variant_docs.to_string()));
            variant_map.insert("variant_type".to_string(), serde_json::Value::String(#variant_kind_str.to_string()));

            #fields_insertion_ts
            #top_level_just_conf

            serde_json::Value::Object(variant_map)
        }
    }
}

#[cfg(test)]
mod test_build_enum_variant_expr_with_justification {
    use super::*;

    #[traced_test]
    fn test_unit_variant_with_skip_self_just_false() {
        trace!("Starting test_unit_variant_with_skip_self_just_false");
        let variant: Variant = parse_quote! {
            /// This is a unit variant for testing.
            UnitVariant
        };
        let variant_name_str = "UnitVariant";
        let variant_docs = "This is a unit variant for testing.";
        let variant_kind_str = "unit_variant";

        // We simulate no field insertion here
        let fields_insertion_ts = quote! {};

        let tokens = build_enum_variant_expr_with_justification(
            &variant,
            variant_name_str,
            variant_docs,
            variant_kind_str,
            fields_insertion_ts,
            /* skip_self_just = */ false
        );

        debug!(?tokens, "Generated tokens for test_unit_variant_with_skip_self_just_false");
        let expanded = tokens.to_string();

        // The generated code should include "variant_justification" and "variant_confidence"
        assert!(
            expanded.contains("variant_justification"),
            "Expected variant_justification to be present, got:\n{}",
            expanded
        );
        assert!(
            expanded.contains("variant_confidence"),
            "Expected variant_confidence to be present, got:\n{}",
            expanded
        );
        assert!(
            expanded.contains("UnitVariant"),
            "Expected variant_name_str to appear, got:\n{}",
            expanded
        );
        trace!("Completed test_unit_variant_with_skip_self_just_false");
    }

    #[traced_test]
    fn test_unit_variant_with_skip_self_just_true() {
        trace!("Starting test_unit_variant_with_skip_self_just_true");
        let variant: Variant = parse_quote! {
            /// This is a unit variant for skipping self justification.
            AnotherUnitVariant
        };
        let variant_name_str = "AnotherUnitVariant";
        let variant_docs = "This is a unit variant for skipping self justification.";
        let variant_kind_str = "unit_variant";

        let fields_insertion_ts = quote! {};

        let tokens = build_enum_variant_expr_with_justification(
            &variant,
            variant_name_str,
            variant_docs,
            variant_kind_str,
            fields_insertion_ts,
            /* skip_self_just = */ true
        );

        debug!(?tokens, "Generated tokens for test_unit_variant_with_skip_self_just_true");
        let expanded = tokens.to_string();

        // The generated code should NOT include variant_justification nor variant_confidence
        assert!(
            !expanded.contains("variant_justification"),
            "Should NOT have inserted variant_justification, got:\n{}",
            expanded
        );
        assert!(
            !expanded.contains("variant_confidence"),
            "Should NOT have inserted variant_confidence, got:\n{}",
            expanded
        );
        assert!(
            expanded.contains("AnotherUnitVariant"),
            "Expected variant_name_str to appear, got:\n{}",
            expanded
        );
        trace!("Completed test_unit_variant_with_skip_self_just_true");
    }

    #[traced_test]
    fn test_named_variant_with_skip_self_just_false() {
        trace!("Starting test_named_variant_with_skip_self_just_false");
        let variant: Variant = parse_quote! {
            /// Named variant docs
            NamedVariant {
                alpha: i32,
                beta: String
            }
        };
        let variant_name_str = "NamedVariant";
        let variant_docs = "Named variant docs";
        let variant_kind_str = "struct_variant";

        // Pretend we're inserting field schemas
        let fields_insertion_ts = quote! {
            let mut fields_map = serde_json::Map::new();
            fields_map.insert("alpha".to_string(), serde_json::Value::Null);
            fields_map.insert("beta".to_string(), serde_json::Value::Null);
            variant_map.insert("fields".to_string(), serde_json::Value::Object(fields_map));
        };

        let tokens = build_enum_variant_expr_with_justification(
            &variant,
            variant_name_str,
            variant_docs,
            variant_kind_str,
            fields_insertion_ts,
            /* skip_self_just = */ false
        );

        debug!(?tokens, "Generated tokens for test_named_variant_with_skip_self_just_false");
        let expanded = tokens.to_string();

        // The generated code should include top-level justification/conf placeholders
        assert!(
            expanded.contains("variant_justification"),
            "Expected variant_justification to be present, got:\n{}",
            expanded
        );
        assert!(
            expanded.contains("variant_confidence"),
            "Expected variant_confidence to be present, got:\n{}",
            expanded
        );
        // And also references to alpha/beta
        assert!(
            expanded.contains("alpha"),
            "Expected 'alpha' in fields insertion, got:\n{}",
            expanded
        );
        assert!(
            expanded.contains("beta"),
            "Expected 'beta' in fields insertion, got:\n{}",
            expanded
        );
        trace!("Completed test_named_variant_with_skip_self_just_false");
    }

    #[traced_test]
    fn test_unnamed_variant_with_skip_self_just_true() {
        trace!("Starting test_unnamed_variant_with_skip_self_just_true");
        let variant: Variant = parse_quote! {
            /// Unnamed (tuple) variant docs
            TupleVariant(u8, bool)
        };
        let variant_name_str = "TupleVariant";
        let variant_docs = "Unnamed (tuple) variant docs";
        let variant_kind_str = "tuple_variant";

        // Mock some child field insertions
        let fields_insertion_ts = quote! {
            let mut fields_map = serde_json::Map::new();
            fields_map.insert("field_0".to_string(), serde_json::Value::Null);
            fields_map.insert("field_1".to_string(), serde_json::Value::Null);
            variant_map.insert("fields".to_string(), serde_json::Value::Object(fields_map));
        };

        let tokens = build_enum_variant_expr_with_justification(
            &variant,
            variant_name_str,
            variant_docs,
            variant_kind_str,
            fields_insertion_ts,
            /* skip_self_just = */ true
        );

        debug!(?tokens, "Generated tokens for test_unnamed_variant_with_skip_self_just_true");
        let expanded = tokens.to_string();

        // Because skip_self_just = true, we expect no top-level justification/conf
        assert!(
            !expanded.contains("variant_justification"),
            "Should not have variant_justification, got:\n{}",
            expanded
        );
        assert!(
            !expanded.contains("variant_confidence"),
            "Should not have variant_confidence, got:\n{}",
            expanded
        );
        // But we do expect references to field_0 and field_1
        assert!(
            expanded.contains("field_0"),
            "Expected 'field_0' in fields insertion, got:\n{}",
            expanded
        );
        assert!(
            expanded.contains("field_1"),
            "Expected 'field_1' in fields insertion, got:\n{}",
            expanded
        );
        trace!("Completed test_unnamed_variant_with_skip_self_just_true");
    }
}
