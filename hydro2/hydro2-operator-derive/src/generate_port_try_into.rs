// ---------------- [ File: src/generate_port_try_into.rs ]
crate::ix!();

// A single macro that factors out all the repeating pieces:
macro_rules! generate_port_aware_try_into {
    ($fn_name:ident, $trait_fn_name:ident, $trait_name:ident, $variant_name:ident, $index:expr) => {
        pub fn $fn_name(
            io_enum_ident: &Ident,
            generics:      &syn::Generics,
            operator_spec: &OperatorSpec,
        ) -> proc_macro2::TokenStream {

            let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

            let output: Type = operator_spec.get_output($index).expect("expected to get the output");

            match output.is_unit_type() {
                true => quote! {
                    impl #impl_generics $trait_name<#output>
                        for #io_enum_ident #type_generics
                            #where_clause
                    {
                        type Error = NetworkError;

                        fn $trait_fn_name(self) -> Result<#output, Self::Error> {
                            match self {
                                _ => Err(NetworkError::InvalidPinAssignment),
                            }
                        }
                    }
                },
                false => quote! {
                    impl #impl_generics $trait_name<#output>
                        for #io_enum_ident #type_generics
                            #where_clause
                    {
                        type Error = NetworkError;

                        fn $trait_fn_name(self) -> Result<#output, Self::Error> {
                            match self {
                                #io_enum_ident::$variant_name(x) => Ok(x),
                                _ => Err(NetworkError::InvalidPinAssignment),
                            }
                        }
                    }
                },
            }
        }
    };
}

// Then just invoke the macro for each port index you need:
generate_port_aware_try_into!(generate_port_aware_try_into0, port_try_into0, PortTryInto0, Output0, 0);
generate_port_aware_try_into!(generate_port_aware_try_into1, port_try_into1, PortTryInto1, Output1, 1);
generate_port_aware_try_into!(generate_port_aware_try_into2, port_try_into2, PortTryInto2, Output2, 2);
generate_port_aware_try_into!(generate_port_aware_try_into3, port_try_into3, PortTryInto3, Output3, 3);

macro_rules! generate_port_aware_try_into_any {
    ($fn_name:ident, $trait_fn_name:ident, $trait_name:ident, $variant_name:ident, $index:expr) => {
        pub fn $fn_name(
            io_enum_ident: &syn::Ident,
            generics:      &syn::Generics,
            operator_spec: &OperatorSpec,
        ) -> proc_macro2::TokenStream {

            let (_, type_generics, _where_clause_for_type) = generics.split_for_impl();

            // 1) Clone and add `'a` to the generics
            let mut augmented = generics.clone();
            augmented.params.push(syn::parse_quote!('a));

            // 2) Force the enum type to outlive `'a`.
            augmented
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#io_enum_ident #type_generics: 'a));

            // 3) Also require each generic type param `T: 'a`.
            for type_param in generics.type_params() {
                let ty_ident = &type_param.ident;
                augmented
                    .make_where_clause()
                    .predicates
                    .push(syn::parse_quote!(#ty_ident: 'a));
            }

            let (impl_generics, _, where_clause_for_impl) = augmented.split_for_impl();

            let output: Type = operator_spec
                .get_output($index)
                .expect("expected output");

            match output.is_unit_type() {
                true => quote::quote! {
                    impl #impl_generics $trait_name
                        for #io_enum_ident #type_generics
                        #where_clause_for_impl
                    {
                        type Error = NetworkError;

                        fn $trait_fn_name(self) -> Result<unsafe_erased::Erased, Self::Error> {
                            Err(NetworkError::InvalidPinAssignment)
                        }
                    }
                },
                false => quote::quote! {
                    impl #impl_generics $trait_name
                        for #io_enum_ident #type_generics
                        #where_clause_for_impl
                    {
                        type Error = NetworkError;

                        fn $trait_fn_name(self) -> Result<unsafe_erased::Erased, Self::Error> {
                            match self {
                                #io_enum_ident::$variant_name(x) => Ok(unsafe_erased::Erased::new(x)),
                                _ => Err(NetworkError::InvalidPinAssignment),
                            }
                        }
                    }
                },
            }
        }
    };
}

// Then instantiate the macro for each port
generate_port_aware_try_into_any!(generate_port_aware_try_into0_any, port_try_into0_any, PortTryInto0Any, Output0, 0);
generate_port_aware_try_into_any!(generate_port_aware_try_into1_any, port_try_into1_any, PortTryInto1Any, Output1, 1);
generate_port_aware_try_into_any!(generate_port_aware_try_into2_any, port_try_into2_any, PortTryInto2Any, Output2, 2);
generate_port_aware_try_into_any!(generate_port_aware_try_into3_any, port_try_into3_any, PortTryInto3Any, Output3, 3);
