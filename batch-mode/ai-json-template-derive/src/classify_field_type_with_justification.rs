// ---------------- [ File: ai-json-template-derive/src/classify_field_type_with_justification.rs ]
crate::ix!();

pub fn classify_field_type_with_justification(
    ty: &syn::Type,
    doc_str: &str,
    required: bool,
) -> Option<proc_macro2::TokenStream> {
    let required_bool = if required { quote::quote!(true) } else { quote::quote!(false) };
    let doc_lit = proc_macro2::Literal::string(doc_str.trim());

    // 1) If it's Option<T>, handle T as not required
    if let Some(inner) = extract_option_inner(ty) {
        let child = classify_field_type_with_justification(inner, doc_str, false)?;
        return Some(quote::quote!({ #child }));
    }

    // 2) If it's Vec<T>, handle array_of
    if let Some(elem_ty) = extract_vec_inner(ty) {
        let item_schema = classify_field_type_with_justification(elem_ty, doc_str, true)?;
        return Some(quote::quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                obj.insert("item_template".to_string(), #item_schema);
                serde_json::Value::Object(obj)
            }
        });
    }

    // 3) If it's HashMap<K, V>, decide how to represent K and V
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(ty) {
        // Key handling
        let key_schema = if is_bool(k_ty) {
            // Hard error if key is bool
            let err_msg = format!("Unsupported key type in HashMap<bool, _> for AiJsonTemplateWithJustification");
            let err = syn::Error::new(k_ty.span(), &err_msg);
            return Some(err.to_compile_error());
        } else if is_numeric(k_ty) {
            quote::quote! { serde_json::Value::String("number".to_string()) }
        } else if is_string_type(k_ty) {
            quote::quote! { serde_json::Value::String("string".to_string()) }
        } else {
            // Treat key as a nested struct/enum
            quote::quote! {
                {
                    let mut k_obj = serde_json::Map::new();
                    k_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
                    k_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    k_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));

                    // If you do want child-level justification for the key, you can call:
                    let nested_k = <#k_ty as AiJsonTemplateWithJustification>::to_template_with_justification();
                    k_obj.insert("nested_template".to_string(), nested_k);
                    serde_json::Value::Object(k_obj)
                }
            }
        };

        // Value handling
        let val_schema = if is_bool(v_ty) {
            quote::quote! { serde_json::Value::String("boolean".to_string()) }
        } else if is_numeric(v_ty) {
            quote::quote! { serde_json::Value::String("number".to_string()) }
        } else if is_string_type(v_ty) {
            quote::quote! { serde_json::Value::String("string".to_string()) }
        } else {
            // Non-primitive => nested
            quote::quote! {
                {
                    let mut v_obj = serde_json::Map::new();
                    v_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
                    v_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    v_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));

                    let nested_v = <#v_ty as AiJsonTemplateWithJustification>::to_template_with_justification();
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

    // 4) Builtin bool => "boolean"
    if is_bool(ty) {
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

    // 5) Builtin String => "string"
    if is_string_type(ty) {
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

    // 6) Builtin numeric => "number"
    if is_numeric(ty) {
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

    // 7) Otherwise => nested struct or enum => call AiJsonTemplateWithJustification
    Some(quote::quote! {
        {
            let mut nested_obj = serde_json::Map::new();
            nested_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
            nested_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            nested_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

            let nested = <#ty as AiJsonTemplateWithJustification>::to_template_with_justification();
            nested_obj.insert("nested_template".to_string(), nested);
            serde_json::Value::Object(nested_obj)
        }
    })
}
