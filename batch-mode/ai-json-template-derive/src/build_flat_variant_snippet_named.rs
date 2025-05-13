// ---------------- [ File: ai-json-template-derive/src/build_flat_variant_snippet_named.rs ]
crate::ix!();

// ---------------------------------------------------------------------------
//  Subroutine E: Build the final “flat variant” snippet for named variants
// ---------------------------------------------------------------------------
pub fn build_flat_variant_snippet_named(
    variant_ident:       &syn::Ident,
    field_decls_top:     &[proc_macro2::TokenStream],
    field_decls_fields:  &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream
{
    trace!(
        "build_flat_variant_snippet_named: variant='{}'",
        variant_ident
    );

    let mut all_fields = Vec::new();
    all_fields.extend_from_slice(field_decls_top);
    all_fields.extend_from_slice(field_decls_fields);

    if !all_fields.is_empty() {
        quote::quote! {
            #variant_ident {
                #(#all_fields),*
            },
        }
    } else {
        // no fields at all
        quote::quote! {
            #variant_ident {},
        }
    }
}

#[cfg(test)]
mod test_build_flat_variant_snippet_named {
    use super::*;

    #[traced_test]
    fn test_no_fields() {
        trace!("Starting test_no_fields");

        let variant_ident = syn::Ident::new("EmptyVariant", proc_macro2::Span::call_site());
        let top_fields: Vec<proc_macro2::TokenStream> = vec![];
        let normal_fields: Vec<proc_macro2::TokenStream> = vec![];

        let snippet = build_flat_variant_snippet_named(&variant_ident, &top_fields, &normal_fields);

        // Wrap the snippet in an enum so we can parse and inspect it
        let wrapped = quote! {
            enum Temp {
                #snippet
            }
        };

        debug!("Emitted tokens:\n{}", wrapped);

        let parsed: ItemEnum = parse2(wrapped).expect("Failed to parse the generated snippet as an enum");
        assert_eq!(parsed.variants.len(), 1, "Should have exactly 1 variant");

        let v = &parsed.variants[0];
        assert_eq!(v.ident, "EmptyVariant", "Variant name mismatch");
        assert!(v.fields.is_empty(), "Expected no fields in this variant");
    }

    #[traced_test]
    fn test_only_top_fields() {
        trace!("Starting test_only_top_fields");

        let variant_ident = syn::Ident::new("TopVariant", proc_macro2::Span::call_site());
        let top_fields = vec![
            quote! { top_field_one: String },
            quote! { top_field_two: i32 },
        ];
        let normal_fields: Vec<proc_macro2::TokenStream> = vec![];

        let snippet = build_flat_variant_snippet_named(&variant_ident, &top_fields, &normal_fields);
        let wrapped = quote! {
            enum Temp {
                #snippet
            }
        };

        debug!("Emitted tokens:\n{}", wrapped);

        let parsed: ItemEnum = parse2(wrapped).expect("Failed to parse the generated snippet as an enum");
        assert_eq!(parsed.variants.len(), 1);

        let v = &parsed.variants[0];
        assert_eq!(v.ident, "TopVariant");
        match &v.fields {
            syn::Fields::Named(fields) => {
                let names: Vec<String> = fields.named.iter().map(|f| f.ident.as_ref().unwrap().to_string()).collect();
                assert_eq!(names.len(), 2);
                assert!(names.contains(&"top_field_one".to_string()));
                assert!(names.contains(&"top_field_two".to_string()));
            },
            _ => panic!("Expected named fields"),
        }
    }

