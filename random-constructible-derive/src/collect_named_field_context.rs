crate::ix!();

pub struct NamedContext {
    member_idents:     Vec<Ident>,
    provider_types:    Vec<Type>,
    rand_bounds:       Vec<TokenStream2>,
    inits_random:      Vec<TokenStream2>,
    inits_uniform:     Vec<TokenStream2>,
    inits_random_env:  Vec<TokenStream2>,
    inits_uniform_env: Vec<TokenStream2>,
}

pub fn collect_named_field_context(fields: &FieldsNamed) -> NamedContext {
    let mut ctx = NamedContext {
        member_idents:     Vec::new(),
        provider_types:    Vec::new(),
        rand_bounds:       Vec::new(),
        inits_random:      Vec::new(),
        inits_uniform:     Vec::new(),
        inits_random_env:  Vec::new(),
        inits_uniform_env: Vec::new(),
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
        append_ts(&mut ctx.inits_random_env,  tokens.random_env());
        append_ts(&mut ctx.inits_uniform_env, tokens.uniform_env());

        ctx.provider_types.extend(tokens.provider_types().iter().cloned());
        ctx.rand_bounds.push(tokens.rand_bound().clone());
    }

    ctx
}
