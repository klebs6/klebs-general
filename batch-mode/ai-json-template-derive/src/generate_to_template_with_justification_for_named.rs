// ---------------- [ File: ai-json-template-derive/src/generate_to_template_with_justification_for_named.rs ]
crate::ix!();

/// Builds the `fn to_template_with_justification()` for a named struct,
/// inserting each field's child schema plus `_justification` & `_confidence`.
pub fn generate_to_template_with_justification_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    container_docs_str: &str
) -> proc_macro2::TokenStream {
    let mut field_inits = Vec::new();
    for field in &named_fields.named {
        let field_ident = field.ident.as_ref().unwrap();
        let field_name_str = field_ident.to_string();

        let doc_str = gather_doc_comments(&field.attrs).join("\n");
        let is_required = extract_option_inner(&field.ty).is_none();
        let skip_self_just = is_justification_disabled_for_field(field);
        let skip_child_just = skip_self_just || is_justification_disabled_for_inner(field);

        // normal child schema
        if let Some(expr) = classify_field_type_for_child(&field.ty, &doc_str, is_required, skip_child_just) {
            field_inits.push(quote::quote! {
                map.insert(#field_name_str.to_string(), #expr);
            });
        }
        // add justification/conf placeholders if not skip_self_just
        if !skip_self_just {
            field_inits.push(quote::quote! {
                {
                    let justify_key = format!("{}_justification", #field_name_str);
                    let conf_key    = format!("{}_confidence", #field_name_str);

                    let mut just_obj = serde_json::Map::new();
                    just_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                    just_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                    map.insert(justify_key, serde_json::Value::Object(just_obj));

                    let mut conf_obj = serde_json::Map::new();
                    conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                    conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                    map.insert(conf_key, serde_json::Value::Object(conf_obj));
                }
            });
        }
    }

    quote::quote! {
        impl AiJsonTemplateWithJustification for #ty_ident {
            fn to_template_with_justification() -> serde_json::Value {
                let mut root = serde_json::Map::new();
                root.insert("struct_docs".to_string(), serde_json::Value::String(#container_docs_str.to_string()));
                root.insert("struct_name".to_string(), serde_json::Value::String(stringify!(#ty_ident).to_string()));
                root.insert("type".to_string(), serde_json::Value::String("struct".to_string()));
                root.insert("has_justification".to_string(), serde_json::Value::Bool(true));

                let mut map = serde_json::Map::new();
                #(#field_inits)*

                root.insert("fields".to_string(), serde_json::Value::Object(map));
                serde_json::Value::Object(root)
            }
        }
    }
}
