// ---------------- [ File: src/generate_port_aware_try_from.rs ]
crate::ix!();

pub fn generate_port_aware_try_from_impls(
    io_enum_ident: &Ident,
    generics:      &syn::Generics,
    operator_spec: &OperatorSpec,
) -> proc_macro2::TokenStream {

    let port_try_from0 
        = generate_port_aware_try_from0(
            io_enum_ident,
            generics,
            operator_spec
        );

    let port_try_from1 
        = generate_port_aware_try_from1(
            io_enum_ident,
            generics,
            operator_spec
        );

    let port_try_from2 
        = generate_port_aware_try_from2(
            io_enum_ident,
            generics,
            operator_spec
        );

    let port_try_from3 
        = generate_port_aware_try_from3(
            io_enum_ident,
            generics,
            operator_spec
        );

    quote::quote! { 
        #port_try_from0 
        #port_try_from1 
        #port_try_from2 
        #port_try_from3 
    }
}
