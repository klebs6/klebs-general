// ---------------- [ File: ai-json-template-derive/src/gather_field_injections.rs ]
crate::ix!();

/// Creates code expansions that add "xxx_justification" and "xxx_confidence"
/// placeholders for each field at runtime in `to_template_with_justification`.
/// If a field is marked `#[justify = false]`, we skip injecting placeholders for it.
pub fn gather_field_injections(
    named_fields: &syn::FieldsNamed
) -> Vec<proc_macro2::TokenStream> {
    let mut expansions = Vec::new();
    for field in &named_fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => continue,
        };

        if !is_justification_enabled(field) {
            tracing::trace!("Field '{}' has #[justify=false], skipping injection", field_ident);
            continue;
        }

        let fname_str = field_ident.to_string();
        expansions.push(quote! {
            {
                let field_str = #fname_str;
                let justify_key = format!("{}_justification", field_str);
                let conf_key    = format!("{}_confidence", field_str);

                // Minimal approach => always "string" for justification, "number" for confidence.
                let mut just_obj = serde_json::Map::new();
                just_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                just_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                just_obj.insert("generation_instructions".to_string(),
                    serde_json::Value::String(format!("Explain or justify your choice for the field '{}'", field_str)));
                fields_obj.insert(justify_key, serde_json::Value::Object(just_obj));

                let mut conf_obj = serde_json::Map::new();
                conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                conf_obj.insert("generation_instructions".to_string(),
                    serde_json::Value::String(format!("Confidence in '{}', in [0..1]", field_str)));
                fields_obj.insert(conf_key, serde_json::Value::Object(conf_obj));
            }
        });
    }
    expansions
}

#[cfg(test)]
mod test_subroutine_gather_field_injections {
    use super::*;

    #[traced_test]
    fn test_injections() {
        // We create two fields manually: alpha (bool) and beta (String).
        let f1 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("alpha", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { bool },
        };
        let f2 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("beta", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { String },
        };
        let named = FieldsNamed {
            brace_token: Default::default(),
            named: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(f1);
                p.push(f2);
                p
            },
        };

        let expansions = gather_field_injections(&named);
        assert_eq!(expansions.len(), 2, "We want one expansion per field");
    }

    #[traced_test]
    fn test_injections_empty() {
        // Zero fields => expansions should be empty
        let named = FieldsNamed {
            brace_token: Default::default(),
            named: syn::punctuated::Punctuated::new(),
        };

        let expansions = gather_field_injections(&named);
        assert_eq!(expansions.len(), 0, "No fields => no expansions");
    }

    #[traced_test]
    fn test_injections_with_justify_false() {
        // One field is normal, one has #[justify=false].
        let f1 = Field {
            attrs: vec![parse_quote! { #[justify = false] }],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("skip_me", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { u8 },
        };
        let f2 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("regular", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { bool },
        };
        let named = FieldsNamed {
            brace_token: Default::default(),
            named: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(f1);
                p.push(f2);
                p
            },
        };

        let expansions = gather_field_injections(&named);
        assert_eq!(
            expansions.len(),
            1,
            "Only 'regular' field should produce an injection"
        );
    }
}
