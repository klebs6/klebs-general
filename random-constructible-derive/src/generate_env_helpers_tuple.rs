crate::ix!();

pub fn generate_env_helpers_tuple(
    name: &Ident,
    generics: &Generics,
    c: &TupleContext,
) -> TokenStream2 {
    if provider_types_contain_primitive(&c.provider_types) {
        return quote!{};
    }

    let TupleContext {
        provider_types,
        inits_random_env,
        inits_uniform_env,
        ..
    } = c;

    quote! {
        impl #generics #name #generics {
            pub fn random_with_env<ENV>() -> Self
            where #( ENV : RandConstructProbabilityMapProvider<#provider_types>, )* {
                Self( #( #inits_random_env ),* )
            }
            pub fn random_uniform_with_env<ENV>() -> Self
            where #( ENV : RandConstructProbabilityMapProvider<#provider_types>, )* {
                Self( #( #inits_uniform_env ),* )
            }
        }
    }
}

pub fn append_ts(target: &mut Vec<TokenStream2>, ts: &TokenStream2) {
    target.push(ts.clone());
}
