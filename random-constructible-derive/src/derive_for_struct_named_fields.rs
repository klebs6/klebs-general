crate::ix!();

pub fn derive_for_named_fields(name: &Ident, fields_named: &FieldsNamed) -> TokenStream2 {
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut field_random_initializers  = Vec::new();
    let mut field_uniform_initializers = Vec::new();
    let mut rand_construct_bounds = Vec::new();

    for field in &fields_named.named {
        let field_name = field.ident.clone().unwrap();
        let field_type = field.ty.clone();

        // Check if field is Option<T>
        if let Type::Path(TypePath { path, .. }) = &field_type {
            if path.segments.last().unwrap().ident == "Option" {
                // Extract T from Option<T>
                if let syn::PathArguments::AngleBracketed(ref args) = path.segments.last().unwrap().arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        // Parse attributes to get the 'some' probability
                        let mut some_prob = None;
                        for attr in &field.attrs {
                            if attr.path.is_ident("rand_construct") {
                                // Parse the attribute content
                                let meta = attr.parse_meta().unwrap();
                                if let syn::Meta::List(meta_list) = meta {
                                    for nested_meta in meta_list.nested {
                                        if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = nested_meta {
                                            if nv.path.is_ident("psome") {
                                                if let syn::Lit::Float(lit_float) = &nv.lit {
                                                    some_prob = Some(lit_float.base10_parse::<f64>().unwrap());
                                                } else if let syn::Lit::Int(lit_int) = &nv.lit {
                                                    some_prob = Some(lit_int.base10_parse::<f64>().unwrap());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        let some_prob = some_prob.unwrap_or(0.5);

                        // For random(), use the specified probability
                        let random_initializer = quote! {
                            if rand::random::<f64>() < #some_prob {
                                Some(<#inner_type as RandConstruct>::random())
                            } else {
                                None
                            }
                        };

                        // For uniform(), use probability 0.5
                        let uniform_initializer = quote! {
                            if rand::random::<f64>() < 0.5 {
                                Some(<#inner_type as RandConstruct>::uniform())
                            } else {
                                None
                            }
                        };

                        field_names.push(field_name);
                        field_types.push(field_type.clone());
                        field_random_initializers.push(random_initializer);
                        field_uniform_initializers.push(uniform_initializer);
                        rand_construct_bounds.push(quote! { #inner_type: RandConstruct });
                        continue;
                    }
                }
            }
        }

        // Default handling for other fields
        field_names.push(field_name.clone());
        field_types.push(field_type.clone());
        field_random_initializers.push(quote! { <#field_type as RandConstruct>::random() });
        field_uniform_initializers.push(quote! { <#field_type as RandConstruct>::uniform() });
        rand_construct_bounds.push(quote! { #field_type: RandConstruct });
    }

    let has_primitive_field_type: bool = contains_primitive_type(&field_types);

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
                        #field_names: <#field_types as RandConstruct>::random_with_rng(rng),
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
                #( ENV: RandConstructProbabilityMapProvider<#field_types>, )*
            {
                Self {
                    #(
                        #field_names: {
                            // Handle Option<T> fields
                            let value = {
                                #[allow(unused_imports)]
                                use rand::Rng;
                                if rand::thread_rng().gen::<f64>() < 0.5 {
                                    Some(#field_types::random_with_env::<ENV>())
                                } else {
                                    None
                                }
                            };
                            value
                        },
                    )*
                }
            }

            pub fn random_uniform_with_env<ENV>() -> Self
            where
                #( ENV: RandConstructProbabilityMapProvider<#field_types>, )*
            {
                Self {
                    #(
                        #field_names: {
                            // Similar handling as above
                            let value = {
                                #[allow(unused_imports)]
                                use rand::Rng;
                                if rand::thread_rng().gen::<f64>() < 0.5 {
                                    Some(field_types::random_uniform_with_env::<ENV>())
                                } else {
                                    None
                                }
                            };
                            value
                        },
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
