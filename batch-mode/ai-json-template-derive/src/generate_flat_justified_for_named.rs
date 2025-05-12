// ---------------- [ File: ai-json-template-derive/src/generate_flat_justified_for_named.rs ]
crate::ix!();

/// Replaces the original `generate_flat_justified_for_named` with a refactored version
/// that delegates to our single-purpose subroutines.
pub fn generate_flat_justified_for_named(
    ty_ident:     &syn::Ident,
    named_fields: &syn::FieldsNamed,
    span:         proc_macro2::Span
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream)
{
    trace!(
        "generate_flat_justified_for_named: starting for struct '{}'",
        ty_ident
    );

    let (flat_ident, justified_ident, justification_ident, confidence_ident) =
        create_flat_justified_idents_for_named(ty_ident, span);

    let mut flat_fields = Vec::new();
    let mut item_inits  = Vec::new();
    let mut just_inits  = Vec::new();
    let mut conf_inits  = Vec::new();

    gather_flat_fields_and_inits_for_named(
        ty_ident,
        named_fields,
        &mut flat_fields,
        &mut item_inits,
        &mut just_inits,
        &mut conf_inits,
    );

    let flat_ts = build_flattened_named_struct_for_named(&flat_ident, &flat_fields);
    let from_ts = build_from_impl_for_named(
        &flat_ident,
        &justified_ident,
        ty_ident,
        &justification_ident,
        &confidence_ident,
        &item_inits,
        &just_inits,
        &conf_inits,
    );

    debug!(
        "generate_flat_justified_for_named: finished building for struct '{}'",
        ty_ident
    );
    (flat_ts, from_ts)
}

#[cfg(test)]
mod test_refactored_generate_flat_justified_for_named {
    use super::*;
    use traced_test::traced_test;
    use syn::{parse_quote, Field, Visibility, FieldMutability};

    #[traced_test]
    fn test_refactored_generation_basic_named_struct() {
        // We'll build a struct with fields: name: String, count: u32
        let name_field = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("name", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { String },
        };
        let count_field = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("count", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { u32 },
        };

        let fields_named = syn::FieldsNamed {
            brace_token: Default::default(),
            named: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(name_field);
                p.push(count_field);
                p
            },
        };

        let struct_ident = syn::Ident::new("MyStruct", proc_macro2::Span::call_site());
        let (flat_ts, from_ts) = generate_flat_justified_for_named(
            &struct_ident,
            &fields_named,
            proc_macro2::Span::call_site()
        );

        let flat_str = flat_ts.to_string();
        let from_str = from_ts.to_string();

        assert!(
            flat_str.contains("pub struct FlatJustifiedMyStruct")
                && flat_str.contains("pub name : String")
                && flat_str.contains("pub count : u32"),
            "Expected flattened struct with fields name, count"
        );

        assert!(
            from_str.contains("impl From < FlatJustifiedMyStruct > for JustifiedMyStruct"),
            "Expected an impl From<FlatJustifiedMyStruct> for JustifiedMyStruct"
        );
        assert!(
            from_str.contains("name : flat . name")
                && from_str.contains("count : flat . count"),
            "Expected item init from flat.name, flat.count"
        );

        // Should see name_justification, count_justification in the flattened struct:
        assert!(
            flat_str.contains("pub name_justification : String")
                && flat_str.contains("pub count_justification : String"),
            "Expected top-level justification fields for each"
        );
        // And the From impl referencing them:
        assert!(
            from_str.contains("detail_justification : flat . name_justification")
                || from_str.contains("detail_justification: flat . count_justification"),
            "Expected usage of detail_justification for nested expansions"
        );
    }

    #[traced_test]
    fn test_refactored_generation_with_justify_false() {
        // We'll build a struct with fields:
        //   skip_it: String (with #[justify=false])
        //   keep_it: bool
        let skip_field = Field {
            attrs: vec![parse_quote! { #[justify = false] }],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("skip_it", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { String },
        };
        let keep_field = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(syn::Ident::new("keep_it", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: parse_quote! { bool },
        };
        let fields_named = syn::FieldsNamed {
            brace_token: Default::default(),
            named: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(skip_field);
                p.push(keep_field);
                p
            },
        };

        let si = syn::Ident::new("TestStruct", proc_macro2::Span::call_site());
        let (flat_ts, from_ts) = generate_flat_justified_for_named(
            &si,
            &fields_named,
            proc_macro2::Span::call_site()
        );

        let fs = flat_ts.to_string();
        let fr = from_ts.to_string();

        // skip_it => we do not see "skip_it_justification"
        assert!(fs.contains("skip_it : String"));
        assert!(!fs.contains("skip_it_justification"));

        // keep_it => has keep_it_justification
        assert!(fs.contains("keep_it : bool"));
        assert!(fs.contains("keep_it_justification"));
        assert!(fr.contains("detail_justification : flat . keep_it_justification"));
    }
}
