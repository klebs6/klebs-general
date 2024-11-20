crate::ix!();

pub fn derive_for_unit_struct(name: &Ident) -> TokenStream2 {
    let expanded = quote! {
        impl RandConstruct for #name {
            fn random() -> Self {
                Self
            }

            fn random_with_rng<R: rand::Rng + ?Sized>(_rng: &mut R) -> Self {
                Self
            }

            fn uniform() -> Self {
                Self
            }
        }

        impl #name {
            pub fn random_with_env<ENV>() -> Self {
                Self
            }

            pub fn random_uniform_with_env<ENV>() -> Self {
                Self
            }
        }
    };

    TokenStream2::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_for_unit_struct() {
        let input: DeriveInput = parse_quote! {
            struct TestStruct;
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Unit = data_struct.fields {
                let output = derive_for_unit_struct(&input.ident);
                let output_str = token_stream_to_string(&output);
                println!("output_str = {}", output_str);

                // Assert that the output contains the expected impl blocks
                assert!(output_str.contains("impl RandConstruct for TestStruct"));
                assert!(output_str.contains("fn random () -> Self { Self }"));
                assert!(output_str.contains("random_with_env"));
            }
        }
    }
}
