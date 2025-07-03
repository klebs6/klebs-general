crate::ix!();

/// Parsed semantic description of a single `syn::Field`.
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ParsedFieldSpec {
    ident:      Option<Ident>,
    ty:         Type,
    is_option:  bool,
    inner_ty:   Option<Type>,
    some_prob:  f64,
    min_max:    (Option<f64>, Option<f64>),
}

impl ParsedFieldSpec {
    /// Convert a raw `syn::Field` into a rich [`ParsedFieldSpec`].
    pub fn from_syn_field(field: &Field) -> Self {
        let ty = field.ty.clone();
        let (is_option, inner_ty) = match is_option_type(&ty) {
            Some(inner) => (true, Some(inner)),
            None        => (false, None),
        };

        ParsedFieldSpec {
            ident:    field.ident.clone(),
            ty,
            is_option,
            inner_ty,
            some_prob: parse_some_probability(&field.attrs).unwrap_or(0.5),
            min_max:  parse_min_max(&field.attrs),
        }
    }

    /// Build all [`FieldGenerationTokens`] needed for this field.
    pub fn build_generation_tokens(&self) -> FieldGenerationTokens {
        if self.is_option {
            self.make_tokens_for_option_field()
        } else {
            self.make_tokens_for_plain_field()
        }
    }

    /* ───── internal token builders ────────────────────────────── */

    fn make_tokens_for_plain_field(&self) -> FieldGenerationTokens {
        let ty     = &self.ty;
        let clamp  = clamp_code(ty, self.min_max.0, self.min_max.1);

        FieldGenerationTokens {
            random:      quote!({ let mut v = <#ty>::random();                 #clamp v }),
            uniform:     quote!({ let mut v = <#ty>::uniform();                #clamp v }),
            random_env:  quote!({ let mut v = <#ty>::random_with_env::<ENV>(); #clamp v }),
            uniform_env: quote!({ let mut v = <#ty>::random_uniform_with_env::<ENV>(); #clamp v }),
            provider_types: vec![ty.clone()],
            rand_bound: quote! { #ty : RandConstruct },
        }
    }

    fn make_tokens_for_option_field(&self) -> FieldGenerationTokens {
        let inner  = self.inner_ty.as_ref().expect("inner_ty exists");
        let clamp  = clamp_code(inner, self.min_max.0, self.min_max.1);

        let build_branch = |expr: TokenStream2, prob: TokenStream2| quote! {
            {
                if rand::random::<f64>() < #prob {
                    let mut v = #expr;
                    #clamp
                    Some(v)
                } else { None }
            }
        };

        FieldGenerationTokens {
            random:      build_branch(quote! { <#inner>::random() },                 quote! { #self.some_prob }),
            uniform:     build_branch(quote! { <#inner>::uniform() },                quote! { 0.5 }),
            random_env:  build_branch(quote! { <#inner>::random_with_env::<ENV>() }, quote! { #self.some_prob }),
            uniform_env: build_branch(quote! { <#inner>::random_uniform_with_env::<ENV>() }, quote! { 0.5 }),
            provider_types: vec![inner.clone()],
            rand_bound: quote! { #inner : RandConstruct },
        }
    }
}

/// Returns `true` if *any* provider‑type is a primitive; in that case the
/// env‑aware helpers must **not** be generated for the enclosing struct.
pub fn provider_types_contain_primitive(provider_types: &[Type]) -> bool {
    contains_primitive_type(provider_types)
}

#[cfg(test)]
mod parsed_field_spec_tests {
    use super::*;
    use quote::quote;
    use syn::{parse_quote, Field};

    /// Helper: parse a single field from a pseudo‑struct.
    fn parse_field(ts: proc_macro2::TokenStream) -> Field {
        let st: syn::ItemStruct = syn::parse2(quote! { struct _Tmp { #ts } }).unwrap();
        match st.fields {
            syn::Fields::Named(named) => named.named.first().unwrap().clone(),
            _ => unreachable!(),
        }
    }

    #[test]
    fn parsed_field_spec_plain() {
        let field = parse_field(parse_quote!( num: u16 ));
        let spec  = ParsedFieldSpec::from_syn_field(&field);

        assert_eq!(spec.ident().as_ref().unwrap(), "num");
        assert!(!spec.is_option());
        assert!(spec.inner_ty().is_none());
    }

    #[test]
    fn parsed_field_spec_option() {
        let field = parse_field(parse_quote!( #[rand_construct(psome = 0.8)] flag: Option<bool> ));
        let spec  = ParsedFieldSpec::from_syn_field(&field);

        assert!(spec.is_option());
        assert_eq!(spec.some_prob(), 0.8);
        assert_eq!(spec.inner_ty().as_ref().unwrap().to_token_stream().to_string(), "bool");
    }

    #[test]
    fn field_generation_tokens_plain() {
        let field  = parse_field(parse_quote!( value: i32 ));
        let spec   = ParsedFieldSpec::from_syn_field(&field);
        let tokens = spec.build_generation_tokens();

        // provider_types should contain *exactly* i32
        assert_eq!(tokens.provider_types().len(), 1);
        assert_eq!(tokens.provider_types()[0].to_token_stream().to_string(), "i32");

        // rand_bound should read "i32 : RandConstruct"
        assert_eq!(tokens.rand_bound().to_string(), "i32 : RandConstruct");
    }

    #[test]
    fn field_generation_tokens_option() {
        let field  = parse_field(parse_quote!( flag: Option<u8> ));
        let spec   = ParsedFieldSpec::from_syn_field(&field);
        let tokens = spec.build_generation_tokens();

        // provider_types contains only the *inner* type u8
        assert_eq!(tokens.provider_types().len(), 1);
        assert_eq!(tokens.provider_types()[0].to_token_stream().to_string(), "u8");

        // All four constructor snippets must mention "Some"
        let all = format!(
            "{}{}{}{}",
            tokens.random().to_string(),
            tokens.uniform().to_string(),
            tokens.random_env().to_string(),
            tokens.uniform_env().to_string()
        );
        assert!(all.contains("Some"));
    }

    #[test]
    fn provider_types_contain_primitive_predicate() {
        // Positive case: primitive present
        let vec1: Vec<Type> = vec![parse_quote!(u8), parse_quote!(MyType)];
        assert!(provider_types_contain_primitive(&vec1));

        // Negative case: only custom types
        let vec2: Vec<Type> = vec![parse_quote!(Foo), parse_quote!(Bar)];
        assert!(!provider_types_contain_primitive(&vec2));
    }
}
