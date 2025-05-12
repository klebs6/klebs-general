crate::ix!();

/// Builds the schema for a `HashMap<K, V>`, handling special cases for K=bool, K=number, etc.
pub fn build_hashmap_schema(
    k_ty: &syn::Type,
    v_ty: &syn::Type,
    required_bool: proc_macro2::TokenStream,
    doc_lit: proc_macro2::Literal
) -> Option<proc_macro2::TokenStream> {
    trace!("build_hashmap_schema => K: {:?}, V: {:?}", k_ty, v_ty);

    // Key handling
    let key_schema = if is_bool(k_ty) {
        let err_msg = format!("Unsupported key type in HashMap<bool, _> for AiJsonTemplateWithJustification");
        let err = syn::Error::new(k_ty.span(), &err_msg);
        return Some(err.to_compile_error());
    } else if is_numeric(k_ty) {
        quote::quote! {
            serde_json::Value::String("number".to_string())
        }
    } else if is_string_type(k_ty) {
        quote::quote! {
            serde_json::Value::String("string".to_string())
        }
    } else {
        // Non-primitive key => treat as nested
        quote::quote! {
            {
                let mut k_obj = serde_json::Map::new();
                k_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
                k_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                k_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));

                let nested_k = <#k_ty as AiJsonTemplateWithJustification>::to_template_with_justification();
                k_obj.insert("nested_template".to_string(), nested_k);
                serde_json::Value::Object(k_obj)
            }
        }
    };

    // Value handling
    let val_schema = if is_bool(v_ty) {
        quote::quote! {
            serde_json::Value::String("boolean".to_string())
        }
    } else if is_numeric(v_ty) {
        quote::quote! {
            serde_json::Value::String("number".to_string())
        }
    } else if is_string_type(v_ty) {
        quote::quote! {
            serde_json::Value::String("string".to_string())
        }
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

    // Build final map_of
    Some(quote::quote! {
        {
            let mut map_obj = serde_json::Map::new();
            map_obj.insert("type".to_string(), serde_json::Value::String("map_of".to_string()));
            map_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            map_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            map_obj.insert("map_key_template".to_string(), #key_schema);
            map_obj.insert("map_value_template".to_string(), #val_schema);
            serde_json::Value::Object(map_obj)
        }
    })
}
