// ---------------- [ File: src/phantom_data_for_generics.rs ]
crate::ix!();

pub fn phantom_data_for_generics(generics: &syn::Generics) -> TokenStream {
    let mut phantom_elems = Vec::new();
    for param in &generics.params {
        match param {
            syn::GenericParam::Type(t) => {
                let ident = &t.ident;
                phantom_elems.push(quote! { #ident });
            },
            syn::GenericParam::Lifetime(lt) => {
                let lifetime = &lt.lifetime;
                phantom_elems.push(quote! { &#lifetime () });
            },
            syn::GenericParam::Const(c) => {
                let ident = &c.ident;
                phantom_elems.push(quote! { #ident });
            }
        }
    }

    if phantom_elems.is_empty() {
        return quote!{};
    }

    quote! {
        _phantom: ::core::marker::PhantomData<(#(#phantom_elems),*)>
    }
}
