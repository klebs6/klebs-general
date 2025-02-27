crate::ix!();

pub fn generate_active_output_port_impl(
    io_enum_ident: &Ident,
    generics:      &syn::Generics,
    operator_spec: &OperatorSpec,
) -> proc_macro2::TokenStream 
{
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    // Build match arms for each declared output index
    let mut arms = Vec::new();
    for i in 0..operator_spec.outputs().len() {
        let variant_ident = syn::Ident::new(&format!("Output{}", i), proc_macro2::Span::call_site());
        let i_lit = syn::LitInt::new(&i.to_string(), proc_macro2::Span::call_site());
        arms.push(quote::quote! {
            Self::#variant_ident(_) => Some(#i_lit)
        });
    }

    if arms.is_empty() {
        // No output variants => always None
        quote::quote! {
            impl #impl_generics #io_enum_ident #type_generics #where_clause {
                pub fn active_output_port(&self) -> Option<usize> {
                    match self {
                        _ => None
                    }
                }
            }
        }
    } else {
        // We have 1+ arms => separate them with commas, then `_ => None`
        quote::quote! {
            impl #impl_generics #io_enum_ident #type_generics #where_clause {
                pub fn active_output_port(&self) -> Option<usize> {
                    match self {
                        #( #arms ),*,
                        _ => None
                    }
                }
            }
        }
    }
}
