// ---------------- [ File: ai-json-template-derive/src/gather_justification_and_confidence_fields.rs ]
crate::ix!();

pub fn gather_justification_and_confidence_fields(
    named_fields: &syn::FieldsNamed,
    out_justification_fields: &mut Vec<proc_macro2::TokenStream>,
    out_confidence_fields: &mut Vec<proc_macro2::TokenStream>,
    out_err: &mut proc_macro2::TokenStream,
    out_mappings: &mut Vec<FieldJustConfMapping>,
) {
    for field in &named_fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => {
                let e = syn::Error::new(field.span(), "Unnamed fields not supported.");
                let err_ts = e.to_compile_error();
                *out_err = quote::quote! { #out_err #err_ts };
                continue;
            }
        };

        if !is_justification_enabled(field) {
            // skip
            continue;
        }

        let just_ident = syn::Ident::new(
            &format!("{}_justification", field_ident),
            field_ident.span()
        );
        let conf_ident = syn::Ident::new(
            &format!("{}_confidence", field_ident),
            field_ident.span()
        );
        match classify_for_justification(&field.ty) {
            Ok(ClassifyResult::JustString) => {
                out_justification_fields.push(quote::quote! {
                    pub #just_ident: String,
                });
                out_confidence_fields.push(quote::quote! {
                    pub #conf_ident: f32,
                });
                out_mappings.push(FieldJustConfMappingBuilder::default()
                    .field_ident(field_ident.clone())
                    .justification_field_ident(just_ident)
                    .confidence_field_ident(conf_ident)
                    .justification_field_type(quote::quote!(String))
                    .confidence_field_type(quote::quote!(f32))
                    .build()
                    .unwrap()
                );
            }
            Ok(ClassifyResult::NestedJustification{ justification_type, confidence_type }) => {
                out_justification_fields.push(quote::quote! {
                    pub #just_ident: #justification_type,
                });
                out_confidence_fields.push(quote::quote! {
                    pub #conf_ident: #confidence_type,
                });
                out_mappings.push(FieldJustConfMappingBuilder::default()
                    .field_ident(field_ident.clone())
                    .justification_field_ident(just_ident)
                    .confidence_field_ident(conf_ident)
                    .justification_field_type(justification_type)
                    .confidence_field_type(confidence_type)
                    .build()
                    .unwrap()
                );
            }
            Err(e_ts) => {
                *out_err = quote::quote! { #out_err #e_ts };
            }
        }
    }
}

#[cfg(test)]
mod test_gather_just_conf_fields {
    use super::*;
    use traced_test::*;

    #[traced_test]
    fn test_fields() {
        // We'll build 2 fields manually: "count" (u8) and "label" (String).
        let f1 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("count", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { u8 },
        };
        let f2 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("label", proc_macro2::Span::call_site())),
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

        let mut jf = Vec::new();
        let mut cf = Vec::new();
        let mut err = quote!();
        let mut mappings = Vec::new();

        gather_justification_and_confidence_fields(&named, &mut jf, &mut cf, &mut err, &mut mappings);

        assert!(err.is_empty(), "No errors expected for these field types");
        assert_eq!(jf.len(), 2, "Should produce 2 justification fields");
        assert_eq!(cf.len(), 2, "Should produce 2 confidence fields");
        assert_eq!(mappings.len(), 2, "Should have 2 FieldJustConfMapping items");
    }

    #[traced_test]
    fn test_unnamed_should_error() {
        // This scenario has a field with no 'ident', simulating a tuple struct field => expect an error
        let unnamed = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: None, // no name => triggers error
            colon_token: None,
            ty: parse_quote! { bool },
        };

        let named = FieldsNamed {
            brace_token: Default::default(),
            named: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(unnamed);
                p
            },
        };

        let mut jf = Vec::new();
        let mut cf = Vec::new();
        let mut err = quote!();
        let mut mappings = Vec::new();

        gather_justification_and_confidence_fields(&named, &mut jf, &mut cf, &mut err, &mut mappings);

        // We expect the function to put an error message in 'err'
        assert!(!err.is_empty(), "Should have an error for unnamed field");
        assert_eq!(jf.len(), 0);
        assert_eq!(cf.len(), 0);
        assert_eq!(mappings.len(), 0);
    }

    #[traced_test]
    fn test_custom_type_error() {
        // If the classify_for_justification function rejects a type "BadType", we see that error here.
        let f1 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("bad_field", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { BadType },
        };

        let named = FieldsNamed {
            brace_token: Default::default(),
            named: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(f1);
                p
            },
        };

        let mut jf = Vec::new();
        let mut cf = Vec::new();
        let mut err = quote!();
        let mut mappings = Vec::new();

        gather_justification_and_confidence_fields(&named, &mut jf, &mut cf, &mut err, &mut mappings);

        // If classify_for_justification returned an Err => that error is appended to 'err'.
        // So let's confirm 'err' is not empty, and that we didn't produce fields for it.
        assert!(!err.is_empty(), "Expected an error for unknown or unsupported type 'BadType'");
        assert_eq!(jf.len(), 0, "Shouldn't produce justification fields for a failing type");
        assert_eq!(cf.len(), 0, "Shouldn't produce confidence fields for a failing type");
        assert_eq!(mappings.len(), 0, "No mapping should be produced for a failing type");
    }

    #[traced_test]
    fn test_field_with_justify_false_is_skipped() {
        // field 'skip_me' has #[justify = false]
        let f_skip = Field {
            attrs: vec![parse_quote! { #[justify = false] }],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("skip_me", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { String },
        };

        let f_normal = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("normal_field", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { bool },
        };

        let named = FieldsNamed {
            brace_token: Default::default(),
            named: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(f_skip);
                p.push(f_normal);
                p
            },
        };

        let mut jf = Vec::new();
        let mut cf = Vec::new();
        let mut err = quote!();
        let mut mappings = Vec::new();
        gather_justification_and_confidence_fields(&named, &mut jf, &mut cf, &mut err, &mut mappings);

        // skip_me => not present
        // normal_field => present
        assert!(err.is_empty(), "No error expected");
        assert_eq!(jf.len(), 1, "Only normal_field should produce justification field");
        assert_eq!(cf.len(), 1, "Only normal_field should produce confidence field");
        assert_eq!(mappings.len(), 1, "Only normal_field should appear in the mapping");
        let single_map = &mappings[0];
        assert_eq!(single_map.field_ident().to_string(), "normal_field");
    }
}
