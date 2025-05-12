// ---------------- [ File: ai-json-template-derive/src/build_justified_struct_accessors.rs ]
crate::ix!();

pub fn build_justified_struct_accessors(
    justified_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    ty_ident: &syn::Ident,
    field_mappings: &[FieldJustConfMapping],
) -> proc_macro2::TokenStream {
    trace!(
        "Building accessor impl for the 'Justified' struct => '{}'",
        justified_ident
    );

    let (item_acc, just_acc, conf_acc) =
        gather_item_accessors(named_fields, ty_ident, field_mappings);

    let expanded = quote::quote! {
        impl #justified_ident {
            #(#item_acc)*
            #(#just_acc)*
            #(#conf_acc)*
        }
    };

    debug!(
        "Accessor impl for '{}' now has item/just/conf methods: total={}",
        justified_ident,
        item_acc.len() + just_acc.len() + conf_acc.len()
    );
    expanded
}

#[cfg(test)]
mod test_build_justified_struct_accessors_exhaustive {
    use super::*;
    
    #[traced_test]
    fn test_no_fields() {
        trace!("Starting test_no_fields");

        // Prepare an empty FieldsNamed:
        let fields_named = FieldsNamed {
            brace_token: Default::default(),
            named: syn::punctuated::Punctuated::new(),
        };

        // No field mappings
        let field_mappings: Vec<FieldJustConfMapping> = Vec::new();

        // Our function under test:
        let justified_ident: Ident = parse_quote! { JustifiedEmpty };
        let ty_ident: Ident        = parse_quote! { EmptyStruct };

        // Invoke build_justified_struct_accessors
        let output = build_justified_struct_accessors(
            &justified_ident,
            &fields_named,
            &ty_ident,
            &field_mappings
        );

        debug!("Generated TokenStream for no-fields scenario: {}", output);

        // Check that the impl block references "JustifiedEmpty" and is empty in content
        let out_string = output.to_string();
        assert!(out_string.contains("impl JustifiedEmpty {"));
        // With no fields, we expect no item/just/conf accessors
        assert!(!out_string.contains("pub fn"));
    }

    #[traced_test]
    fn test_single_field_with_mapping() {
        trace!("Starting test_single_field_with_mapping");

        // We'll define a struct with exactly one named field:
        // struct SingleField { alpha: i32 }
        let alpha_field: Field = parse_quote! {
            alpha: i32
        };
        let mut named = syn::punctuated::Punctuated::new();
        named.push(alpha_field);
        let fields_named = FieldsNamed {
            brace_token: Default::default(),
            named,
        };

        // Suppose our "FieldJustConfMapping" indicates that there's a justification/conf field
        // named alpha_justification and alpha_confidence
        let field_ident = Ident::new("alpha", proc_macro2::Span::call_site());
        let alpha_just  = Ident::new("alpha_justification", proc_macro2::Span::call_site());
        let alpha_conf  = Ident::new("alpha_confidence",     proc_macro2::Span::call_site());

        let alpha_mapping = FieldJustConfMappingBuilder::default()
            .field_ident(field_ident)
            .justification_field_ident(alpha_just)
            .confidence_field_ident(alpha_conf)
            .justification_field_type(quote! { String })
            .confidence_field_type(quote! { f32 })
            .build()
            .unwrap();

        let field_mappings = vec![alpha_mapping];

        // Our function under test:
        let justified_ident: Ident = parse_quote! { JustifiedSingleField };
        let ty_ident: Ident        = parse_quote! { SingleField };

        // Generate code
        let output = build_justified_struct_accessors(
            &justified_ident,
            &fields_named,
            &ty_ident,
            &field_mappings
        );

        debug!("Generated TokenStream for single-field scenario: {}", output);

        // Confirm the output has the correct function names and signatures
        let out_string = output.to_string();
        // Should have "pub fn alpha(&self) -> & i32"
        assert!(out_string.contains("pub fn alpha (& self) -> & i32"));
        // Should have "pub fn alpha_justification(&self) -> & String"
        assert!(out_string.contains("pub fn alpha_justification (& self) -> & String"));
        // Should have "pub fn alpha_confidence(&self) -> & f32"
        assert!(out_string.contains("pub fn alpha_confidence (& self) -> & f32"));
    }

