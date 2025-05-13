// ---------------- [ File: ai-json-template-derive/src/flatten_named_variant_fields.rs ]
crate::ix!();

pub fn flatten_named_variant_fields(
    named_fields:             &syn::FieldsNamed,
    skip_field_self_just_fn: impl Fn(&syn::Field) -> bool,
    is_leaf_type_fn:         impl Fn(&syn::Type) -> bool,
    skip_child_just:         bool,
    flatten_named_field_fn:  impl Fn(&syn::Ident, &syn::Type, bool, bool)
        -> (Vec<proc_macro2::TokenStream>, proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream)
) -> FlattenedFieldResult
{
    trace!(
        "flatten_named_variant_fields: {} field(s) to process.",
        named_fields.named.len()
    );

    let mut field_decls_for_fields  = Vec::new();
    let mut pattern_vars_for_fields = Vec::new();
    let mut item_inits              = Vec::new();
    let mut just_inits_for_fields   = Vec::new();
    let mut conf_inits_for_fields   = Vec::new();

    for field in &named_fields.named {

        let f_ident = match &field.ident {
            Some(id) => id,
            None => {
                warn!("Unnamed field in 'named' variant? Skipping.");
                continue;
            }
        };

        let skip_f_self = skip_field_self_just_fn(field);
        let child_skip  = skip_f_self || skip_child_just || is_leaf_type_fn(&field.ty);

        let (decls, i_init, j_init, c_init) =
            flatten_named_field_fn(f_ident, &field.ty, skip_f_self, child_skip);

        // Insert them (with commas after the first).
        for (i, decl) in decls.into_iter().enumerate() {
            if i == 0 {
                field_decls_for_fields.push(decl);
            } else {
                let with_comma = quote::quote! { #decl, };
                field_decls_for_fields.push(with_comma);
            }
        }

        pattern_vars_for_fields.push(quote::quote! { #f_ident });

        if !i_init.is_empty() {
            item_inits.push(i_init);
        }
        if !j_init.is_empty() {
            just_inits_for_fields.push(j_init);
        }
        if !c_init.is_empty() {
            conf_inits_for_fields.push(c_init);
        }
    }

    FlattenedFieldResultBuilder::default()
        .field_decls_for_fields(field_decls_for_fields)
        .pattern_vars_for_fields(pattern_vars_for_fields)
        .item_inits(item_inits)
        .just_inits_for_fields(just_inits_for_fields)
        .conf_inits_for_fields(conf_inits_for_fields)
        .build()
        .unwrap()
}

#[cfg(test)]
mod tests_flatten_named_variant_fields {
    use super::*;

    #[traced_test]
    fn test_zero_fields() {
        trace!("Starting test_zero_fields...");

        // We'll construct a struct that has no fields
        // Then extract its FieldsNamed, which should be empty.
        let ast: syn::DeriveInput = parse_quote! {
            struct EmptyStruct {}
        };
        let named_fields = match ast.data {
            syn::Data::Struct(ds) => match ds.fields {
                Fields::Named(nf) => nf,
                _ => panic!("Expected named fields")
            },
            _ => panic!("Expected a struct")
        };

        let skip_field_self_just_fn = |_field: &Field| {
            trace!("skip_field_self_just_fn called => returning false");
            false
        };
        let is_leaf_type_fn = |_ty: &syn::Type| {
            trace!("is_leaf_type_fn called => returning false");
            false
        };
        let flatten_named_field_fn = |_ident: &syn::Ident,
                                      _ty: &syn::Type,
                                      _skip_self_just: bool,
                                      _skip_child_just: bool|
         -> (Vec<proc_macro2::TokenStream>,
             proc_macro2::TokenStream,
             proc_macro2::TokenStream,
             proc_macro2::TokenStream)
        {
            trace!("flatten_named_field_fn called => returning empty expansions");
            (vec![], quote::quote!(), quote::quote!(), quote::quote!())
        };

        let result = flatten_named_variant_fields(
            &named_fields,
            skip_field_self_just_fn,
            is_leaf_type_fn,
            /* skip_child_just = */ false,
            flatten_named_field_fn
        );
        debug!("Result: {:?}", result);

        // With zero fields, we expect all vectors to be empty
        assert!(result.field_decls_for_fields().is_empty());
        assert!(result.pattern_vars_for_fields().is_empty());
        assert!(result.item_inits().is_empty());
        assert!(result.just_inits_for_fields().is_empty());
        assert!(result.conf_inits_for_fields().is_empty());

        info!("test_zero_fields passed successfully.");
    }

    #[traced_test]
    fn test_single_field_no_skips() {
        use quote::format_ident; // <-- ADDED for creating new Ident dynamically
        trace!("Starting test_single_field_no_skips...");

        // We'll construct a struct with a single named field.
        let ast: syn::DeriveInput = parse_quote! {
            struct SingleField {
                alpha: String
            }
        };
        let named_fields = match ast.data {
            syn::Data::Struct(ds) => match ds.fields {
                Fields::Named(nf) => nf,
                _ => panic!("Expected named fields")
            },
            _ => panic!("Expected a struct")
        };

        let skip_field_self_just_fn = |_field: &Field| {
            trace!("skip_field_self_just_fn => returning false");
            false
        };
        let is_leaf_type_fn = |_ty: &syn::Type| {
            trace!("is_leaf_type_fn => returning false for demonstration");
            false
        };

        let flatten_named_field_fn = |ident: &syn::Ident,
                                      ty: &syn::Type,
                                      skip_self_just: bool,
                                      parent_skip_child: bool|
            -> (Vec<proc_macro2::TokenStream>,
                proc_macro2::TokenStream,
                proc_macro2::TokenStream,
                proc_macro2::TokenStream)
        {
            debug!("flatten_named_field_fn called for field '{}', skip_self_just={}, parent_skip_child={}",
                   ident, skip_self_just, parent_skip_child);

            let decl = quote::quote! { #ident: #ty };
            let item_init = quote::quote! { #ident };

            let jname = format_ident!("{}_justification", ident);
            let cinit = format_ident!("{}_confidence", ident);

            let just_init = if skip_self_just {
                quote::quote!()
            } else {
                quote::quote!(#jname : String::from("justified"))
            };
            let conf_init = if skip_self_just {
                quote::quote!()
            } else {
                quote::quote!(#cinit : 1.0_f32)
            };

            (vec![decl], item_init, just_init, conf_init)
        };

        let result = flatten_named_variant_fields(
            &named_fields,
            skip_field_self_just_fn,
            is_leaf_type_fn,
            /* skip_child_just = */ false,
            flatten_named_field_fn
        );
        debug!("Result: {:?}", result);

        // We should have exactly one field in each vector, including justification/confidence expansions.
        assert_eq!(result.field_decls_for_fields().len(), 1);
        assert_eq!(result.pattern_vars_for_fields().len(), 1);
        assert_eq!(result.item_inits().len(), 1);
        assert_eq!(result.just_inits_for_fields().len(), 1);
        assert_eq!(result.conf_inits_for_fields().len(), 1);

        info!("test_single_field_no_skips passed successfully.");
    }

    #[traced_test]
    fn test_multiple_fields_with_mixed_skips() {
        use quote::format_ident; // <-- ADDED
        trace!("Starting test_multiple_fields_with_mixed_skips...");

        let ast: syn::DeriveInput = parse_quote! {
            struct MixedFields {
                alpha: i32,
                beta: Option<bool>,
                gamma: String
            }
        };
        let named_fields = match ast.data {
            syn::Data::Struct(ds) => match ds.fields {
                Fields::Named(nf) => nf,
                _ => panic!("Expected named fields")
            },
            _ => panic!("Expected a struct")
        };

        // We'll skip justification for 'beta' only
        let skip_field_self_just_fn = |field: &Field| {
            let name = field.ident.as_ref().unwrap().to_string();
            if name == "beta" {
                trace!("skip_field_self_just_fn => skipping justification for '{}'", name);
                true
            } else {
                false
            }
        };

        // We'll treat all fields as non-leaf for demonstration, so skip_child_just never triggers
        let is_leaf_type_fn = |_ty: &syn::Type| {
            false
        };

        let flatten_named_field_fn = |ident: &syn::Ident,
                                      _ty: &syn::Type,
                                      skip_self_just: bool,
                                      _skip_child_just: bool|
            -> (Vec<proc_macro2::TokenStream>,
                proc_macro2::TokenStream,
                proc_macro2::TokenStream,
                proc_macro2::TokenStream)
        {
            debug!("flatten_named_field_fn => field '{}', skip_self_just={}",
                   ident, skip_self_just);

            let decl = vec![quote::quote!( #ident: String )];
            let item_init = quote::quote!( #ident );

            let jname = format_ident!("{}_justification", ident);
            let cinit = format_ident!("{}_confidence", ident);

            let just_init = if skip_self_just {
                quote::quote!()
            } else {
                quote::quote!( #jname : String::from("ok") )
            };
            let conf_init = if skip_self_just {
                quote::quote!()
            } else {
                quote::quote!( #cinit : 0.99_f32 )
            };

            (decl, item_init, just_init, conf_init)
        };

        let result = flatten_named_variant_fields(
            &named_fields,
            skip_field_self_just_fn,
            is_leaf_type_fn,
            /* skip_child_just = */ false,
            flatten_named_field_fn
        );
        debug!("Result: {:?}", result);

        // We have 3 fields => alpha, beta, gamma
        // skip_self_just is false for alpha/gamma, true for beta
        // So alpha/gamma => we have just/conf expansions. Beta => none.
        assert_eq!(result.field_decls_for_fields().len(), 3);
        assert_eq!(result.pattern_vars_for_fields().len(), 3);
        assert_eq!(result.item_inits().len(), 3);

        // just_inits_for_fields should have expansions for alpha/gamma but not for beta
        assert_eq!(result.just_inits_for_fields().len(), 2, "Only alpha/gamma should have justification expansions");
        assert_eq!(result.conf_inits_for_fields().len(), 2, "Only alpha/gamma should have confidence expansions");

        info!("test_multiple_fields_with_mixed_skips passed successfully.");
    }

    #[traced_test]
    fn test_skip_child_just_entirely() {
        use quote::format_ident; // <-- ADDED
        trace!("Starting test_skip_child_just_entirely...");

        let ast: syn::DeriveInput = parse_quote! {
            struct ChildJustTest {
                first: u32,
                second: String
            }
        };
        let named_fields = match ast.data {
            syn::Data::Struct(ds) => match ds.fields {
                Fields::Named(nf) => nf,
                _ => panic!("Expected named fields")
            },
            _ => panic!("Expected a struct")
        };

        // We won't skip self justification for any field
        let skip_field_self_just_fn = |_field: &Field| {
            false
        };

        // We'll treat everything as a "non-leaf" so normal logic is used. But we'll call skip_child_just=true below.
        let is_leaf_type_fn = |_ty: &syn::Type| {
            false
        };

        let flatten_named_field_fn = |ident: &syn::Ident,
                                      _ty: &syn::Type,
                                      skip_self_just: bool,
                                      skip_child_just: bool|
            -> (Vec<proc_macro2::TokenStream>,
                proc_macro2::TokenStream,
                proc_macro2::TokenStream,
                proc_macro2::TokenStream)
        {
            debug!("flatten_named_field_fn => field '{}', skip_self_just={}, skip_child_just={}",
                   ident, skip_self_just, skip_child_just);

            let decls = vec![quote::quote!( #ident: i64 )];
            let item_init = if skip_child_just {
                quote::quote!( #ident )
            } else {
                quote::quote!( ::core::convert::From::from(#ident) )
            };

            let jname = format_ident!("{}_justification", ident);
            let cinit = format_ident!("{}_confidence", ident);

            let just_init = if skip_self_just {
                quote::quote!()
            } else {
                quote::quote!(#jname : String::from("default justification"))
            };

            let conf_init = if skip_self_just {
                quote::quote!()
            } else {
                quote::quote!(#cinit : 0.5_f32)
            };

            (decls, item_init, just_init, conf_init)
        };

        let result = flatten_named_variant_fields(
            &named_fields,
            skip_field_self_just_fn,
            is_leaf_type_fn,
            /* skip_child_just = */ true,
            flatten_named_field_fn
        );
        debug!("Result: {:?}", result);

        // We have 2 fields => first, second
        // skip_self_just=false => we do have just/conf expansions for each field
        // skip_child_just=true => the item inits won't have the ::core::convert::From call
        assert_eq!(result.field_decls_for_fields().len(), 2, "We have 2 declared fields");
        assert_eq!(result.pattern_vars_for_fields().len(), 2, "We have 2 pattern vars");
        assert_eq!(result.item_inits().len(), 2, "We have 2 item init expansions");
        assert_eq!(result.just_inits_for_fields().len(), 2, "We have 2 justification expansions");
        assert_eq!(result.conf_inits_for_fields().len(), 2, "We have 2 confidence expansions");

        let item_inits_strs: Vec<String> = result.item_inits()
            .iter()
            .map(|ts| ts.to_string())
            .collect();

        // Ensure skip_child_just = true => we do not see "From::from"
        for s in &item_inits_strs {
            assert!(
                !s.contains("::core :: convert :: From :: from"),
                "Expected no usage of From::from in item_inits: got '{}'",
                s
            );
        }

        info!("test_skip_child_just_entirely passed successfully.");
    }
}
