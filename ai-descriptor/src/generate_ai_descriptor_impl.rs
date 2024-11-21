crate::ix!();

/// Generate the implementation of the `AIDescriptor` trait for an enum.
pub fn generate_ai_descriptor_impl(enum_name: &Ident, variant_arms: &[TokenStream2]) 
    -> TokenStream2 
{
    quote! {
        impl AIDescriptor for #enum_name {
            fn ai(&self) -> std::borrow::Cow<'_, str> {
                match self {
                    #(#variant_arms),*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_generate_ai_descriptor_impl() {
        let enum_name: Ident = parse_quote!(MyEnum);

        let variant_arms = vec![
            quote! { MyEnum::Variant1 => std::borrow::Cow::Borrowed("Variant1") },
            quote! { MyEnum::Variant2 => std::borrow::Cow::Borrowed("Variant2") },
        ];

        let tokens = generate_ai_descriptor_impl(&enum_name, &variant_arms);

        let expected = quote! {
            impl AIDescriptor for MyEnum {
                fn ai(&self) -> std::borrow::Cow<'_, str> {
                    match self {
                        MyEnum::Variant1 => std::borrow::Cow::Borrowed("Variant1"),
                        MyEnum::Variant2 => std::borrow::Cow::Borrowed("Variant2")
                    }
                }
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }
}
