crate::ix!();

/// Generate a match arm for a unit variant in an enum.
/// Example: `MyEnum::VariantName => std::borrow::Cow::Borrowed("VariantName")`
pub fn generate_unit_variant_arm(enum_name: &Ident, variant_name: &Ident) 
    -> TokenStream2 
{
    quote! {
        #enum_name::#variant_name => std::borrow::Cow::Borrowed(stringify!(#variant_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generate_unit_variant_arm() {
        let enum_name: Ident = parse_quote!(MyEnum);
        let variant_name: Ident = parse_quote!(MyVariant);

        let tokens = generate_unit_variant_arm(&enum_name, &variant_name);

        let expected = quote! {
            MyEnum::MyVariant => std::borrow::Cow::Borrowed(stringify!(MyVariant))
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }
}
