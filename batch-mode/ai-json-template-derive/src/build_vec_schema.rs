// ---------------- [ File: ai-json-template-derive/src/build_vec_schema.rs ]
crate::ix!();

/// Builds the schema for `Vec<T>`, returning a JSON "array_of" schema with `item_template`.
pub fn build_vec_schema(
    elem_ty: &syn::Type,
    required_bool: proc_macro2::TokenStream,
    doc_lit: proc_macro2::Literal
) -> Option<proc_macro2::TokenStream> {
    trace!("build_vec_schema => elem: {:?}", elem_ty);

    let item_schema = classify_field_type_with_justification(elem_ty, &doc_lit.to_string(), true)?;
    Some(quote::quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("array_of".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            obj.insert("item_template".to_string(), #item_schema);
            serde_json::Value::Object(obj)
        }
    })
}