    #[traced_test]
    fn test_multiple_fields_with_and_without_mapping() {
        trace!("Starting test_multiple_fields_with_and_without_mapping");

        // We'll define a struct with multiple named fields:
        // struct MultiField { alpha: i32, beta: String, gamma: bool }
        let alpha_field: Field = parse_quote! { alpha: i32 };
        let beta_field:  Field = parse_quote! { beta: String };
        let gamma_field: Field = parse_quote! { gamma: bool };

        let mut named = syn::punctuated::Punctuated::new();
        named.push(alpha_field);
        named.push(beta_field);
        named.push(gamma_field);

        let fields_named = FieldsNamed {
            brace_token: Default::default(),
            named,
        };

        // We'll provide a mapping for only two of them: alpha and gamma
        let alpha_mapping = FieldJustConfMappingBuilder::default()
            .field_ident(Ident::new("alpha", proc_macro2::Span::call_site()))
            .justification_field_ident(Ident::new("alpha_justification", proc_macro2::Span::call_site()))
            .confidence_field_ident(Ident::new("alpha_confidence", proc_macro2::Span::call_site()))
            .justification_field_type(quote! { String })
            .confidence_field_type(quote! { f64 })
            .build()
            .unwrap();

        // We'll skip providing a mapping for "beta" to simulate "no justification needed"
        let gamma_mapping = FieldJustConfMappingBuilder::default()
            .field_ident(Ident::new("gamma", proc_macro2::Span::call_site()))
            .justification_field_ident(Ident::new("gamma_justification", proc_macro2::Span::call_site()))
            .confidence_field_ident(Ident::new("gamma_confidence",     proc_macro2::Span::call_site()))
            .justification_field_type(quote! { String })
            .confidence_field_type(quote! { f32 })
            .build()
            .unwrap();

        let field_mappings = vec![alpha_mapping, gamma_mapping];

        let justified_ident: Ident = parse_quote! { JustifiedMultiField };
        let ty_ident: Ident        = parse_quote! { MultiField };

        // Generate code
        let output = build_justified_struct_accessors(
            &justified_ident,
            &fields_named,
            &ty_ident,
            &field_mappings
        );

        debug!("Generated TokenStream for multiple-fields scenario: {}", output);

        let out_string = output.to_string();

        // Should have an item accessor for alpha, beta, gamma
        assert!(out_string.contains("pub fn alpha"));
        assert!(out_string.contains("-> & i32"));
        assert!(out_string.contains("pub fn beta"));
        assert!(out_string.contains("-> & String"));
        assert!(out_string.contains("pub fn gamma"));
        assert!(out_string.contains("-> & bool"));

        // Should have a justification accessor only for alpha and gamma
        assert!(out_string.contains("pub fn alpha_justification"));
        assert!(out_string.contains("-> & String"));
        assert!(out_string.contains("pub fn gamma_justification"));
        assert!(out_string.contains("-> & String"));

        // Beta justification should not exist
        assert!(!out_string.contains("beta_justification"));

        // Should have a confidence accessor only for alpha and gamma
        assert!(out_string.contains("pub fn alpha_confidence"));
        // alpha was declared as f64 in the field mapping
        assert!(out_string.contains("-> & f64"));
        assert!(out_string.contains("pub fn gamma_confidence"));
        assert!(out_string.contains("-> & f32"));

        // Beta confidence should not exist
        assert!(!out_string.contains("beta_confidence"));
    }

    #[traced_test]
    fn test_repeated_field_mappings_ignores_duplicates() {
        trace!("Starting test_repeated_field_mappings_ignores_duplicates");

        // We'll define a struct with one field:
        let single_field: Field = parse_quote! { value: i32 };
        let mut named = syn::punctuated::Punctuated::new();
        named.push(single_field);

        let fields_named = FieldsNamed {
            brace_token: Default::default(),
            named,
        };

        // We'll provide multiple mappings for the same field "value" to see if it gracefully picks the first match
        let field_mapping_1 = FieldJustConfMappingBuilder::default()
            .field_ident(Ident::new("value", proc_macro2::Span::call_site()))
            .justification_field_ident(Ident::new("value_justification", proc_macro2::Span::call_site()))
            .confidence_field_ident(Ident::new("value_confidence", proc_macro2::Span::call_site()))
            .justification_field_type(quote! { String })
            .confidence_field_type(quote! { f32 })
            .build()
            .unwrap();

        let field_mapping_2 = FieldJustConfMappingBuilder::default()
            .field_ident(Ident::new("value", proc_macro2::Span::call_site()))
            .justification_field_ident(Ident::new("value_justification2", proc_macro2::Span::call_site()))
            .confidence_field_ident(Ident::new("value_confidence2", proc_macro2::Span::call_site()))
            .justification_field_type(quote! { String })
            .confidence_field_type(quote! { f32 })
            .build()
            .unwrap();

        let field_mappings = vec![field_mapping_1, field_mapping_2];

        let justified_ident: Ident = parse_quote! { JustifiedOneField };
        let ty_ident: Ident        = parse_quote! { OneField };

        let output = build_justified_struct_accessors(
            &justified_ident,
            &fields_named,
            &ty_ident,
            &field_mappings
        );

        debug!("Generated TokenStream for repeated-mappings scenario: {}", output);

        let out_string = output.to_string();
        // We do expect a single set of item/just/conf methods for "value"
        assert!(out_string.contains("pub fn value"));
        assert!(out_string.contains("-> & i32"));
        assert!(out_string.contains("pub fn value_justification"));
        assert!(out_string.contains("-> & String"));
        assert!(out_string.contains("pub fn value_confidence"));
        assert!(out_string.contains("-> & f32"));
        // We do NOT expect "value_justification2" or "value_confidence2"
        assert!(!out_string.contains("value_justification2"));
        assert!(!out_string.contains("value_confidence2"));
    }

