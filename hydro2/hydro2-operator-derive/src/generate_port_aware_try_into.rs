crate::ix!();

pub fn generate_port_aware_try_into_impls(
    io_enum_ident: &Ident,
    generics:      &syn::Generics,
    operator_spec: &OperatorSpec,
) -> proc_macro2::TokenStream {

    let port_try_into0 
        = generate_port_aware_try_into0(
            io_enum_ident,
            generics,
            operator_spec
        );

    let port_try_into1 
        = generate_port_aware_try_into1(
            io_enum_ident,
            generics,
            operator_spec
        );

    let port_try_into2 
        = generate_port_aware_try_into2(
            io_enum_ident,
            generics,
            operator_spec
        );

    let port_try_into3 
        = generate_port_aware_try_into3(
            io_enum_ident,
            generics,
            operator_spec
        );

    let port_try_into0_any 
        = generate_port_aware_try_into0_any(
            io_enum_ident,
            generics,
            operator_spec
        );

    let port_try_into1_any 
        = generate_port_aware_try_into1_any(
            io_enum_ident,
            generics,
            operator_spec
        );

    let port_try_into2_any 
        = generate_port_aware_try_into2_any(
            io_enum_ident,
            generics,
            operator_spec
        );

    let port_try_into3_any 
        = generate_port_aware_try_into3_any(
            io_enum_ident,
            generics,
            operator_spec
        );


    quote::quote! { 
        #port_try_into0 
        #port_try_into1 
        #port_try_into2 
        #port_try_into3 
        #port_try_into0_any 
        #port_try_into1_any 
        #port_try_into2_any 
        #port_try_into3_any 
    }
}
