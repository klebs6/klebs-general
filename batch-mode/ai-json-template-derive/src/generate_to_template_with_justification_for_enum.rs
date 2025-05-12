// ---------------- [ File: ai-json-template-derive/src/generate_to_template_with_justification_for_enum.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_to_template_with_justification_for_enum(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    container_docs_str: &str
) -> proc_macro2::TokenStream {
    let type_name_str = ty_ident.to_string();
    trace!(
        "Starting generate_to_template_with_justification_for_enum for '{}'",
        type_name_str
    );

    let mut variant_exprs = Vec::new();
    for var in &data_enum.variants {
        let var_name_str = var.ident.to_string();
        let var_docs = gather_doc_comments(&var.attrs).join("\n");

        // Should we skip top-level variant_justification/conf?
        let skip_self_just = is_justification_disabled_for_variant(var);
        let skip_child_just = skip_self_just || is_justification_disabled_for_inner_variant(var);

        let variant_kind_str = match var.fields {
            syn::Fields::Unit => "unit_variant",
            syn::Fields::Named(_) => "struct_variant",
            syn::Fields::Unnamed(_) => "tuple_variant",
        };

        // Build the snippet that populates the `fields` map
        let fields_insertion_ts = build_enum_variant_fields_map_with_justification(
            var,
            skip_self_just,
            skip_child_just
        );

        // Build the final variant expr
        let variant_expr_ts = build_enum_variant_expr_with_justification(
            var,
            &var_name_str,
            &var_docs,
            variant_kind_str,
            fields_insertion_ts,
            skip_self_just
        );

        variant_exprs.push(variant_expr_ts);
    }

    let expanded = quote::quote! {
        impl AiJsonTemplateWithJustification for #ty_ident {
            fn to_template_with_justification() -> serde_json::Value {
                let base = <#ty_ident as AiJsonTemplate>::to_template();
                let mut root_map = if let Some(obj) = base.as_object() {
                    debug!("Cloning existing object from AiJsonTemplate for '{}'", #type_name_str);
                    obj.clone()
                } else {
                    debug!("No valid object from AiJsonTemplate => using empty object for '{}'", #type_name_str);
                    serde_json::Map::new()
                };

                // Insert disclaimers
                root_map.insert("has_justification".to_string(), serde_json::Value::Bool(true));
                if root_map.contains_key("enum_docs") {
                    if let Some(serde_json::Value::String(sdoc)) = root_map.get_mut("enum_docs") {
                        *sdoc = format!("{}\n{}", *sdoc, #container_docs_str);
                    }
                } else {
                    root_map.insert("enum_docs".to_string(), serde_json::Value::String(#container_docs_str.to_string()));
                }

                // Overwrite or fill "variants"
                let variants_vec = vec![ #(#variant_exprs),* ];
                root_map.insert("variants".to_string(), serde_json::Value::Array(variants_vec));

                serde_json::Value::Object(root_map)
            }
        }
    };

    trace!(
        "Completed generate_to_template_with_justification_for_enum for '{}'",
        type_name_str
    );
    expanded
}
