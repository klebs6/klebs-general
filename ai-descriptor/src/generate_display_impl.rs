crate::ix!();

/// Generate the `Display` implementation for a type if `derive_display` is true.
pub fn generate_display_impl(derive_display: bool, type_name: &Ident) 
    -> TokenStream2 
{
    if derive_display {
        quote! {
            impl std::fmt::Display for #type_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.ai())
                }
            }
        }
    } else {
        quote! {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::parse_quote;

    #[test]
    fn test_generate_display_impl_enabled() {
        let type_name: Ident = parse_quote!(MyEnum);
        let tokens = generate_display_impl(true, &type_name);

        let expected = quote! {
            impl std::fmt::Display for MyEnum {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.ai())
                }
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_display_impl_disabled() {
        let type_name: Ident = parse_quote!(MyEnum);
        let tokens = generate_display_impl(false, &type_name);

        let expected = quote! {}; // No implementation expected.

        assert_eq!(tokens.to_string(), expected.to_string());
    }
}
