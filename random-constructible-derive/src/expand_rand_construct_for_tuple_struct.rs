crate::ix!();

pub fn expand_rand_construct_for_tuple_struct(
    name: &Ident,
    fields: &FieldsUnnamed,
    generics: &Generics,
) -> TokenStream2 {
    let ctx  = collect_tuple_field_context(fields);
    let base = generate_rand_impl_tuple(name, generics, &ctx);
    let env  = generate_env_helpers_tuple(name, generics, &ctx);

    quote! { #base #env }
}

#[cfg(test)]
mod expand_for_tuple_struct_tests {
    use super::*;
    use quote::{quote, ToTokens};
    use syn::{parse_quote, ItemStruct};

    fn parse_tuple_struct(ts: proc_macro2::TokenStream) -> FieldsUnnamed {
        let item: ItemStruct = syn::parse2(ts).unwrap();
        match item.fields {
            syn::Fields::Unnamed(u) => u,
            _ => unreachable!(),
        }
    }

    #[traced_test]
    fn collect_tuple_context_happy_path() {
        let fields = parse_tuple_struct(quote! { struct T(u8, Option<u16>); });
        let ctx    = collect_tuple_field_context(&fields);

        // provider types are [u8, u16]
        let providers = ctx.provider_types.iter().map(|t| t.to_token_stream().to_string()).collect::<Vec<_>>();
        assert_eq!(providers, vec!["u8", "u16"]);
    }

    #[traced_test]
    fn generate_rand_impl_tuple_text() {
        let fields = parse_tuple_struct(quote! { struct T(i32); });
        let ctx    = collect_tuple_field_context(&fields);
        let impl_ts= generate_rand_impl_tuple(&parse_quote!(T), &Generics::default(), &ctx);

        assert!(impl_ts.to_string().contains("impl RandConstruct for T"));
    }
}