    #[traced_test]
    fn test_only_normal_fields() {
        trace!("Starting test_only_normal_fields");

        let variant_ident = syn::Ident::new("NormalVariant", proc_macro2::Span::call_site());
        let top_fields: Vec<proc_macro2::TokenStream> = vec![];
        let normal_fields = vec![
            quote! { normal_field_a: bool },
            quote! { normal_field_b: u64 },
        ];

        let snippet = build_flat_variant_snippet_named(&variant_ident, &top_fields, &normal_fields);
        let wrapped = quote! {
            enum Temp {
                #snippet
            }
        };

        debug!("Emitted tokens:\n{}", wrapped);

        let parsed: ItemEnum = parse2(wrapped).expect("Failed to parse the generated snippet as an enum");
        assert_eq!(parsed.variants.len(), 1);

        let v = &parsed.variants[0];
        assert_eq!(v.ident, "NormalVariant");
        match &v.fields {
            syn::Fields::Named(fields) => {
                let names: Vec<String> = fields.named.iter().map(|f| f.ident.as_ref().unwrap().to_string()).collect();
                assert_eq!(names.len(), 2);
                assert!(names.contains(&"normal_field_a".to_string()));
                assert!(names.contains(&"normal_field_b".to_string()));
            },
            _ => panic!("Expected named fields"),
        }
    }

    #[traced_test]
    fn test_both_top_and_normal_fields() {
        trace!("Starting test_both_top_and_normal_fields");

        let variant_ident = syn::Ident::new("MixedVariant", proc_macro2::Span::call_site());
        let top_fields = vec![
            quote! { top_one: String },
            quote! { top_two: i32 },
        ];
        let normal_fields = vec![
            quote! { normal_a: bool },
            quote! { normal_b: f32 },
        ];

        let snippet = build_flat_variant_snippet_named(&variant_ident, &top_fields, &normal_fields);
        let wrapped = quote! {
            enum Temp {
                #snippet
            }
        };

        debug!("Emitted tokens:\n{}", wrapped);

        let parsed: ItemEnum = parse2(wrapped).expect("Failed to parse the generated snippet as an enum");
        assert_eq!(parsed.variants.len(), 1);

        let v = &parsed.variants[0];
        assert_eq!(v.ident, "MixedVariant");
        match &v.fields {
            syn::Fields::Named(fields) => {
                let names: Vec<String> = fields.named.iter().map(|f| f.ident.as_ref().unwrap().to_string()).collect();
                assert_eq!(names.len(), 4);
                assert!(names.contains(&"top_one".to_string()));
                assert!(names.contains(&"top_two".to_string()));
                assert!(names.contains(&"normal_a".to_string()));
                assert!(names.contains(&"normal_b".to_string()));
            },
            _ => panic!("Expected named fields"),
        }
    }

    #[traced_test]
    fn test_many_top_and_normal_fields() {
        trace!("Starting test_many_top_and_normal_fields");

        let variant_ident = syn::Ident::new("ManyFieldsVariant", proc_macro2::Span::call_site());
        let top_fields = vec![
            quote! { top_one: String },
            quote! { top_two: i32 },
            quote! { top_three: bool },
        ];
        let normal_fields = vec![
            quote! { normal_one: Option<u64> },
            quote! { normal_two: Vec<String> },
            quote! { normal_three: HashMap<String, bool> },
        ];

        let snippet = build_flat_variant_snippet_named(&variant_ident, &top_fields, &normal_fields);
        let wrapped = quote! {
            enum Temp {
                #snippet
            }
        };

        debug!("Emitted tokens:\n{}", wrapped);

        let parsed: ItemEnum = parse2(wrapped).expect("Failed to parse the generated snippet as an enum");
        assert_eq!(parsed.variants.len(), 1);

        let v = &parsed.variants[0];
        assert_eq!(v.ident, "ManyFieldsVariant");
        match &v.fields {
            syn::Fields::Named(fields) => {
                let names: Vec<String> = fields.named.iter().map(|f| f.ident.as_ref().unwrap().to_string()).collect();
                assert_eq!(names.len(), 6);
                assert!(names.contains(&"top_one".to_string()));
                assert!(names.contains(&"top_two".to_string()));
                assert!(names.contains(&"top_three".to_string()));
                assert!(names.contains(&"normal_one".to_string()));
                assert!(names.contains(&"normal_two".to_string()));
                assert!(names.contains(&"normal_three".to_string()));
            },
            _ => panic!("Expected named fields"),
        }
    }
}
