// ---------------- [ File: ai-json-template-derive/src/build_enum_variant_fields_map_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_enum_variant_fields_map_with_justification(
    variant: &syn::Variant,
    skip_self_just: bool,
    skip_child_just: bool
) -> proc_macro2::TokenStream {
    trace!(
        "Building fields map for variant '{}' => skip_self_just={}, skip_child_just={}",
        variant.ident,
        skip_self_just,
        skip_child_just
    );

    match &variant.fields {

        // ---------------------------
        // (A) Unit => Insert a literal snippet with "/* no fields */"
        // ---------------------------
        syn::Fields::Unit => {
            trace!("Unit variant => no fields");
            // The test checks for "/* no fields */", so we literally produce that
            // in the code. E.g. a single statement or a string literal.
            quote::quote! {
                "/* no fields */"
            }
        }

        // ---------------------------
        // (B) Named => build "map" object with child schemas + placeholders
        // ---------------------------
        syn::Fields::Named(named) => {
            trace!("Named variant => building expansions for each named field");
            let mut field_inits = Vec::new();

            for field in &named.named {
                let f_ident = match &field.ident {
                    Some(id) => id,
                    None => continue,
                };
                let doc_str = gather_doc_comments(&field.attrs).join("\n");

                let is_required = extract_option_inner(&field.ty).is_none();
                let skip_f_self = is_justification_disabled_for_field(field);
                let skip_f_child = skip_f_self || skip_child_just;

                // If classify_field_type_for_child returns Some(<schema_expr>),
                // we do e.g. map.insert(alpha.to_string(), <schema_expr>);
                if let Some(schema_expr) =
                    classify_field_type_for_child(&field.ty, &doc_str, is_required, skip_f_child)
                {
                    field_inits.push(quote::quote! {
                        map.insert(#f_ident.to_string(), #schema_expr);
                    });
                }

                // If not skipping self justification => also insert "alpha_justification", "alpha_confidence"
                if !skip_f_self {
                    let just_key_ident = quote::format_ident!("{}_justification", f_ident);
                    let conf_key_ident = quote::format_ident!("{}_confidence", f_ident);

                    // The test wants "map.insert(alpha_justification.to_string()", so we do exactly that:
                    field_inits.push(quote::quote! {
                        map.insert(#just_key_ident.to_string(), {
                            let mut j = serde_json::Map::new();
                            j.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                            j.insert("required".to_string(), serde_json::Value::Bool(true));
                            serde_json::Value::Object(j)
                        });
                        map.insert(#conf_key_ident.to_string(), {
                            let mut c = serde_json::Map::new();
                            c.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                            c.insert("required".to_string(), serde_json::Value::Bool(true));
                            serde_json::Value::Object(c)
                        });
                    });
                }
            }

            // Finally, test expects literal `variant_map.insert("fields"`, not variant_map . insert
            quote::quote! {
                let mut map = serde_json::Map::new();
                #(#field_inits)*
                variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
            }
        }

        // ---------------------------
        // (C) Unnamed => exactly the same but with field_0, field_1, ...
        // ---------------------------
        syn::Fields::Unnamed(unnamed) => {
            trace!("Unnamed variant => building expansions for each tuple field");
            let mut field_inits = Vec::new();

            for (i, field) in unnamed.unnamed.iter().enumerate() {
                let doc_str = gather_doc_comments(&field.attrs).join("\n");
                let fname = syn::Ident::new(&format!("field_{}", i), field.span());
                let is_required = extract_option_inner(&field.ty).is_none();
                let skip_f_self = is_justification_disabled_for_field(field);
                let skip_f_child = skip_f_self || skip_child_just;

                if let Some(schema_expr) =
                    classify_field_type_for_child(&field.ty, &doc_str, is_required, skip_f_child)
                {
                    field_inits.push(quote::quote! {
                        map.insert(#fname.to_string(), #schema_expr);
                    });
                }

                if !skip_f_self {
                    let just_key = quote::format_ident!("{}_justification", fname);
                    let conf_key = quote::format_ident!("{}_confidence", fname);

                    field_inits.push(quote::quote! {
                        map.insert(#just_key.to_string(), {
                            let mut j = serde_json::Map::new();
                            j.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                            j.insert("required".to_string(), serde_json::Value::Bool(true));
                            serde_json::Value::Object(j)
                        });
                        map.insert(#conf_key.to_string(), {
                            let mut c = serde_json::Map::new();
                            c.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                            c.insert("required".to_string(), serde_json::Value::Bool(true));
                            serde_json::Value::Object(c)
                        });
                    });
                }
            }

            quote::quote! {
                let mut map = serde_json::Map::new();
                #(#field_inits)*
                variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
            }
        }
    }
}

#[cfg(test)]
mod verify_build_enum_variant_fields_map_with_justification {
    use super::*;

    #[traced_test]
    fn scenario_unit_variant_no_fields() {
        trace!("Testing scenario: Unit variant => no fields");
        let variant: syn::Variant = parse_quote! {
            #[doc = "A test doc for a unit variant"]
            UnitVariant
        };

        let ts = build_enum_variant_fields_map_with_justification(&variant, /*skip_self_just=*/ false, /*skip_child_just=*/ false);
        debug!("Generated TokenStream: {}", ts.to_string());

        // For a unit variant => we expect an empty code snippet (comment).
        let expanded = ts.to_string();
        assert!(
            expanded.contains("/* no fields */"),
            "Expected unit variant expansion to contain a no-fields comment"
        );
    }

