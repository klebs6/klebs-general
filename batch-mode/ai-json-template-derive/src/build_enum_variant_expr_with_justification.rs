crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_enum_variant_expr_with_justification(
    variant: &syn::Variant,
    variant_name_str: &str,
    variant_docs: &str,
    variant_kind_str: &str,
    fields_insertion_ts: proc_macro2::TokenStream,
    skip_self_just: bool
) -> proc_macro2::TokenStream {
    trace!(
        "Building enum variant expr with justification => variant: '{}', kind: '{}'",
        variant_name_str,
        variant_kind_str
    );

    // Optionally add top-level "variant_justification" & "variant_confidence"
    let top_level_just_conf = if !skip_self_just {
        debug!("Adding top-level variant_justification and variant_confidence for '{}'", variant_name_str);
        quote::quote! {
            {
                let mut j_obj = serde_json::Map::new();
                j_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                j_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                variant_map.insert("variant_justification".to_string(), serde_json::Value::Object(j_obj));

                let mut c_obj = serde_json::Map::new();
                c_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                c_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                variant_map.insert("variant_confidence".to_string(), serde_json::Value::Object(c_obj));
            }
        }
    } else {
        trace!("skip_self_just=true => No variant_justification/confidence for '{}'", variant_name_str);
        quote::quote! {}
    };

    quote::quote! {
        {
            let mut variant_map = serde_json::Map::new();
            variant_map.insert("variant_name".to_string(), serde_json::Value::String(#variant_name_str.to_string()));
            variant_map.insert("variant_docs".to_string(), serde_json::Value::String(#variant_docs.to_string()));
            variant_map.insert("variant_type".to_string(), serde_json::Value::String(#variant_kind_str.to_string()));

            #fields_insertion_ts
            #top_level_just_conf

            serde_json::Value::Object(variant_map)
        }
    }
}
