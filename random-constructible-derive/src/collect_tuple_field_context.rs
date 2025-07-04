// ---------------- [ File: random-constructible-derive/src/collect_tuple_field_context.rs ]
crate::ix!();

#[derive(Builder,Getters)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct TupleContext {
    provider_types:    Vec<Type>,
    rand_bounds:       Vec<TokenStream2>,
    inits_random:      Vec<TokenStream2>,
    inits_uniform:     Vec<TokenStream2>,
    inits_random_env:  Vec<TokenStream2>,
    inits_uniform_env: Vec<TokenStream2>,
    field_types:       Vec<Type>,
}

pub fn collect_tuple_field_context(fields: &FieldsUnnamed) -> TupleContext {
    let mut ctx = TupleContext {
        provider_types:    Vec::new(),
        rand_bounds:       Vec::new(),
        inits_random:      Vec::new(),
        inits_uniform:     Vec::new(),
        inits_random_env:  Vec::new(),
        inits_uniform_env: Vec::new(),
        field_types:       Vec::new(),
    };

    for field in &fields.unnamed {
        let spec   = ParsedFieldSpec::from_syn_field(field);
        let tokens = spec.build_generation_tokens();

        append_ts(&mut ctx.inits_random,      tokens.random());
        append_ts(&mut ctx.inits_uniform,     tokens.uniform());
        append_ts(&mut ctx.inits_random_env,  tokens.random_env());
        append_ts(&mut ctx.inits_uniform_env, tokens.uniform_env());

        ctx.provider_types.extend(tokens.provider_types().iter().cloned());
        ctx.rand_bounds.push(tokens.rand_bound().clone());
        ctx.field_types.push(spec.ty().clone());
    }

    ctx
}

pub fn generate_env_helpers_tuple(
    name:     &Ident,
    generics: &Generics,
    c:        &TupleContext,

) -> TokenStream2 {

    if provider_types_contain_primitive(c.provider_types()) {
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

pub fn generate_rand_impl_tuple(
    name: &Ident,
    generics: &Generics,
    c: &TupleContext,
) -> TokenStream2 {
    let TupleContext {
        rand_bounds,
        inits_random,
        inits_uniform,
        field_types,
        ..
    } = c;

    quote! {
        impl #generics RandConstruct for #name #generics
        where #(#rand_bounds,)* {
            fn random()  -> Self { Self( #( #inits_random ),* ) }
            fn uniform() -> Self { Self( #( #inits_uniform ),* ) }
            fn random_with_rng<R: rand::Rng + ?Sized>(rng: &mut R) -> Self {
                Self( #( <#field_types as RandConstruct>::random_with_rng(rng) ),* )
            }
        }
    }
}
