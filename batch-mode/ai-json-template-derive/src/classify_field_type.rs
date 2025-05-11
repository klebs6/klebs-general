// ---------------- [ File: ai-json-template-derive/src/classify_field_type.rs ]
crate::ix!();

pub fn classify_field_type(ty: &syn::Type, doc_str: &str) -> Option<proc_macro2::TokenStream> {
    tracing::trace!("classify_field_type => doc_str={:?}, type=? => Checking type for AiJsonTemplate", doc_str);

    let doc_lit = proc_macro2::Literal::string(doc_str.trim());

    // 1) If it's an Option<T>, treat T as not required
    if let Some(inner_ty) = extract_option_inner(ty) {
        tracing::trace!("Field is Option<...> => required=false");
        return emit_schema_for_type(inner_ty, doc_lit, false);
    }

    // Otherwise required=true
    tracing::trace!("Field is not an Option => required=true");
    emit_schema_for_type(ty, doc_lit, true)
}

/// We restore the logic that:
///   - If a HashMap has a numeric/string key => "map_key_template" = "number"/"string"
///   - If the key is custom => call AiJsonTemplateWithJustification for the key if not skip_child_just
///   - If the value is a builtin => "boolean"/"number"/"string"
///   - If the value is custom => "AiJsonTemplateWithJustification" unless skip_child_just
/// This is exactly what your old tests rely upon.
pub fn classify_field_type_for_child(
    field_ty: &syn::Type,
    doc_str: &str,
    required: bool,
    skip_child_just: bool,
) -> Option<proc_macro2::TokenStream> {
    let doc_lit = proc_macro2::Literal::string(doc_str.trim());
    let required_bool = if required { quote::quote!(true) } else { quote::quote!(false) };

    // If Option<T> => treat T as not required
    if let Some(inner) = extract_option_inner(field_ty) {
        let snippet = classify_field_type_for_child(inner, doc_str, false, skip_child_just)?;
        return Some(quote::quote!({
            #snippet
        }));
    }

    // If Vec<T> => array_of
    if let Some(elem) = extract_vec_inner(field_ty) {
        let item_snippet = classify_field_type_for_child(elem, doc_str, true, skip_child_just)?;
        return Some(quote::quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                obj.insert("item_template".to_string(), #item_snippet);
                serde_json::Value::Object(obj)
            }
        });
    }

    // If HashMap<K, V>
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(field_ty) {
        // If key=bool => error out
        if is_bool(k_ty) {
            let err_msg = format!("Unsupported key type in HashMap<bool, _> for AiJsonTemplateWithJustification");
            let e = syn::Error::new(k_ty.span(), &err_msg);
            return Some(e.to_compile_error());
        }

        let key_schema = if is_numeric(k_ty) {
            quote::quote!(serde_json::Value::String("number".to_string()))
        } else if is_string_type(k_ty) {
            quote::quote!(serde_json::Value::String("string".to_string()))
        } else {
            // fallback => treat key as nested
            let child_expr = if skip_child_just {
                quote::quote!(<#k_ty as AiJsonTemplate>::to_template())
            } else {
                quote::quote!(<#k_ty as AiJsonTemplateWithJustification>::to_template_with_justification())
            };
            quote::quote! {
                {
                    let mut k_obj = serde_json::Map::new();
                    k_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
                    k_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    k_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    let nested_k = #child_expr;
                    k_obj.insert("nested_template".to_string(), nested_k);
                    serde_json::Value::Object(k_obj)
                }
            }
        };

        let val_schema = if is_bool(v_ty) {
            quote::quote!(serde_json::Value::String("boolean".to_string()))
        } else if is_numeric(v_ty) {
            quote::quote!(serde_json::Value::String("number".to_string()))
        } else if is_string_type(v_ty) {
            quote::quote!(serde_json::Value::String("string".to_string()))
        } else {
            // fallback => treat value as nested
            let child_expr = if skip_child_just {
                quote::quote!(<#v_ty as AiJsonTemplate>::to_template())
            } else {
                quote::quote!(<#v_ty as AiJsonTemplateWithJustification>::to_template_with_justification())
            };
            quote::quote! {
                {
                    let mut v_obj = serde_json::Map::new();
                    v_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
                    v_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    v_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    let nested_v = #child_expr;
                    v_obj.insert("nested_template".to_string(), nested_v);
                    serde_json::Value::Object(v_obj)
                }
            }
        };

        return Some(quote::quote! {
            {
                let mut map_obj = serde_json::Map::new();
                map_obj.insert("type".to_string(), serde_json::Value::String("map_of".to_string()));
                map_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                map_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                map_obj.insert("map_key_template".to_string(), #key_schema);
                map_obj.insert("map_value_template".to_string(), #val_schema);
                serde_json::Value::Object(map_obj)
            }
        });
    }

    // built-in bool => "boolean"
    if is_bool(field_ty) {
        return Some(quote::quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("boolean".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    // built-in string => "string"
    if is_string_type(field_ty) {
        return Some(quote::quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    // built-in numeric => "number"
    if is_numeric(field_ty) {
        return Some(quote::quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    // otherwise => treat as nested struct or enum => call AiJsonTemplateWithJustification (unless skip)
    let child_expr = if skip_child_just {
        quote::quote!(<#field_ty as AiJsonTemplate>::to_template())
    } else {
        quote::quote!(<#field_ty as AiJsonTemplateWithJustification>::to_template_with_justification())
    };
    Some(quote::quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

            let nested = #child_expr;
            obj.insert("nested_template".to_string(), nested);
            serde_json::Value::Object(obj)
        }
    })
}
