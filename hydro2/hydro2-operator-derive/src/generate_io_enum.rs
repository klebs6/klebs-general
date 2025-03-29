// ---------------- [ File: src/generate_io_enum.rs ]
crate::ix!();

/*
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
*/

#[instrument]
pub fn generate_io_enum_with_phantom(
    struct_ident:  &syn::Ident,
    operator_spec: &OperatorSpec,
    uses_lifetime: bool,
    impl_generics: &proc_macro2::TokenStream,
    type_generics: &proc_macro2::TokenStream,
    where_clause:  &proc_macro2::TokenStream,
    full_generics: &syn::Generics,
) -> proc_macro2::TokenStream {
    trace!("Generating operator IO enum with a phantom-data variant to avoid unused-generic errors.");

    // The enum name is e.g. "MyOperatorIO"
    let enum_ident = syn::Ident::new(
        &format!("{}IO", struct_ident),
        proc_macro2::Span::call_site(),
    );

    // Collect the standard input/output variants
    let mut variants = Vec::new();

    // For inputN => `InputN(TY)`
    for (i, ty) in operator_spec.inputs().iter().enumerate() {
        let variant_name = syn::Ident::new(&format!("Input{}", i), proc_macro2::Span::call_site());
        variants.push(quote::quote! { #variant_name(#ty) });
    }
    // For outputN => `OutputN(TY)`
    for (i, ty) in operator_spec.outputs().iter().enumerate() {
        let variant_name = syn::Ident::new(&format!("Output{}", i), proc_macro2::Span::call_site());
        variants.push(quote::quote! { #variant_name(#ty) });
    }

    // Next, build a phantom type referencing *all* generics in a single tuple.
    // This ensures that if T (or other generics) do *not* appear in the IO variants,
    // we still "use" them in the enum, preventing E0392 "unused type parameter".
    let mut phantom_params = Vec::new();
    for param in &full_generics.params {
        match param {
            syn::GenericParam::Type(t) => {
                let ident = &t.ident;
                phantom_params.push(quote::quote!(#ident));
            }
            syn::GenericParam::Lifetime(lf) => {
                let lifetime = &lf.lifetime;
                phantom_params.push(quote::quote!(&#lifetime ()));
            }
            syn::GenericParam::Const(c) => {
                let ident = &c.ident;
                phantom_params.push(quote::quote!(#ident));
            }
        }
    }

    // If we have any generics at all, build a "_Phantom" variant referencing them.
    // If there are zero generics, we can skip this variant safely.
    if !phantom_params.is_empty() {
        let phantom_tuple = quote::quote! { (#(#phantom_params),*) };
        variants.push(
            quote::quote! { _Phantom(::core::marker::PhantomData<#phantom_tuple>) }
        );
    }

    // Our final enum definition
    let enum_def = quote::quote! {
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum #enum_ident #impl_generics #where_clause {
            None,
            #( #variants ),*
        }
    };

    // Add a Default impl that just returns `None`
    let default_impl = quote::quote! {
        impl #impl_generics ::core::default::Default for #enum_ident #type_generics #where_clause {
            fn default() -> Self {
                Self::None
            }
        }
    };

    // Combine them
    let output = quote::quote! {
        #enum_def

        #default_impl
    };

    trace!("Finished generating IO enum with phantom-data variant: {}", enum_ident);
    output
}

#[cfg(test)]
mod test_generate_io_enum {
    use super::*;

    // There's already a test with "uses_lifetime = true" in your snippet,
    // but we can add another to show multiple references, etc.

    /// Example test for an operator with references (`uses_lifetime=true`).
    #[test]
    fn test_generate_io_enum() {
        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(foo))
            .opcode_expr::<syn::Path>(parse_quote!(BasicOpCode::Bar))
            .inputs(vec![parse_quote! { &[u32] }])
            .outputs(vec![parse_quote! { Vec<String> }])
            .build()
            .unwrap();

        // Here, we create generics `<'a, T: Debug>` for demonstration.
        // If you want a different set of bounds, just parse or build accordingly.
        let generics: syn::Generics = parse_quote! {
            <'a, T: Debug>
        };

        let enum_ts = generate_io_enum_with_phantom(
            &Ident::new("MyOperator", proc_macro2::Span::call_site()),
            &spec,
            true, // uses_lifetime
            &quote! { <'a, T:Debug> },
            &quote! { <'a, T> },
            &quote! {},
            &generics,
        );

        let enum_str = normalize_whitespace(&enum_ts.to_string());
        assert!(enum_str.contains("enum MyOperatorIO < 'a , T : Debug >"));
        assert!(enum_str.contains("Input0 (& [u32])"));
        assert!(enum_str.contains("Output0 (Vec < String >)"));
    }

    /// Example test for zero I/O (no inputs, no outputs).
    #[test]
    fn test_generate_io_enum_zero_inputs_outputs() {
        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(some_execute_fn))
            .opcode_expr::<syn::Path>(parse_quote!(BasicOpCode::SomeOp))
            .inputs(vec![])
            .outputs(vec![])
            .build()
            .unwrap();

        // For zero I/O, we can just do empty generics or minimal `<T>`.
        let generics: syn::Generics = parse_quote! { <T:Debug> };

        let enum_ts = generate_io_enum_with_phantom(
            &Ident::new("EmptyOperator", proc_macro2::Span::call_site()),
            &spec,
            false, // no &-refs => not forcing a lifetime
            &quote! { <T:Debug> },
            &quote! { <T> },
            &quote! {},
            &generics,
        );

        let enum_str = normalize_whitespace(&enum_ts.to_string());
        // Validate the generated code...
        assert!(enum_str.contains("enum EmptyOperatorIO < T : Debug >"));
        assert!(enum_str.contains("None"));
    }

    /// Example test for multiple inputs/outputs (no references).
    #[test]
    fn test_generate_io_enum_multiple_inputs_outputs_no_lifetime() {
        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(foo))
            .opcode_expr::<syn::Path>(parse_quote!(BasicOpCode::Bar))
            .inputs(vec![parse_quote! { Foo }, parse_quote! { i32 }])
            .outputs(vec![parse_quote! { String }, parse_quote! { Vec<u8> }])
            .build()
            .unwrap();

        // No lifetime param, but let's assume we want `<T:Debug>`.
        let generics: syn::Generics = parse_quote! { <T: Debug> };

        let enum_ts = generate_io_enum_with_phantom(
            &Ident::new("MultiOperator", proc_macro2::Span::call_site()),
            &spec,
            false, // no & references
            &quote! { <T:Debug> },
            &quote! { <T> },
            &quote! {},
            &generics,
        );

        let enum_str = normalize_whitespace(&enum_ts.to_string());
        assert!(enum_str.contains("enum MultiOperatorIO < T : Debug >"));
        assert!(enum_str.contains("Input0 (Foo)"));
        assert!(enum_str.contains("Input1 (i32)"));
        assert!(enum_str.contains("Output0 (String)"));
        assert!(enum_str.contains("Output1 (Vec < u8 >)"));
    }
}
