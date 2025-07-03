crate::ix!();

pub fn generate_env_helpers_named(name: &Ident, c: &NamedContext) -> TokenStream2 {
    if provider_types_contain_primitive(&c.provider_types) {
        return quote! {};
    }

    let NamedContext {
        member_idents,
        provider_types,
        inits_random_env,
        inits_uniform_env,
        ..
    } = c;

    quote! {
        impl #name {
            pub fn random_with_env<ENV>() -> Self
            where #( ENV : RandConstructProbabilityMapProvider<#provider_types>, )* {
                Self { #( #member_idents : #inits_random_env , )* }
            }
            pub fn random_uniform_with_env<ENV>() -> Self
            where #( ENV : RandConstructProbabilityMapProvider<#provider_types>, )* {
                Self { #( #member_idents : #inits_uniform_env , )* }
            }
        }
    }
}
