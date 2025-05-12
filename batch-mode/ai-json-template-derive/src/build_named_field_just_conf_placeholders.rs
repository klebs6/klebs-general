crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn build_named_field_just_conf_placeholders(field_name_str: &str) -> proc_macro2::TokenStream {
    tracing::trace!(
        "build_named_field_just_conf_placeholders: field='{}'",
        field_name_str
    );

    let justify_key = format!("{}_justification", field_name_str);
    let conf_key    = format!("{}_confidence",    field_name_str);

    quote::quote! {
        {
            let justify_key = format!(#justify_key, #field_name_str);
            let conf_key    = format!(#conf_key, #field_name_str);

            let mut just_obj = serde_json::Map::new();
            just_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            just_obj.insert("required".to_string(), serde_json::Value::Bool(true));
            map.insert(justify_key, serde_json::Value::Object(just_obj));

            let mut conf_obj = serde_json::Map::new();
            conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
            conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
            map.insert(conf_key, serde_json::Value::Object(conf_obj));
        }
    }
}
