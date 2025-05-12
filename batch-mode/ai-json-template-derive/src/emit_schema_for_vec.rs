crate::ix!();

pub fn emit_schema_for_vec(
    elem_ty: &syn::Type,
    generation_instructions: &str,
    required_bool: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    trace!("emit_schema_for_vec invoked");
    if is_numeric(elem_ty) {
        trace!("Array of numeric => array_of_numbers");
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of_numbers".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else if is_bool(elem_ty) {
        trace!("Array of bool => array_of_booleans");
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of_booleans".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else if is_string_type(elem_ty) {
        trace!("Array of string => array_of_strings");
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of_strings".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else {
        trace!("Array of nested => array_of + item_template");
        quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of".to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

                let nested_t = <#elem_ty as AiJsonTemplate>::to_template();
                obj.insert("item_template".to_string(), nested_t);

                serde_json::Value::Object(obj)
            }
        }
    }
}
