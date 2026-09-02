// ---------------- [ File: random-constructible-derive/src/expand_rand_construct_for_named_struct.rs ]
crate::ix!();

pub fn expand_rand_construct_for_named_struct(
    name: &Ident,
    fields: &FieldsNamed,
) -> TokenStream2 {
    let ctx  = collect_named_field_context(fields);
    let base = generate_rand_impl_named(name, &ctx);
    let env  = generate_env_helpers_named(name, &ctx);

    quote! { #base #env }
}

#[cfg(test)]
mod expand_for_named_struct_tests {
    use super::*;
    use quote::{quote, ToTokens};
    use syn::{parse_quote, ItemStruct};

    // ─── helpers ────────────────────────────────────────────────────────
    fn parse_named_struct(ts: proc_macro2::TokenStream) -> FieldsNamed {
        let item: ItemStruct = syn::parse2(ts).unwrap();
        match item.fields {
            syn::Fields::Named(n) => n,
            _ => unreachable!(),
        }
    }

    // ─── named‑field context & impls ────────────────────────────────────
    #[traced_test]
    fn collect_named_context_happy_path() {
        let fields = parse_named_struct(quote! {
            struct S {
                a: u8,
                #[rand_construct(psome = 0.3)] b: Option<i16>,
            }
        });

        let ctx = collect_named_field_context(&fields);
        // two identifiers captured
        assert_eq!(ctx.member_idents().len(), 2);
        // provider_types should be [u8, i16]
        let providers = ctx.provider_types().iter().map(|t| t.to_token_stream().to_string()).collect::<Vec<_>>();
        assert_eq!(providers, vec!["u8", "i16"]);
        // env helpers must be *skipped* because u8 is primitive
        assert!(provider_types_contain_primitive(&ctx.provider_types()));
    }

    #[traced_test]
    fn generated_rand_impl_contains_ident() {
        let fields = parse_named_struct(quote! { struct S { a: i32 } });
        let ctx    = collect_named_field_context(&fields);
        let impl_ts= generate_rand_impl_named(&parse_quote!(S), &ctx);

        let code = impl_ts.to_string();
        assert!(code.contains("impl RandConstruct for S"));
        assert!(code.contains("a"));
    }
}
