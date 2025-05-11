// ---------------- [ File: ai-json-template-derive/src/generate_to_template_with_justification_for_enum.rs ]
crate::ix!();

/// Builds the `fn to_template_with_justification() -> serde_json::Value` for an enum, 
/// giving each variant a `fields` section. If it’s a named variant, we produce `field_justification`
/// and `field_confidence` placeholders for each field. This matches the old approach used by your tests.
pub fn generate_to_template_with_justification_for_enum(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    container_docs_str: &str
) -> proc_macro2::TokenStream {
    let type_name_str = ty_ident.to_string();
    let mut variant_exprs = Vec::new();

    for var in &data_enum.variants {
        let var_name_str = var.ident.to_string();
        let var_docs = gather_doc_comments(&var.attrs).join("\n");
        let skip_self_just = is_justification_disabled_for_variant(var);
        let skip_child_just = skip_self_just || is_justification_disabled_for_inner_variant(var);

        let variant_kind_str = match var.fields {
            syn::Fields::Unit => "unit_variant",
            syn::Fields::Named(_) => "struct_variant",
            syn::Fields::Unnamed(_) => "tuple_variant",
        };

        // We'll gather expansions inside `variant_map`.
        let fields_insertion = match &var.fields {
            // -----------------------------------------------------------------------------
            // Unit variant => no fields
            // -----------------------------------------------------------------------------
            syn::Fields::Unit => quote::quote! {
                // no fields
            },

            // -----------------------------------------------------------------------------
            // Named (struct) variant => produce a set of fields => each gets child schema plus
            // justification/confidence placeholders. The old tests expect "name_justification", etc.
            // -----------------------------------------------------------------------------
            syn::Fields::Named(named) => {
                let mut field_inits = Vec::new();
                for field in &named.named {
                    let f_ident = field.ident.as_ref().unwrap();
                    let fname = f_ident.to_string();
                    let doc_str = gather_doc_comments(&field.attrs).join("\n");
                    let is_required = extract_option_inner(&field.ty).is_none();
                    let skip_f_self = is_justification_disabled_for_field(field);
                    let skip_f_child = skip_f_self || skip_child_just;

                    // The child's normal schema
                    if let Some(expr) = classify_field_type_for_child(&field.ty, &doc_str, is_required, skip_f_child) {
                        field_inits.push(quote::quote! {
                            map.insert(#fname.to_string(), #expr);
                        });
                    }
                    // Now also produce the justification/conf placeholders if not skip_f_self
                    if !skip_f_self {
                        field_inits.push(quote::quote! {
                            {
                                let justify_key = format!("{}_justification", #fname);
                                let conf_key    = format!("{}_confidence", #fname);
                                let mut just_obj = serde_json::Map::new();
                                just_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                                just_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                map.insert(justify_key, serde_json::Value::Object(just_obj));

                                let mut conf_obj = serde_json::Map::new();
                                conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                                conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                map.insert(conf_key, serde_json::Value::Object(conf_obj));
                            }
                        });
                    }
                }
                quote::quote! {
                    let mut map = serde_json::Map::new();
                    #(#field_inits)*
                    variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
                }
            },

            // -----------------------------------------------------------------------------
            // Unnamed (tuple) variant => produce placeholders for each field_0, field_1, ...
            // plus justification/conf if not skip_f_self.
            // -----------------------------------------------------------------------------
            syn::Fields::Unnamed(unnamed) => {
                let mut field_inits = Vec::new();
                for (i, field) in unnamed.unnamed.iter().enumerate() {
                    let fname = format!("field_{}", i);
                    let doc_str = gather_doc_comments(&field.attrs).join("\n");
                    let is_required = extract_option_inner(&field.ty).is_none();
                    let skip_f_self = is_justification_disabled_for_field(field);
                    let skip_f_child = skip_f_self || skip_child_just;

                    if let Some(expr) = classify_field_type_for_child(&field.ty, &doc_str, is_required, skip_f_child) {
                        field_inits.push(quote::quote! {
                            map.insert(#fname.to_string(), #expr);
                        });
                    }
                    if !skip_f_self {
                        field_inits.push(quote::quote! {
                            {
                                let justify_key = format!("{}_justification", #fname);
                                let conf_key    = format!("{}_confidence", #fname);
                                let mut just_obj = serde_json::Map::new();
                                just_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                                just_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                map.insert(justify_key, serde_json::Value::Object(just_obj));

                                let mut conf_obj = serde_json::Map::new();
                                conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                                conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                map.insert(conf_key, serde_json::Value::Object(conf_obj));
                            }
                        });
                    }
                }
                quote::quote! {
                    let mut map = serde_json::Map::new();
                    #(#field_inits)*
                    variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
                }
            }
        };

        // If not skip_self_just, produce top-level "variant_justification" & "variant_confidence"
        let variant_just_conf = if !skip_self_just {
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
            quote::quote! {}
        };

        let expr = quote::quote! {
            {
                let mut variant_map = serde_json::Map::new();
                variant_map.insert("variant_name".to_string(), serde_json::Value::String(#var_name_str.to_string()));
                variant_map.insert("variant_docs".to_string(), serde_json::Value::String(#var_docs.to_string()));
                variant_map.insert("variant_type".to_string(), serde_json::Value::String(#variant_kind_str.to_string()));

                #fields_insertion
                #variant_just_conf

                serde_json::Value::Object(variant_map)
            }
        };
        variant_exprs.push(expr);
    }

    // The final expansions => merges the old AiJsonTemplate output plus justification disclaimers
    quote::quote! {
        impl AiJsonTemplateWithJustification for #ty_ident {
            fn to_template_with_justification() -> serde_json::Value {
                let base = <#ty_ident as AiJsonTemplate>::to_template();
                let mut root_map = if let Some(obj) = base.as_object() {
                    obj.clone()
                } else {
                    serde_json::Map::new()
                };

                // Insert disclaimers
                root_map.insert("has_justification".to_string(), serde_json::Value::Bool(true));
                if root_map.contains_key("enum_docs") {
                    if let Some(serde_json::Value::String(sdoc)) = root_map.get_mut("enum_docs") {
                        *sdoc = format!("{}\n{}", *sdoc, #container_docs_str);
                    }
                } else {
                    root_map.insert("enum_docs".to_string(), serde_json::Value::String(#container_docs_str.to_string()));
                }

                // Overwrite or fill "variants"
                let variants_vec = vec![ #(#variant_exprs),* ];
                root_map.insert("variants".to_string(), serde_json::Value::Array(variants_vec));

                serde_json::Value::Object(root_map)
            }
        }
    }
}
