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

    let inner_ts = match &variant.fields {
        // ---------------------------
        // (A) Unit
        // ---------------------------
        syn::Fields::Unit => {
            trace!("Unit variant => no fields");
            quote::quote! {}
        }

        // ---------------------------
        // (B) Named fields
        // ---------------------------
        syn::Fields::Named(named) => {
            trace!("Named fields variant => processing named fields");
            let mut field_inits = Vec::new();

            for field in &named.named {
                let f_ident = match &field.ident {
                    Some(id) => id,
                    None => {
                        warn!("Encountered a named field without an ident (unexpected). Skipping.");
                        continue;
                    }
                };
                let field_name_str = f_ident.to_string();
                trace!(
                    "Handling named field: {}",
                    field_name_str
                );

                let field_name_lit = syn::LitStr::new(&field_name_str, f_ident.span());
                let doc_str = gather_doc_comments(&field.attrs).join("\n");
                let is_required = extract_option_inner(&field.ty).is_none();
                trace!(
                    "Field '{}' doc_str.len()={}, is_required={}",
                    field_name_str,
                    doc_str.len(),
                    is_required
                );

                let skip_f_self  = is_justification_disabled_for_field(field);
                let skip_f_inner = skip_child_just || skip_f_self;
                trace!(
                    "Field '{}' skip_f_self={}, skip_f_inner={}",
                    field_name_str,
                    skip_f_self,
                    skip_f_inner
                );

                // We'll only emit placeholders if we do NOT skip self
                // and either not skipping child OR the child is a leaf.
                let emit_placeholders = !skip_f_self && (!skip_child_just || is_leaf_type(&field.ty));
                trace!(
                    "Field '{}' => emit_placeholders={}",
                    field_name_str,
                    emit_placeholders
                );

                if let Some(schema_expr) = classify_field_type_for_child(
                    &field.ty,
                    &doc_str,
                    is_required,
                    skip_f_inner
                ) {
                    trace!(
                        "Field '{}' => inserting child schema expression",
                        field_name_str
                    );
                    field_inits.push(quote::quote! {
                        map.insert(#field_name_lit.to_string(), #schema_expr);
                    });
                } else {
                    debug!(
                        "Field '{}' => No schema expression was generated, possibly unsupported?",
                        field_name_str
                    );
                }

                if emit_placeholders {
                    let just_str = format!("{}_justification", field_name_str);
                    let conf_str = format!("{}_confidence", field_name_str);

                    let just_lit = syn::LitStr::new(&just_str, f_ident.span());
                    let conf_lit = syn::LitStr::new(&conf_str, f_ident.span());

                    trace!(
                        "Field '{}' => inserting justification/conf placeholders: {}, {}",
                        field_name_str,
                        just_str,
                        conf_str
                    );

                    field_inits.push(quote::quote! {
                        map.insert(#just_lit.to_string(), {
                            let mut j = serde_json::Map::new();
                            j.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                            j.insert("required".to_string(), serde_json::Value::Bool(true));
                            serde_json::Value::Object(j)
                        });
                        map.insert(#conf_lit.to_string(), {
                            let mut c = serde_json::Map::new();
                            c.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                            c.insert("required".to_string(), serde_json::Value::Bool(true));
                            serde_json::Value::Object(c)
                        });
                    });
                }
            }

            trace!("Finished building field map for named variant");
            quote::quote! {
                let mut map = serde_json::Map::new();
                #(#field_inits)*
                variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
            }
        }

        // ---------------------------
        // (C) Unnamed fields
        // ---------------------------
        syn::Fields::Unnamed(unnamed) => {
            trace!("Unnamed variant => building expansions for each tuple field");
            let mut field_inits = Vec::new();

            for (i, field) in unnamed.unnamed.iter().enumerate() {
                let fname_str = format!("field_{}", i);
                let fname_lit = syn::LitStr::new(&fname_str, field.span());

                let doc_str = gather_doc_comments(&field.attrs).join("\n");
                let is_required = extract_option_inner(&field.ty).is_none();

                trace!(
                    "Handling unnamed field index={}, is_required={}",
                    i,
                    is_required
                );

                let skip_f_self  = is_justification_disabled_for_field(field);
                let skip_f_inner = skip_child_just || skip_f_self;

                trace!(
                    "Unnamed field index={} => skip_f_self={}, skip_f_inner={}",
                    i,
                    skip_f_self,
                    skip_f_inner
                );

                if let Some(schema_expr) = classify_field_type_for_child(
                    &field.ty,
                    &doc_str,
                    is_required,
                    skip_f_inner
                ) {
                    trace!(
                        "Unnamed field index={} => inserting child schema expression",
                        i
                    );
                    field_inits.push(quote::quote! {
                        map.insert(#fname_lit.to_string(), #schema_expr);
                    });
                } else {
                    debug!(
                        "Unnamed field index={} => no schema expression generated",
                        i
                    );
                }

                if !skip_f_self {
                    let just_key_str = format!("field_{}_justification", i);
                    let just_key_lit = syn::LitStr::new(&just_key_str, field.span());

                    let conf_key_str = format!("field_{}_confidence", i);
                    let conf_key_lit = syn::LitStr::new(&conf_key_str, field.span());

                    trace!(
                        "Unnamed field index={} => inserting justification/conf placeholders",
                        i
                    );

                    field_inits.push(quote::quote! {
                        map.insert(#just_key_lit.to_string(), {
                            let mut j = serde_json::Map::new();
                            j.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                            j.insert("required".to_string(), serde_json::Value::Bool(true));
                            serde_json::Value::Object(j)
                        });
                        map.insert(#conf_key_lit.to_string(), {
                            let mut c = serde_json::Map::new();
                            c.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                            c.insert("required".to_string(), serde_json::Value::Bool(true));
                            serde_json::Value::Object(c)
                        });
                    });
                }
            }

            trace!("Finished building field map for unnamed variant");
            quote::quote! {
                let mut map = serde_json::Map::new();
                #(#field_inits)*
                variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
            }
        }
    };

    // We produce a single item (const) with the expansions so it parses cleanly.
    let final_ts = quote::quote! {
        #inner_ts
    };

    let code_str = final_ts.to_string();
    debug!("Generated code: {}", code_str);

    final_ts
}

