// ---------------- [ File: src/classify_field_type.rs ]
crate::ix!();

pub fn classify_field_type(ty: &syn::Type, doc_str: &str) -> Option<proc_macro2::TokenStream> {
    trace!("classify_field_type => doc_str={:?}, ty=?", doc_str);
    let ty_str = quote!(#ty).to_string().replace(' ', "");
    let doc_lit = proc_macro2::Literal::string(doc_str);

    match ty_str.as_str() {
        "String" => {
            trace!("Field is a String. Required = true");
            Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.trim().to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(true));
                    serde_json::Value::Object(obj)
                }
            })
        },
        "Vec<String>" => {
            trace!("Field is a Vec<String>. Required = true");
            Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_strings".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.trim().to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(true));
                    serde_json::Value::Object(obj)
                }
            })
        },
        "Option<String>" => {
            trace!("Field is an Option<String>. Required = false");
            Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.trim().to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(false));
                    serde_json::Value::Object(obj)
                }
            })
        },
        _ => {
            // assume it's a nested struct implementing AiJsonTemplate
            trace!("Treating as nested struct => AiJsonTemplate");
            Some(quote! {
                {
                    let nested = <#ty as AiJsonTemplate>::to_template();
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("nested_struct".to_string()));
                    obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.trim().to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(true));
                    obj.insert("nested_template".to_string(), nested);
                    serde_json::Value::Object(obj)
                }
            })
        }
    }
}
