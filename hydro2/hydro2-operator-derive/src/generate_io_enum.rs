// ---------------- [ File: hydro2-operator-derive/src/generate_io_enum.rs ]
crate::ix!();

pub fn generate_io_enum(
    struct_ident:  &Ident,
    operator_spec: &OperatorSpec,
    uses_lifetime: bool,
    impl_generics: &TokenStream,
    type_generics: &TokenStream,
    where_clause:  &TokenStream,

) -> TokenStream {

    let enum_ident = Ident::new(&format!("{}IO", struct_ident), Span::call_site());

    let mut variants = Vec::new();

    for (i, ty) in operator_spec.inputs().iter().enumerate() {
        let variant_name = Ident::new(&format!("Input{}", i), Span::call_site());
        variants.push(quote! { #variant_name(#ty) });
    }
    for (i, ty) in operator_spec.outputs().iter().enumerate() {
        let variant_name = Ident::new(&format!("Output{}", i), Span::call_site());
        variants.push(quote! { #variant_name(#ty) });
    }

    // If uses_lifetime == true, that means we've inserted <'a> into generics,
    // so the enum type signature might look like `enum FooIO<'a, T> { ... }`.
    // We'll simply emit the same generics for both cases.
    quote! {
        #[derive(Clone,PartialEq,Eq,Debug)]
        pub enum #enum_ident #impl_generics #where_clause {
            None,
            #( #variants ),*
        }

        impl #impl_generics Default for #enum_ident #type_generics #where_clause {
            fn default() -> Self {
                Self::None
            }
        }
    }
}

#[cfg(test)]
mod test_generate_io_enum {
    use super::*;

    // There's already a test with "uses_lifetime = true" in your snippet,
    // but we can add another to show multiple references, etc.

    #[test]
    fn test_generate_io_enum() {

        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(foo))
            .opcode_expr::<syn::Path>(parse_quote!(BasicOpCode::Bar))
            .inputs(vec![parse_quote! { &[u32] }])
            .outputs(vec![parse_quote! { Vec<String> }])
            .build()
            .unwrap();

        let enum_ts = generate_io_enum(
            &Ident::new("MyOperator", proc_macro2::Span::call_site()),
            &spec,
            true, // uses_lifetime
            &quote! { <'a, T:Debug> },
            &quote! { <'a, T> },
            &quote! {},
        );

        // Turn into a string for assertion:
        let enum_str = normalize_whitespace(&enum_ts.to_string());
        assert!(enum_str.contains("enum MyOperatorIO < 'a , T : Debug >"));
        assert!(enum_str.contains("Input0 (& [u32])"));
        assert!(enum_str.contains("Output0 (Vec < String >)"));
    }


    //------------------------------------------------------------------
    // 2) Tests for generate_io_enum
    //------------------------------------------------------------------
    #[test]
    fn test_generate_io_enum_zero_inputs_outputs() {
        // OperatorSpec with no inputs/outputs
        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(some_execute_fn))
            .opcode_expr::<syn::Path>(parse_quote!(BasicOpCode::SomeOp))
            .inputs(vec![])
            .outputs(vec![])
            .build()
            .unwrap();

        let enum_ts = generate_io_enum(
            &Ident::new("EmptyOperator", proc_macro2::Span::call_site()),
            &spec,
            /* uses_lifetime = */ false,
            &quote! { <T:Debug> },
            &quote! { <T> },
            &quote! {},
        );

        let enum_str = normalize_whitespace(&enum_ts.to_string());

        // Expect an enum with no variants
        // e.g. "pub enum EmptyOperatorIO < T >  { }"
        assert!(enum_str.contains("enum EmptyOperatorIO < T : Debug >"));
        // Should not contain "Input" or "Output"
        assert!(!enum_str.contains("Input0"));
        assert!(!enum_str.contains("Output0"));
    }

    #[test]
    fn test_generate_io_enum_multiple_inputs_outputs_no_lifetime() {
        // 2 inputs, 2 outputs, no references => uses_lifetime = false
        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(foo))
            .opcode_expr::<syn::Path>(parse_quote!(BasicOpCode::Bar))
            .inputs(vec![parse_quote! { Foo }, parse_quote! { i32 }])
            .outputs(vec![parse_quote! { String }, parse_quote! { Vec<u8> }])
            .build()
            .unwrap();

        let enum_ts = generate_io_enum(
            &Ident::new("MultiOperator", proc_macro2::Span::call_site()),
            &spec,
            false,
            &quote! { <T:Debug> },
            &quote! { <T> },
            &quote! {},
        );
        let enum_str = enum_ts.to_string();

        // We expect variants: Input0(Foo), Input1(i32), Output0(String), Output1(Vec<u8>)
        assert!(enum_str.contains("enum MultiOperatorIO < T : Debug >"));
        assert!(enum_str.contains("Input0 (Foo)"));
        assert!(enum_str.contains("Input1 (i32)"));
        assert!(enum_str.contains("Output0 (String)"));
        assert!(enum_str.contains("Output1 (Vec < u8 >)"));
    }
}