#[cfg(test)]
mod verify_build_enum_variant_fields_map_with_justification {
    use super::*;

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
            expanded.contains("\"alpha\" . to_string ()")
                && expanded.contains("map . insert (\"alpha\" . to_string ()")
                && expanded.contains("alpha_confidence")
                && expanded.contains("alpha_justification"),
            "Expected expansions for alpha + justification/conf"
        );
        assert!(
            expanded.contains("\"beta\" . to_string ()")
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

    /// If you want to test the `build_enum_variant_fields_map_with_justification` directly:
    #[traced_test]
    fn test_build_enum_variant_fields_map_with_justification_for_named() {
        // Suppose your function is:
        //   fn build_enum_variant_fields_map_with_justification(
        //       variant: &syn::Variant,
        //       skip_self_just: bool,
        //       skip_child_just: bool
        //   ) -> proc_macro2::TokenStream
        //
        // We'll create a dummy variant with 2 named fields:
        let var: syn::Variant = syn::parse_quote! {
            NumericStuff {
                count: u32,
                label: String
            }
        };
        // For the sake of the test, skip_self_just=false, skip_child_just=false
        let ts = build_enum_variant_fields_map_with_justification(&var, false, false);

        //assert_tokens_parse_ok(&ts);

        // Check that "count:" is not turned into "count::"
        let code_str = ts.to_string();
        assert!(
            code_str.contains("count"),
            "Expected code to contain 'count' but not 'count::'"
        );
        assert!(!code_str.contains("count::"), "Should never produce 'count::'!");
    }

    /// Example test for verifying we don't break HashMap expansions
    #[traced_test]
    fn test_build_enum_variant_fields_map_with_justification_for_hashmap() {
        let var: syn::Variant = syn::parse_quote! {
            MapVariant {
                items: std::collections::HashMap<u8, String>
            }
        };
        let ts = build_enum_variant_fields_map_with_justification(&var, false, false);
        //assert_tokens_parse_ok(&ts);
        let code_str = ts.to_string();
        info!("code_str={}",code_str);
        assert!(
            code_str.contains("HashMap < u8 , String >"),
            "Should reference 'HashMap<u8, String>' in some form"
        );
        assert!(!code_str.contains("items::"), "No 'items::' path should appear!");
    }
}
