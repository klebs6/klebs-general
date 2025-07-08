// ---------------- [ File: random-constructible-derive/src/collect_named_field_context.rs ]
crate::ix!();

#[derive(Builder,Getters)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct NamedContext {
    member_idents:     Vec<Ident>,
    provider_types:    Vec<Type>,
    rand_bounds:       Vec<TokenStream2>,
    inits_random:      Vec<TokenStream2>,
    inits_uniform:     Vec<TokenStream2>,

    #[cfg(feature="env")]
    inits_random_env:  Vec<TokenStream2>,

    #[cfg(feature="env")]
    inits_uniform_env: Vec<TokenStream2>,

    field_types:       Vec<Type>,
}

pub fn collect_named_field_context(fields: &FieldsNamed) -> NamedContext {
    let mut ctx = NamedContext {
        member_idents:     Vec::new(),
        provider_types:    Vec::new(),
        rand_bounds:       Vec::new(),
        inits_random:      Vec::new(),
        inits_uniform:     Vec::new(),

        #[cfg(feature="env")]
        inits_random_env:  Vec::new(),

        #[cfg(feature="env")]
        inits_uniform_env: Vec::new(),

        field_types:       Vec::new(),
    };

    for field in &fields.named {
        let spec   = ParsedFieldSpec::from_syn_field(field);
        let tokens = spec.build_generation_tokens();

        ctx.member_idents.push(
            spec.ident()
                .clone()
                .expect("named struct field must have ident"),
        );

        append_ts(&mut ctx.inits_random,      tokens.random());
        append_ts(&mut ctx.inits_uniform,     tokens.uniform());

        #[cfg(feature="env")]
        append_ts(&mut ctx.inits_random_env,  tokens.random_env());

        #[cfg(feature="env")]
        append_ts(&mut ctx.inits_uniform_env, tokens.uniform_env());

        ctx.provider_types.extend(tokens.provider_types().iter().cloned());
        ctx.rand_bounds.push(tokens.rand_bound().clone());
        ctx.field_types.push(spec.ty().clone());
    }

    ctx
}

#[cfg(not(feature="env"))]
pub fn generate_env_helpers_named(name: &Ident, c: &NamedContext) -> TokenStream2 {
    quote! { }
}

#[cfg(feature="env")]
pub fn generate_env_helpers_named(name: &Ident, c: &NamedContext) -> TokenStream2 {
    if provider_types_contain_primitive(c.provider_types()) {
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
