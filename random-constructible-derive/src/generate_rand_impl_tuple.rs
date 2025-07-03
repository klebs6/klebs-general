crate::ix!();

pub fn generate_rand_impl_tuple(
    name: &Ident,
    generics: &Generics,
    c: &TupleContext,
) -> TokenStream2 {
    let TupleContext {
        rand_bounds,
        inits_random,
        inits_uniform,
        ..
    } = c;

    quote! {
        impl #generics RandConstruct for #name #generics
        where #(#rand_bounds,)* {
            fn random()  -> Self { Self( #( #inits_random ),* ) }
            fn uniform() -> Self { Self( #( #inits_uniform ),* ) }
            fn random_with_rng<R: rand::Rng + ?Sized>(rng: &mut R) -> Self {
                Self( #( <_ as RandConstruct>::random_with_rng(rng) ),* )
            }
        }
    }
}