    #[traced_test]
    fn test_field_is_missing_in_struct() {
        trace!("Starting test_field_is_missing_in_struct");

        // We'll define a struct with one named field "foo"
        let foo_field: Field = parse_quote! { foo: i32 };
        let mut named = syn::punctuated::Punctuated::new();
        named.push(foo_field);

        let fields_named = FieldsNamed {
            brace_token: Default::default(),
            named,
        };

        // But our field mapping references "bar" which doesn't exist in that struct
        let field_mapping_bar = FieldJustConfMappingBuilder::default()
            .field_ident(Ident::new("bar", proc_macro2::Span::call_site()))
            .justification_field_ident(Ident::new("bar_justification", proc_macro2::Span::call_site()))
            .confidence_field_ident(Ident::new("bar_confidence", proc_macro2::Span::call_site()))
            .justification_field_type(quote! { String })
            .confidence_field_type(quote! { f32 })
            .build()
            .unwrap();

        let field_mappings = vec![field_mapping_bar];

        let justified_ident: Ident = parse_quote! { JustifiedMismatched };
        let ty_ident: Ident        = parse_quote! { MismatchedStruct };

        let output = build_justified_struct_accessors(
            &justified_ident,
            &fields_named,
            &ty_ident,
            &field_mappings
        );

        debug!("Generated TokenStream for missing-field scenario: {}", output);

        let out_string = output.to_string();
        // The item accessor for "foo"
        assert!(out_string.contains("pub fn foo (& self) -> & i32"));
        // There's no "bar" field in the struct, so no bar() accessor
        assert!(!out_string.contains("pub fn bar ("));
        // There's no justification/conf for "foo" in the mapping
        assert!(!out_string.contains("foo_justification"));
        // There's no matching "bar" field, so "bar_justification" won't appear
        assert!(!out_string.contains("bar_justification"));
    }

    #[traced_test]
    fn test_multiple_fields_none_have_mappings() {
        trace!("Starting test_multiple_fields_none_have_mappings");

        // We'll define a struct with multiple named fields but zero mappings
        let alpha_field: Field = parse_quote! { alpha: i8 };
        let beta_field:  Field = parse_quote! { beta: i16 };
        let gamma_field: Field = parse_quote! { gamma: i32 };

        let mut named = syn::punctuated::Punctuated::new();
        named.push(alpha_field);
        named.push(beta_field);
        named.push(gamma_field);

        let fields_named = FieldsNamed {
            brace_token: Default::default(),
            named,
        };

        // Provide no field mappings
        let field_mappings = vec![];

        let justified_ident: Ident = parse_quote! { JustifiedNoMappings };
        let ty_ident: Ident        = parse_quote! { NoMappingsStruct };

        let output = build_justified_struct_accessors(
            &justified_ident,
            &fields_named,
            &ty_ident,
            &field_mappings
        );

        debug!("Generated TokenStream for none-have-mappings scenario: {}", output);

        let out_string = output.to_string();

        // We expect item accessors for alpha, beta, gamma
        assert!(out_string.contains("pub fn alpha (& self) -> & i8"));
        assert!(out_string.contains("pub fn beta (& self) -> & i16"));
        assert!(out_string.contains("pub fn gamma (& self) -> & i32"));

        // We do NOT expect any justification or confidence accessors
        assert!(!out_string.contains("alpha_justification"));
        assert!(!out_string.contains("beta_justification"));
        assert!(!out_string.contains("gamma_justification"));
        assert!(!out_string.contains("alpha_confidence"));
        assert!(!out_string.contains("beta_confidence"));
        assert!(!out_string.contains("gamma_confidence"));
    }
}
