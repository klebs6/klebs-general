// ---------------- [ File: src/classify_field_type.rs ]
crate::ix!();

pub fn classify_field_type(ty: &syn::Type, doc_str: &str) -> Option<proc_macro2::TokenStream> {
    trace!("classify_field_type => doc_str={:?}, ty=?", doc_str);

    let doc_lit = proc_macro2::Literal::string(doc_str.trim());

    /// Returns `Some(inner_type)` if `ty` is `Option<inner_type>`.
    fn extract_option_inner(ty: &syn::Type) -> Option<&syn::Type> {
        if let syn::Type::Path(type_path) = ty {
            // Must have exactly 1 segment, named "Option", with 1 generic arg.
            if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
                let seg = &type_path.path.segments[0];
                if seg.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(ref bracketed) = seg.arguments {
                        if bracketed.args.len() == 1 {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = bracketed.args.first() {
                                return Some(inner_ty);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Is this exactly `String`?
    fn is_string_type(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
                return type_path.path.segments[0].ident == "String";
            }
        }
        false
    }

    /// Is this exactly `Vec<String>`?
    fn is_vec_of_string_type(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
                let seg = &type_path.path.segments[0];
                if seg.ident == "Vec" {
                    if let syn::PathArguments::AngleBracketed(ref bracketed) = seg.arguments {
                        if bracketed.args.len() == 1 {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = bracketed.args.first() {
                                return is_string_type(inner_ty);
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Primitive numeric check, so we can return None for e.g. i32, i64, etc.
    /// (We want to yield a compile error "Unsupported field type...")
    fn is_primitive_numeric(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            // If there's exactly one segment, check its ident
            if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
                let ident_str = type_path.path.segments[0].ident.to_string();
                // We'll treat a few common numeric idents as unsupported:
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

    // -------------------------------------------
    // 1) Check if this is an Option<T>
    // -------------------------------------------
    if let Some(inner_ty) = extract_option_inner(ty) {
        trace!("Field is Option<...> => required=false");
        if is_string_type(inner_ty) {
            trace!("Inner is String => Option<String>");
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(false));
                    serde_json::Value::Object(obj)
                }
            });
        } else if is_vec_of_string_type(inner_ty) {
            trace!("Inner is Vec<String> => Option<Vec<String>>");
            return Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_strings".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(false));
                    serde_json::Value::Object(obj)
                }
            });
        } else if is_primitive_numeric(inner_ty) {
            warn!("Inner is a numeric type => unsupported in AiJsonTemplate for now.");
            return None;
        } else {
            trace!("Inner is unrecognized => treat as nested AiJsonTemplate => Option<nested>");
            return Some(quote! {
                {
                    let nested = <#inner_ty as AiJsonTemplate>::to_template();
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("nested_struct".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(false));
                    obj.insert("nested_template".to_string(), nested);
                    serde_json::Value::Object(obj)
                }
            });
        }
    }

    // -------------------------------------------
    // 2) Not an Option => required = true
    // -------------------------------------------
    trace!("Field is not an Option => required=true");
    if is_string_type(ty) {
        trace!("Field is a String => required=true");
        Some(quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(true));
                serde_json::Value::Object(obj)
            }
        })
    } else if is_vec_of_string_type(ty) {
        trace!("Field is a Vec<String> => required=true");
        Some(quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of_strings".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(true));
                serde_json::Value::Object(obj)
            }
        })
    } else if is_primitive_numeric(ty) {
        warn!("Field is a numeric type => unsupported in AiJsonTemplate for now.");
        None
    } else {
        trace!("Treating field as a nested struct => required=true");
        Some(quote! {
            {
                let nested = <#ty as AiJsonTemplate>::to_template();
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("nested_struct".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(true));
                obj.insert("nested_template".to_string(), nested);
                serde_json::Value::Object(obj)
            }
        })
    }
}
