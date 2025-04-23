// ---------------- [ File: ai-json-template-derive/src/classify_field_type.rs ]
crate::ix!();

pub fn classify_field_type(ty: &syn::Type, doc_str: &str) -> Option<proc_macro2::TokenStream> {
    trace!("classify_field_type => doc_str={:?}, type=? => Checking type for AiJsonTemplate", doc_str);

    let doc_lit = proc_macro2::Literal::string(doc_str.trim());

    // 1) If it's an Option<T>, treat T as not required
    if let Some(inner_ty) = extract_option_inner(ty) {
        trace!("Field is Option<...> => required=false");
        return emit_schema_for_type(inner_ty, doc_lit, false);
    }

    // Otherwise required=true
    trace!("Field is not an Option => required=true");
    emit_schema_for_type(ty, doc_lit, true)
}

/// If `ty` is Option<T>, returns Some(&T). Else None.
fn extract_option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
            let seg = &type_path.path.segments[0];
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ref bracketed) = seg.arguments {
                    if bracketed.args.len() == 1 {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = bracketed.args.first() {
                            trace!("Detected Option<{}>", quote!(#inner_ty).to_string());
                            return Some(inner_ty);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Check if `bool`.
fn is_bool(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        tp.path.segments.len() == 1 && tp.path.segments[0].ident == "bool"
    } else {
        false
    }
}

/// Check if `String`.
fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        tp.path.segments.len() == 1 && tp.path.segments[0].ident == "String"
    } else {
        false
    }
}

/// Check if numeric type (u8,u16,u32,u64,i8,i16,i32,i64,usize,isize,f32,f64).
fn is_numeric(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        if tp.path.segments.len() == 1 {
            let ident_str = tp.path.segments[0].ident.to_string();
            return matches!(
                ident_str.as_str(),
                "i8" | "i16" | "i32" | "i64" | "isize"
                | "u8" | "u16" | "u32" | "u64" | "usize"
                | "f32" | "f64"
            );
        }
    }
    false
}

/// Check if `Vec<T>`. Return T if so.
fn extract_vec_inner(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(tp) = ty {
        if tp.path.segments.len() == 1 && tp.path.segments[0].ident == "Vec" {
            if let syn::PathArguments::AngleBracketed(ref bracketed) = tp.path.segments[0].arguments {
                if bracketed.args.len() == 1 {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = bracketed.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

/// Return true if K is recognized numeric or String.
fn is_simple_key_type(k: &syn::Type) -> bool {
    is_numeric(k) || is_string_type(k)
}

/// If `ty` is `HashMap<K, V>`, returns (K, V).
fn extract_hashmap_inner(ty: &syn::Type) -> Option<(&syn::Type, &syn::Type)> {
    if let syn::Type::Path(tp) = ty {
        if tp.path.segments.len() == 1 && tp.path.segments[0].ident == "HashMap" {
            if let syn::PathArguments::AngleBracketed(ref bracketed) = tp.path.segments[0].arguments {
                if bracketed.args.len() == 2 {
                    let mut args_iter = bracketed.args.iter();
                    if let (Some(syn::GenericArgument::Type(k_ty)), Some(syn::GenericArgument::Type(v_ty))) =
                        (args_iter.next(), args_iter.next())
                    {
                        return Some((k_ty, v_ty));
                    }
                }
            }
        }
    }
    None
}

/// Emit the JSON schema snippet for the given type. 
fn emit_schema_for_type(
    ty: &syn::Type,
    doc_lit: proc_macro2::Literal,
    required: bool
) -> Option<proc_macro2::TokenStream> {
    let required_bool = if required { quote!(true) } else { quote!(false) };
    let type_str = quote!(#ty).to_string();
    trace!("emit_schema_for_type => required={} type={}", required, type_str);

    // 1) bool => "boolean"
    if is_bool(ty) {
        return Some(quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("boolean".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
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
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
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
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    // 4) Vec<T>
    if let Some(elem_ty) = extract_vec_inner(ty) {
        // Check if T is numeric, bool, or string => produce array_of_numbers, array_of_booleans, etc.
        if is_numeric(elem_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_numbers".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else if is_bool(elem_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_booleans".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else if is_string_type(elem_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_strings".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else {
            // fallback => array_of nested type => call AiJsonTemplate for T
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
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
        // We only handle numeric or string keys
        if !is_simple_key_type(k_ty) {
            let err_msg = format!("Unsupported key type in HashMap<{}, _> for AiJsonTemplate", quote!(#k_ty));
            let err = syn::Error::new(k_ty.span(), &err_msg);
            trace!("ERROR: {}", err_msg);
            return Some(err.to_compile_error());
        }

        // Decide how to treat V
        let map_key_schema = if is_string_type(k_ty) {
            quote!("string")
        } else {
            // numeric
            quote!("number")
        };

        // If V is numeric => "map_of_numbers", if V is bool => "map_of_booleans", etc.
        if is_numeric(v_ty) {
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("map_of_numbers".to_string()));
                    obj.insert("map_key_type".to_string(), serde_json::Value::String(#map_key_schema.to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
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
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
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
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    serde_json::Value::Object(obj)
                }
            });
        } else {
            // fallback => map_of + nested template
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("map_of".to_string()));
                    obj.insert("map_key_type".to_string(), serde_json::Value::String(#map_key_schema.to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

                    let nested_val = <#v_ty as AiJsonTemplate>::to_template();
                    obj.insert("map_value_template".to_string(), nested_val);

                    serde_json::Value::Object(obj)
                }
            });
        }
    }

    // 6) fallback => treat as AiJsonTemplate
    Some(quote! {
        {
            let nested = <#ty as AiJsonTemplate>::to_template();
            let mut obj = serde_json::Map::new();

            // attempt to detect "enum" vs. "struct"
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
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            obj.insert("nested_template".to_string(), nested);

            serde_json::Value::Object(obj)
        }
    })
}
