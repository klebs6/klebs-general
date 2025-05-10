// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_type.rs ]
crate::ix!();

pub fn emit_schema_for_type(
    ty:       &syn::Type,
    doc_lit:  proc_macro2::Literal,
    required: bool

) -> Option<proc_macro2::TokenStream> {

    let required_bool = if required { quote!(true) } else { quote!(false) };

    let generation_instructions = format!("{}", doc_lit);

    let type_str = quote!(#ty).to_string();
    trace!("emit_schema_for_type => required={} type={}", required, type_str);

    // 1) bool => "boolean"
    if is_bool(ty) {
        return Some(quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("boolean".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    // 2) String => "string"
    if is_string_type(ty) {
        return Some(quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    // 3) numeric => "number"
    if is_numeric(ty) {
        return Some(quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    // 4) Vec<T>
    if let Some(elem_ty) = extract_vec_inner(ty) {

        if is_numeric(elem_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_numbers".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else if is_bool(elem_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_booleans".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else if is_string_type(elem_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_strings".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else {
            // fallback => array_of + nested
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of".to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    let nested_t = <#elem_ty as AiJsonTemplate>::to_template();
                    obj.insert("item_template".to_string(), nested_t);
                    serde_json::Value::Object(obj)
                }
            });
        }
    }

    // 5) HashMap<K, V>
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(ty) {

        // Decide how to represent the key
        let map_key_schema = if is_bool(k_ty) {
            let err_msg = format!("Unsupported key type in HashMap<bool, _> for AiJsonTemplate");
            trace!("ERROR: {}", err_msg);
            let err = syn::Error::new(k_ty.span(), &err_msg);
            return Some(err.to_compile_error());
        } else if is_numeric(k_ty) {
            quote!("number")
        } else if is_string_type(k_ty) {
            quote!("string")
        } else {
            // fallback => treat as nested struct/enum
            quote!("nested_struct_or_enum")
        };

        if is_numeric(v_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("map_of_numbers".to_string()));
                    obj.insert("map_key_type".to_string(), serde_json::Value::String(#map_key_schema.to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else if is_bool(v_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("map_of_booleans".to_string()));
                    obj.insert("map_key_type".to_string(), serde_json::Value::String(#map_key_schema.to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else if is_string_type(v_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("map_of_strings".to_string()));
                    obj.insert("map_key_type".to_string(), serde_json::Value::String(#map_key_schema.to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else {
            // fallback => map_of + nested template for the value
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("map_of".to_string()));
                    obj.insert("map_key_type".to_string(), serde_json::Value::String(#map_key_schema.to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

                    let nested_val = <#v_ty as AiJsonTemplate>::to_template();
                    obj.insert("map_value_template".to_string(), nested_val);

                    serde_json::Value::Object(obj)
                }
            });
        }
    }

    Some(quote! {
        {
            let nested = <#ty as AiJsonTemplate>::to_template();
            let mut obj = serde_json::Map::new();

            let nested_as_obj = nested.as_object();
            let nested_type_str = if let Some(o) = nested_as_obj {
                if o.contains_key("enum_name") {
                    "nested_enum"
                } else if o.contains_key("struct_name") {
                    "nested_struct"
                } else if o.contains_key("type") && o["type"] == "complex_enum" {
                    "nested_enum"
                } else {
                    "nested_struct"
                }
            } else {
                "nested_struct"
            };

            obj.insert("type".to_string(), serde_json::Value::String(nested_type_str.to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            obj.insert("nested_template".to_string(), nested);

            serde_json::Value::Object(obj)
        }
    })
}
