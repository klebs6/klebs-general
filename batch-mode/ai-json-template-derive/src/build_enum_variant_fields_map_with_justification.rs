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
        // Unit => no fields
        syn::Fields::Unit => {
            trace!("Unit variant => no fields");
            quote::quote! { /* no fields */ }
        }

        // Named (struct) variant => produce a set of fields => each gets child schema + justification/conf placeholders.
        syn::Fields::Named(named) => {
            debug!("Named variant => building child expansions for each named field");
            let mut field_inits = Vec::new();

            for field in &named.named {
                let f_ident = match &field.ident {
                    Some(id) => id,
                    None => continue,
                };
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
        }

        // Unnamed (tuple) variant => produce placeholders for each field_0, field_1, ...
        syn::Fields::Unnamed(unnamed) => {
            debug!("Tuple variant => building child expansions for each tuple field_i");
            let mut field_inits = Vec::new();

            for (i, field) in unnamed.unnamed.iter().enumerate() {
                let fname = format!("field_{}", i);
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
                // Now also produce justification/conf placeholders if not skip_f_self
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
    }
}
