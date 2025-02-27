crate::ix!();

pub fn generate_port_aware_try_from0(
    io_enum_ident: &Ident,
    generics:      &syn::Generics,
    operator_spec: &OperatorSpec,
) -> proc_macro2::TokenStream {

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let input = operator_spec.get_input(0).expect("expected to be able to get the input");

    match input.is_unit_type() {
        true => quote!{
            impl #impl_generics PortTryFrom0<#input>
                for #io_enum_ident #type_generics
                    #where_clause
                {
                    type Error = NetworkError;

                    fn port_try_from0(src: #input) -> Result<Self, Self::Error> {
                        Err(NetworkError::InvalidPinAssignment)
                    }
                }

        },
        false => quote!{
            impl #impl_generics PortTryFrom0<#input>
                for #io_enum_ident #type_generics
                    #where_clause
                {
                    type Error = NetworkError;

                    fn port_try_from0(src: #input) -> Result<Self, Self::Error> {
                        Ok(Self::Input0(src))
                    }
                }
        }
    }
}

pub fn generate_port_aware_try_from1(
    io_enum_ident: &Ident,
    generics:      &syn::Generics,
    operator_spec: &OperatorSpec,
) -> proc_macro2::TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let input = operator_spec.get_input(1).expect("expected to be able to get the input");
    match input.is_unit_type() {
        true => quote!{
            impl #impl_generics PortTryFrom1<#input>
                for #io_enum_ident #type_generics
                    #where_clause
                {
                    type Error = NetworkError;

                    fn port_try_from1(src: #input) -> Result<Self, Self::Error> {
                        Err(NetworkError::InvalidPinAssignment)
                    }
                }

        },
        false => quote!{
            impl #impl_generics PortTryFrom1<#input>
                for #io_enum_ident #type_generics
                    #where_clause
                {
                    type Error = NetworkError;

                    fn port_try_from1(src: #input) -> Result<Self, Self::Error> {
                        Ok(Self::Input1(src))
                    }
                }
        }
    }
}

pub fn generate_port_aware_try_from2(
    io_enum_ident: &Ident,
    generics:      &syn::Generics,
    operator_spec: &OperatorSpec,
) -> proc_macro2::TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let input = operator_spec.get_input(2).expect("expected to be able to get the input");
    match input.is_unit_type() {
        true => quote!{
            impl #impl_generics PortTryFrom2<#input>
                for #io_enum_ident #type_generics
                    #where_clause
                {
                    type Error = NetworkError;

                    fn port_try_from2(src: #input) -> Result<Self, Self::Error> {
                        Err(NetworkError::InvalidPinAssignment)
                    }
                }

        },
        false => quote!{
            impl #impl_generics PortTryFrom2<#input>
                for #io_enum_ident #type_generics
                    #where_clause
                {
                    type Error = NetworkError;

                    fn port_try_from2(src: #input) -> Result<Self, Self::Error> {
                        Ok(Self::Input2(src))
                    }
                }
        }
    }
}

pub fn generate_port_aware_try_from3(
    io_enum_ident: &Ident,
    generics:      &syn::Generics,
    operator_spec: &OperatorSpec,
) -> proc_macro2::TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let input = operator_spec.get_input(3).expect("expected to be able to get the input");
    match input.is_unit_type() {
        true => quote!{
            impl #impl_generics PortTryFrom3<#input>
                for #io_enum_ident #type_generics
                    #where_clause
                {
                    type Error = NetworkError;

                    fn port_try_from3(src: #input) -> Result<Self, Self::Error> {
                        Err(NetworkError::InvalidPinAssignment)
                    }
                }

        },
        false => quote!{
            impl #impl_generics PortTryFrom3<#input>
                for #io_enum_ident #type_generics
                    #where_clause
                {
                    type Error = NetworkError;

                    fn port_try_from3(src: #input) -> Result<Self, Self::Error> {
                        Ok(Self::Input3(src))
                    }
                }
        }
    }
}
