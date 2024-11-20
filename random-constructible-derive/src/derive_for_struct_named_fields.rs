crate::ix!();

pub fn derive_for_named_fields(name: &Ident, fields_named: &FieldsNamed) -> TokenStream2 {

    let field_names = fields_named
        .named
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();

    let field_types = fields_named
        .named
        .iter()
        .map(|f| f.ty.clone())
        .collect::<Vec<_>>();

    let has_primitive_field_type: bool = contains_primitive_type(&field_types);

    let expanded_rand_construct_impl = quote! {
        impl RandConstruct for #name
        where
            #(
                #field_types : RandConstruct,
            )*
        {
            fn random() -> Self {
                Self {
                    #(
                        #field_names: RandConstruct::random(),
                    )*
                }
            }

            fn random_with_rng<R: rand::Rng + ?Sized>(rng: &mut R) -> Self {
                Self {
                    #(
                        #field_names: RandConstruct::random_with_rng(rng),
                    )*
                }
            }

            fn uniform() -> Self {
                Self {
                    #(
                        #field_names: RandConstruct::uniform(),
                    )*
                }
            }
        }
    };

    let expanded_rand_env_impl = quote! {
        impl #name {
            pub fn random_with_env<ENV>() -> Self
            where
                #( ENV: RandConstructProbabilityMapProvider<#field_types>, )*
            {
                Self {
                    #(
                        #field_names: #field_types::random_with_env::<ENV>(),
                    )*
                }
            }

            pub fn random_uniform_with_env<ENV>() -> Self
            where
                #( ENV: RandConstructProbabilityMapProvider<#field_types>, )*
            {
                Self {
                    #(
                        #field_names: #field_types::random_uniform_with_env::<ENV>(),
                    )*
                }
            }
        }
    };

    match has_primitive_field_type {
        true => TokenStream2::from(expanded_rand_construct_impl),
        false => TokenStream2::from(quote! {
            #expanded_rand_construct_impl
            #expanded_rand_env_impl
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_for_named_fields_with_primitive_field() {
        let input: DeriveInput = parse_quote! {
            struct TestStruct {
                a: u8,
                b: String,
            }
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Named(ref fields_named) = data_struct.fields {
                let output = derive_for_named_fields(&input.ident, fields_named);
                let output_str = token_stream_to_string(&output);

                // Assert that the output contains the expected impl blocks
                assert!(output_str.contains("impl RandConstruct for TestStruct"));
                assert!(!output_str.contains("random_with_env"));
            }
        }
    }

    #[test]
    fn test_derive_for_named_fields_without_primitive_field() {
        let input: DeriveInput = parse_quote! {
            struct TestStruct {
                a: String,
                b: Vec<u8>,
            }
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Named(ref fields_named) = data_struct.fields {
                let output = derive_for_named_fields(&input.ident, fields_named);
                let output_str = token_stream_to_string(&output);

                // Assert that the output contains both impl blocks
                assert!(output_str.contains("impl RandConstruct for TestStruct"));
                assert!(output_str.contains("random_with_env"));
            }
        }
    }
}
