// ---------------- [ File: ai-json-template-derive/src/generate_to_template_with_justification_for_named.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn generate_to_template_with_justification_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    container_docs_str: &str
) -> proc_macro2::TokenStream {
    tracing::trace!(
        "Starting generate_to_template_with_justification_for_named for struct '{}'",
        ty_ident
    );

    let field_inits = gather_schemas_and_placeholders_for_named_fields(named_fields);

    let expanded = quote::quote! {
        impl AiJsonTemplateWithJustification for #ty_ident {
            fn to_template_with_justification() -> serde_json::Value {
                trace!(
                    "Generating to_template_with_justification() for struct '{}'",
                    stringify!(#ty_ident)
                );

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
    };

    tracing::trace!(
        "Completed generate_to_template_with_justification_for_named for '{}'",
        ty_ident
    );
    return expanded;
}
