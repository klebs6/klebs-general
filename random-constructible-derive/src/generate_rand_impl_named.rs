crate::ix!();

pub fn generate_rand_impl_named(name: &Ident, c: &NamedContext) -> TokenStream2 {
    let NamedContext {
        member_idents,
        rand_bounds,
        inits_random,
        inits_uniform,
        ..
    } = c;

    quote! {
        impl RandConstruct for #name where #(#rand_bounds,)* {
            fn random()  -> Self { Self { #( #member_idents : #inits_random , )* } }
            fn uniform() -> Self { Self { #( #member_idents : #inits_uniform , )* } }
            fn random_with_rng<R: rand::Rng + ?Sized>(rng: &mut R) -> Self {
                Self { #( #member_idents : <_ as RandConstruct>::random_with_rng(rng), )* }
            }
        }
    }
}
