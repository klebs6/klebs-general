//---------------------[file: hydro2-network-wire-drive/src/build_port_try_into_impl.rs]
crate::ix!();

/// Generates impl blocks that require `X: Clone` in their `where`-clause.
pub fn build_port_try_into_impl(
    enum_ident:     &syn::Ident,

    // The same generics we used on the enum
    base_generics:  &syn::Generics, 
    variant_idents: &Vec<syn::Ident>,
) -> proc_macro2::TokenStream {

    assert!(variant_idents.len() != 0);

    // 1) Clone the original generics
    let mut augmented = base_generics.clone();

    // 2) Append a fresh type parameter `X`.
    //    `syn::parse_quote!` is a handy way to parse tokens into a `GenericParam`.
    augmented.params.push(parse_quote!(X));

    // 3) Ensure we have a `where` clause, then append `X: Clone`.
    //    `make_where_clause` will create one if it doesn't exist already.
    augmented.make_where_clause().predicates.push(parse_quote!(X: Clone));

    // 4) Now split the *augmented* generics for our `impl`s.
    let (augmented_impl_generics, augmented_ty_generics, augmented_where_clause) = augmented.split_for_impl();

    // For the *enum type*, we still use the base generics "as-is".
    let (base_impl_generics, base_ty_generics, base_where_clause) = base_generics.split_for_impl();

    // We'll generate `match` arms for each port-based trait. We just map the
    // `variant_idents` into match arms for 0, 1, 2, 3.
    let mut match_arms0 = Vec::new();
    let mut match_arms1 = Vec::new();
    let mut match_arms2 = Vec::new();
    let mut match_arms3 = Vec::new();

    for variant_ident in variant_idents {
        match_arms0.push(quote::quote! { Self::#variant_ident(x) => x.port_try_into0_any() });
        match_arms1.push(quote::quote! { Self::#variant_ident(x) => x.port_try_into1_any() });
        match_arms2.push(quote::quote! { Self::#variant_ident(x) => x.port_try_into2_any() });
        match_arms3.push(quote::quote! { Self::#variant_ident(x) => x.port_try_into3_any() });
    }

    let mut active_port_match_arms = Vec::new();
    for variant_ident in variant_idents {
        active_port_match_arms.push(
            quote::quote! { Self::#variant_ident(x) => x.active_output_port() }
        );
    }

    // Generate the final code for the four impls. Each demands `X: Clone`
    // in order to safely call `as_x.clone()`.
    quote::quote! {

        impl #base_impl_generics #enum_ident #base_ty_generics #base_where_clause {

            pub fn active_output_port(&self) -> Option<usize> {
                match self {
                    #( #active_port_match_arms ),*,
                    _ => None
                }
            }

            /// Combines `active_output_port()` + the correct `port_try_intoN()` in one shot.
            pub fn port_try_into_dynamic<__T>(&self) -> Result<__T, NetworkError>
            where
                Self: PortTryInto0<__T, Error=NetworkError>
                + PortTryInto1<__T, Error=NetworkError>
                + PortTryInto2<__T, Error=NetworkError>
                + PortTryInto3<__T, Error=NetworkError>,
            {
                let idx = self.active_output_port()
                    .ok_or_else(|| NetworkError::InvalidPinAssignment)?; 

                match idx {
                    0 => self.clone().port_try_into0(),
                    1 => self.clone().port_try_into1(),
                    2 => self.clone().port_try_into2(),
                    3 => self.clone().port_try_into3(),
                    _ => unreachable!("port index {idx} out of range"),
                }
            }
        }

        impl #augmented_impl_generics hydro2_operator::PortTryInto0<X> for #enum_ident #base_ty_generics
            #augmented_where_clause
        {
            type Error = hydro2_operator::NetworkError;

            fn port_try_into0(self) -> ::core::result::Result<X, hydro2_operator::NetworkError> {
                use hydro2_operator::PortTryInto0Any;
                let any_val = match self {
                    #( #match_arms0 ),*,
                    _ => Err(hydro2_operator::NetworkError::InvalidPinAssignment),
                }?;
                Ok( unsafe { any_val.downcast::<X>()? })
            }
        }

        impl #augmented_impl_generics hydro2_operator::PortTryInto1<X> for #enum_ident #base_ty_generics
            #augmented_where_clause
        {
            type Error = hydro2_operator::NetworkError;

            fn port_try_into1(self) -> ::core::result::Result<X, hydro2_operator::NetworkError> {
                use hydro2_operator::PortTryInto1Any;
                let any_val = match self {
                    #( #match_arms1 ),*,
                    _ => Err(hydro2_operator::NetworkError::InvalidPinAssignment),
                }?;
                Ok( unsafe { any_val.downcast::<X>()? })
            }
        }

        impl #augmented_impl_generics hydro2_operator::PortTryInto2<X> for #enum_ident #base_ty_generics
            #augmented_where_clause
        {
            type Error = hydro2_operator::NetworkError;

            fn port_try_into2(self) -> ::core::result::Result<X, hydro2_operator::NetworkError> {
                use hydro2_operator::PortTryInto2Any;
                let any_val = match self {
                    #( #match_arms2 ),*,
                    _ => Err(hydro2_operator::NetworkError::InvalidPinAssignment),
                }?;
                Ok( unsafe { any_val.downcast::<X>()? })
            }
        }

        impl #augmented_impl_generics hydro2_operator::PortTryInto3<X> for #enum_ident #base_ty_generics
            #augmented_where_clause
        {
            type Error = hydro2_operator::NetworkError;

            fn port_try_into3(self) -> ::core::result::Result<X, hydro2_operator::NetworkError> {
                use hydro2_operator::PortTryInto3Any;
                let any_val = match self {
                    #( #match_arms3 ),*,
                    _ => Err(hydro2_operator::NetworkError::InvalidPinAssignment),
                }?;
                Ok( unsafe { any_val.downcast::<X>()? })
            }
        }
    }
}
