crate::ix!();

pub fn derive_for_named_fields(name: &Ident, fields_named: &FieldsNamed) -> TokenStream2 {
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut inner_field_types = Vec::new();
    let mut field_random_initializers = Vec::new();
    let mut field_uniform_initializers = Vec::new();
    let mut field_random_with_env_initializers = Vec::new();
    let mut field_uniform_with_env_initializers = Vec::new();
    let mut rand_construct_bounds = Vec::new();

    for field in &fields_named.named {
        let field_name = field.ident.clone().unwrap();
        let field_type = field.ty.clone();

        if let Some(inner_type) = is_option_type(&field_type) {
            let some_prob = parse_some_probability(&field.attrs).unwrap_or(0.5);

            let random_initializer = quote! {
                if rand::random::<f64>() < #some_prob {
                    Some(<#inner_type>::random())
                } else {
                    None
                }
            };

            let uniform_initializer = quote! {
                if rand::random::<f64>() < 0.5 {
                    Some(<#inner_type>::uniform())
                } else {
                    None
                }
            };

            let random_with_env_initializer = quote! {
                if rand::random::<f64>() < #some_prob {
                    Some(<#inner_type>::random_with_env::<ENV>())
                } else {
                    None
                }
            };

            let uniform_with_env_initializer = quote! {
                if rand::random::<f64>() < 0.5 {
                    Some(<#inner_type>::random_uniform_with_env::<ENV>())
                } else {
                    None
                }
            };

            field_names.push(field_name);
            field_types.push(field_type.clone());
            inner_field_types.push(inner_type.clone());
            field_random_initializers.push(random_initializer);
            field_uniform_initializers.push(uniform_initializer);
            field_random_with_env_initializers.push(random_with_env_initializer);
            field_uniform_with_env_initializers.push(uniform_with_env_initializer);
            rand_construct_bounds.push(quote! { #inner_type: RandConstruct });
        } else {
            let random_initializer = quote! { <#field_type>::random() };
            let uniform_initializer = quote! { <#field_type>::uniform() };
            let random_with_env_initializer = quote! { <#field_type>::random_with_env::<ENV>() };
            let uniform_with_env_initializer = quote! { <#field_type>::random_uniform_with_env::<ENV>() };

            field_names.push(field_name.clone());
            field_types.push(field_type.clone());
            field_random_initializers.push(random_initializer);
            field_uniform_initializers.push(uniform_initializer);
            field_random_with_env_initializers.push(random_with_env_initializer);
            field_uniform_with_env_initializers.push(uniform_with_env_initializer);
            rand_construct_bounds.push(quote! { #field_type: RandConstruct });
        }
    }

    let expanded_rand_construct_impl = quote! {
        impl RandConstruct for #name
        where
            #(#rand_construct_bounds,)*
        {
            fn random() -> Self {
                Self {
                    #(
                        #field_names: #field_random_initializers,
                    )*
                }
            }

            fn random_with_rng<R: rand::Rng + ?Sized>(rng: &mut R) -> Self {
                Self {
                    #(
                        #field_names: <#field_types>::random_with_rng(rng),
                    )*
                }
            }

            fn uniform() -> Self {
                Self {
                    #(
                        #field_names: #field_uniform_initializers,
                    )*
                }
            }
        }
    };

    let expanded_rand_env_impl = quote! {
        impl #name {
            pub fn random_with_env<ENV>() -> Self
            where
                #(#rand_construct_bounds,)*
                #( ENV: RandConstructProbabilityMapProvider<#field_types>, )*
                #( ENV: RandConstructProbabilityMapProvider<#inner_field_types>, )*
            {
                Self {
                    #(
                        #field_names: #field_random_with_env_initializers,
                    )*
                }
            }

            pub fn random_uniform_with_env<ENV>() -> Self
            where
                #(#rand_construct_bounds,)*
                #( ENV: RandConstructProbabilityMapProvider<#field_types>, )*
                #( ENV: RandConstructProbabilityMapProvider<#inner_field_types>, )*
            {
                Self {
                    #(
                        #field_names: #field_uniform_with_env_initializers,
                    )*
                }
            }
        }
    };

    let has_primitive_field_type: bool = contains_primitive_type(&field_types);

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
    use syn::{parse_quote, DeriveInput};
    use quote::ToTokens;

    #[test]
    fn test_option_field_with_specified_probability() {
        let input: DeriveInput = parse_quote! {
            struct TestStruct {
                #[rand_construct(psome=0.8)]
                opt_field: Option<u8>,
            }
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Named(ref fields_named) = data_struct.fields {
                let output = derive_for_named_fields(&input.ident, fields_named);
                let output_str = output.to_string();

                println!("output_str =  {}", output_str);
                // Check that the generated code contains the correct probability
                assert!(output_str.contains("if rand :: random :: < f64 > () < 0.8f64"));
            }
        }
    }

    #[test]
    fn test_option_field_with_default_probability() {
        let input: DeriveInput = parse_quote! {
            struct TestStruct {
                opt_field: Option<u8>,
            }
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Named(ref fields_named) = data_struct.fields {
                let output = derive_for_named_fields(&input.ident, fields_named);
                let output_str = output.to_string();

                println!("output_str =  {}", output_str);
                // Check that the default probability is 0.5
                assert!(output_str.contains("if rand :: random :: < f64 > () < 0.5f64"));
            }
        }
    }

    #[test]
    fn test_option_field_uniform_distribution() {
        let input: DeriveInput = parse_quote! {
            struct TestStruct {
                #[rand_construct(psome=0.8)]
                opt_field: Option<u8>,
            }
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Named(ref fields_named) = data_struct.fields {
                let output = derive_for_named_fields(&input.ident, fields_named);
                let output_str = output.to_string();

                println!("output_str =  {}", output_str);
                // For uniform(), the probability should be 0.5
                assert!(output_str.contains("if rand :: random :: < f64 > () < 0.5"));
            }
        }
    }

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