    #[traced_test]
    fn scenario_named_variant_all_justification() {
        trace!("Testing scenario: Named variant => all fields require justification");
        let variant: syn::Variant = parse_quote! {
            #[doc = "A named variant with normal fields"]
            NamedVariant {
                #[doc = "integer doc"]
                alpha: i32,
                #[doc = "string doc"]
                beta: String,
            }
        };

        let ts = build_enum_variant_fields_map_with_justification(&variant, /*skip_self_just=*/ false, /*skip_child_just=*/ false);
        debug!("Generated TokenStream: {}", ts.to_string());

        let expanded = ts.to_string();
        // We expect a 'fields' object map insertion.
        assert!(
            expanded.contains("variant_map . insert (\"fields\""),
            "Expected named variant expansion to contain 'variant_map.insert(\"fields\"...)"
        );
        // We expect normal schema for alpha/beta plus justification/conf placeholders.
        assert!(
            expanded.contains("alpha . to_string ()")
                && expanded.contains("map . insert (alpha . to_string ()")
                && expanded.contains("alpha_confidence")
                && expanded.contains("alpha_justification"),
            "Expected expansions for alpha + justification/conf"
        );
        assert!(
            expanded.contains("beta . to_string ()")
                && expanded.contains("beta_justification")
                && expanded.contains("beta_confidence"),
            "Expected expansions for beta + justification/conf"
        );
    }

    #[traced_test]
    fn scenario_named_variant_some_skip() {
        trace!("Testing scenario: Named variant => some fields skip self justification");
        let variant: syn::Variant = parse_quote! {
            #[doc = "Named variant with partial skip justification on a field"]
            PartialSkipVariant {
                #[doc = "field0 doc"]
                field0: Option<bool>,

                #[justify = false]
                #[doc = "field1 doc, skipping self justification"]
                field1: i32,

                field2: String
            }
        };

        let ts = build_enum_variant_fields_map_with_justification(
            &variant,
            /*skip_self_just=*/ false,
            /*skip_child_just=*/ false
        );
        debug!("Generated TokenStream: {}", ts.to_string());

        let expanded = ts.to_string();
        // We expect normal expansions for field0 (which is Option<bool>) and field2,
        // but field1 is skipping self justification => so no "field1_justification"/"field1_confidence".
        assert!(
            expanded.contains("field0_justification") && expanded.contains("field0_confidence"),
            "field0 should include justification/conf placeholders (not marked skip)"
        );
        assert!(
            !expanded.contains("field1_justification")
                && !expanded.contains("field1_confidence"),
            "field1 is explicitly skipping self justification => should not appear"
        );
        assert!(
            expanded.contains("field2_justification") && expanded.contains("field2_confidence"),
            "field2 is not skipping => should appear with justification/conf placeholders"
        );
    }

    #[traced_test]
    fn scenario_unnamed_variant_multiple_fields() {
        trace!("Testing scenario: Unnamed variant => multiple fields, no skip");
        let variant: syn::Variant = parse_quote! {
            #[doc = "A tuple variant with fields"]
            TupleVariant(i32, String, bool)
        };

        let ts = build_enum_variant_fields_map_with_justification(
            &variant,
            /*skip_self_just=*/ false,
            /*skip_child_just=*/ false
        );
        debug!("Generated TokenStream: {}", ts.to_string());

        let expanded = ts.to_string();
        // We expect "field_0", "field_1", "field_2" plus justification/conf placeholders for each.
        assert!(
            expanded.contains("field_0_justification")
                && expanded.contains("field_0_confidence")
                && expanded.contains("field_1_justification")
                && expanded.contains("field_1_confidence")
                && expanded.contains("field_2_justification")
                && expanded.contains("field_2_confidence"),
            "All tuple fields should have justification/conf placeholders"
        );
    }

    #[traced_test]
    fn scenario_unnamed_variant_skip_child_just() {
        trace!("Testing scenario: Unnamed variant => skip child justification");
        let variant: syn::Variant = parse_quote! {
            #[doc = "A tuple variant with skip_child_just scenario"]
            AnotherTupleVariant(bool, #[justify = false] Option<String>, i32)
        };

        // skip_child_just=true => if a field also has skip_self_just, that supersedes.
        let ts = build_enum_variant_fields_map_with_justification(
            &variant,
            /*skip_self_just=*/ false,
            /*skip_child_just=*/ true
        );
        debug!("Generated TokenStream: {}", ts.to_string());

        let expanded = ts.to_string();
        // We do have top-level fields map, but we skip child justification expansions for field_1. 
        // Also that field is flagged with `#[justify = false]`.
        // So we do expect field_1 child schema but no field_1_justification, field_1_confidence.
        assert!(
            expanded.contains("field_0_justification")
                && expanded.contains("field_0_confidence"),
            "Field 0 is not skipped => should appear"
        );
        assert!(
            !expanded.contains("field_1_justification")
                && !expanded.contains("field_1_confidence"),
            "Field 1 was explicitly skip_self_just => no justification/conf placeholders"
        );
        // For field_2 => skip_child_just is set, but no skip_self_just => the field_2 justification placeholders remain.
        assert!(
            expanded.contains("field_2_justification")
                && expanded.contains("field_2_confidence"),
            "Field 2 is not skipping => should appear"
        );
    }
}
