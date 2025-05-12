// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_hashmap.rs ]
crate::ix!();

pub fn emit_schema_for_hashmap(
    k_ty: &syn::Type,
    v_ty: &syn::Type,
    generation_instructions: &str,
    required_bool: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    trace!("emit_schema_for_hashmap invoked");

    // Resolve map key type
    let map_key_schema = match resolve_map_key_type(k_ty) {
        Ok(key_str) => {
            quote! { serde_json::Value::String(#key_str.to_string()) }
        }
        Err(err_ts) => {
            // Key is bool => unsupported
            debug!("HashMap<bool, _> => returning compile_error");
            return err_ts;
        }
    };

    // Now handle the value type
    if is_numeric(v_ty) {
        trace!("HashMap => map_of_numbers");
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("map_of_numbers".to_string()));
                obj.insert("map_key_type".to_string(), #map_key_schema);
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else if is_bool(v_ty) {
        trace!("HashMap => map_of_booleans");
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("map_of_booleans".to_string()));
                obj.insert("map_key_type".to_string(), #map_key_schema);
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else if is_string_type(v_ty) {
        trace!("HashMap => map_of_strings");
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("map_of_strings".to_string()));
                obj.insert("map_key_type".to_string(), #map_key_schema);
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else {
        trace!("HashMap => map_of + nested map_value_template");
        quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("map_of".to_string()));
                obj.insert("map_key_type".to_string(), #map_key_schema);
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

                let nested_val = <#v_ty as AiJsonTemplate>::to_template();
                obj.insert("map_value_template".to_string(), nested_val);

                serde_json::Value::Object(obj)
            }
        }
    }
}
