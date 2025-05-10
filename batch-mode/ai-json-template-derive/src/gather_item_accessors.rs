// ---------------- [ File: ai-json-template-derive/src/gather_item_accessors.rs ]
crate::ix!();

/// Gathers three sets of “getter function expansions” for:
///  1) the original item’s fields,
///  2) each field’s justification,
///  3) each field’s confidence.
/// 
/// **Important**: we now accept `field_mappings` so we know each justification/confidence field’s type.
/// This prevents the ambiguous associated-type errors.
///
pub fn gather_item_accessors(
    named_fields:         &syn::FieldsNamed,
    _original_type_ident: &syn::Ident,
    field_mappings:       &[FieldJustConfMapping],
) -> (
    Vec<proc_macro2::TokenStream>, // item field accessors
    Vec<proc_macro2::TokenStream>, // justification field accessors
    Vec<proc_macro2::TokenStream>, // confidence field accessors
) {
    let mut item_accessors = Vec::new();
    let mut just_accessors = Vec::new();
    let mut conf_accessors = Vec::new();

    for field in &named_fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => continue,
        };
        let field_ty = &field.ty;
        let field_name_str = field_ident.to_string();

        // (1) Accessor for the *original* item field
        item_accessors.push(quote! {
            pub fn #field_ident(&self) -> &#field_ty {
                &self.item.#field_ident
            }
        });

        // Find the mapping that belongs to this field
        let mapping_opt = field_mappings.iter().find(|m| m.field_ident() == field_ident);
        if let Some(mapping) = mapping_opt {
            // Build the method name for justification
            let j_method_name = syn::Ident::new(
                &format!("{}_justification", field_name_str),
                field_ident.span()
            );
            // Build the method name for confidence
            let c_method_name = syn::Ident::new(
                &format!("{}_confidence", field_name_str),
                field_ident.span()
            );

            let j_field_ident = mapping.justification_field_ident();
            let c_field_ident = mapping.confidence_field_ident();
            let j_type = mapping.justification_field_type();
            let c_type = mapping.confidence_field_type();

            // (2) Justification accessor
            just_accessors.push(quote! {
                pub fn #j_method_name(&self) -> &#j_type {
                    &self.justification.#j_field_ident
                }
            });

            // (3) Confidence accessor
            conf_accessors.push(quote! {
                pub fn #c_method_name(&self) -> &#c_type {
                    &self.confidence.#c_field_ident
                }
            });
        } else {
            // If not found, we simply skip.
            // (This can happen if a field was omitted from the justification logic due to errors.)
            tracing::trace!("No justification/confidence mapping found for field '{}'", field_name_str);
        }
    }

    (item_accessors, just_accessors, conf_accessors)
}

#[cfg(test)]
mod test_subroutine_gather_item_accessors {
    use super::*;

    /// A helper that simulates generating FieldJustConfMapping for each named field.
    fn build_mappings_for_test(named: &FieldsNamed) -> Vec<FieldJustConfMapping> {
        let mut out = Vec::new();
        for fld in &named.named {
            let field_ident = fld.ident.clone().unwrap();
            let j_ident = syn::Ident::new(
                &format!("{}_justification", field_ident),
                field_ident.span(),
            );
            let c_ident = syn::Ident::new(
                &format!("{}_confidence", field_ident),
                field_ident.span(),
            );
            let cresult = classify_for_justification(&fld.ty).unwrap_or(ClassifyResult::JustString);

            match cresult {
                ClassifyResult::JustString => {
                    out.push(FieldJustConfMappingBuilder::default()
                        .field_ident(field_ident)
                        .justification_field_ident(j_ident)
                        .confidence_field_ident(c_ident)
                        .justification_field_type(quote!(String))
                        .confidence_field_type(quote!(f32))
                        .build()
                        .unwrap()
                    );
                }
                ClassifyResult::NestedJustification { justification_type, confidence_type } => {
                    out.push(FieldJustConfMappingBuilder::default()
                        .field_ident(field_ident)
                        .justification_field_ident(j_ident)
                        .confidence_field_ident(c_ident)
                        .justification_field_type(justification_type)
                        .confidence_field_type(confidence_type)
                        .build()
                        .unwrap()
                    );
                }
            }
        }
        out
    }

    #[traced_test]
    fn test_accessors() {
        // Two fields: "count" (u8) and "label" (String).
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

        let ti = syn::Ident::new("MyType", proc_macro2::Span::call_site());

        // Build mappings
        let mappings = build_mappings_for_test(&named);

        let (items, justs, confs) = gather_item_accessors(&named, &ti, &mappings);
        assert_eq!(items.len(), 2, "expected 2 item-accessor fns");
        assert_eq!(justs.len(), 2, "expected 2 justification-accessor fns");
        assert_eq!(confs.len(), 2, "expected 2 confidence-accessor fns");
    }

    #[traced_test]
    fn test_no_fields() {
        let named = FieldsNamed {
            brace_token: Default::default(),
            named: syn::punctuated::Punctuated::new(),
        };
        let ti = syn::Ident::new("NoneType", proc_macro2::Span::call_site());
        let mappings = vec![]; // no fields => no mappings

        let (items, justs, confs) = gather_item_accessors(&named, &ti, &mappings);
        assert_eq!(items.len(), 0);
        assert_eq!(justs.len(), 0);
        assert_eq!(confs.len(), 0);
    }

    #[traced_test]
    fn test_three_fields() {
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
        let f3 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("gamma", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { u32 },
        };

        let named = FieldsNamed {
            brace_token: Default::default(),
            named: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(f1);
                p.push(f2);
                p.push(f3);
                p
            },
        };

        let ti = syn::Ident::new("TriType", proc_macro2::Span::call_site());
        let mappings = build_mappings_for_test(&named);

        let (items, justs, confs) = gather_item_accessors(&named, &ti, &mappings);
        assert_eq!(items.len(), 3);
        assert_eq!(justs.len(), 3);
        assert_eq!(confs.len(), 3);
    }
}
