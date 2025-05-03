// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_type.rs ]
crate::ix!();

pub fn emit_schema_for_type(
    ty: &syn::Type,
    doc_lit: proc_macro2::Literal,
    required: bool
) -> Option<proc_macro2::TokenStream> {
    let required_bool = if required { quote!(true) } else { quote!(false) };

    // We'll build a small, combined "generation_instructions" string that includes both the doc
    // comments (doc_lit) and universal disclaimers:
    //
    //  - Must be valid JSON of the correct type (no quotes around numbers).
    //  - If optional and not used, set it to null.
    //  - Do not add extra fields or rename them.
    //  - For an enum, pick exactly one variant.
    //  - Etc.
    //
    // We keep it short but explicit, so the AI is forced to produce a well-formed result.

    let disclaimers = "\nIMPORTANT:\n\
        1) Provide a real JSON value (no string quotes around numbers).\n\
        2) If this field is optional and you choose not to fill it, set it to null.\n\
        3) Do not add extra fields or rename existing fields.\n\
        4) If this is an enum field, pick exactly one variant.\n\
    ";
    let merged_instructions = format!("{}{}", doc_lit, disclaimers);

    let type_str = quote!(#ty).to_string();
    tracing::trace!("emit_schema_for_type => required={} type={}", required, type_str);

    // 1) bool => "boolean"
    if is_bool(ty) {
        return Some(quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("boolean".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#merged_instructions.to_string()));
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
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#merged_instructions.to_string()));
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
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#merged_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    // 4) Vec<T>
    if let Some(elem_ty) = extract_vec_inner(ty) {
        // For arrays, we disclaim that it must be a real JSON array. No extra fields, no single scalar.
        let disclaimers_array = format!(
            "{}\nFor arrays: supply a JSON list [ ... ] of the correct item type.\n",
            merged_instructions
        );

        if is_numeric(elem_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_numbers".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#disclaimers_array.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else if is_bool(elem_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_booleans".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#disclaimers_array.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else if is_string_type(elem_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_strings".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#disclaimers_array.to_string()));
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
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#disclaimers_array.to_string()));
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
        // Additional disclaimers about map => must be a JSON object with string keys
        let disclaimers_map = format!(
            "{}\nFor map fields: Provide a JSON object {{ \"key_as_string\": <value>, ... }}.\nKeys must be string in JSON.\n",
            merged_instructions
        );

        // Decide how to represent the key
        let map_key_schema = if is_bool(k_ty) {
            let err_msg = format!("Unsupported key type in HashMap<bool, _> for AiJsonTemplate");
            tracing::trace!("ERROR: {}", err_msg);
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
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#disclaimers_map.to_string()));
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
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#disclaimers_map.to_string()));
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
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#disclaimers_map.to_string()));
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
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#disclaimers_map.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

                    let nested_val = <#v_ty as AiJsonTemplate>::to_template();
                    obj.insert("map_value_template".to_string(), nested_val);

                    serde_json::Value::Object(obj)
                }
            });
        }
    }

    // 6) fallback => treat as AiJsonTemplate
    let disclaimers_nested = format!(
        "{}\nFor nested struct/enum fields, pick exactly one variant if it's an enum, and be sure to fill all required subfields.\n",
        merged_instructions
    );

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
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#disclaimers_nested.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            obj.insert("nested_template".to_string(), nested);

            serde_json::Value::Object(obj)
        }
    })
}
