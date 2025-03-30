// ---------------- [ File: src/derive_for_struct_unnamed_fields.rs ]
crate::ix!();

pub fn derive_for_unnamed_fields(
    name:           &Ident, 
    fields_unnamed: &FieldsUnnamed, 
    generics:       &Generics

) -> TokenStream2 {

    let mut field_types = Vec::new();
    let mut field_random_initializers = Vec::new();
    let mut field_uniform_initializers = Vec::new();
    let mut rand_construct_bounds = Vec::new();

    for field in &fields_unnamed.unnamed {
        let field_type = &field.ty;

        // Check if field is Option<T>
        if let Some(inner_type) = is_option_type(field_type) {
            // Parse attributes to get the 'some' probability
            let some_prob = parse_some_probability(&field.attrs).unwrap_or(0.5);

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

            field_types.push(field_type.clone());
            field_random_initializers.push(random_initializer);
            field_uniform_initializers.push(uniform_initializer);
            rand_construct_bounds.push(quote! { #inner_type: RandConstruct });
        } else {
            // Default handling for other fields
            field_types.push(field_type.clone());
            field_random_initializers.push(quote! { <#field_type as RandConstruct>::random() });
            field_uniform_initializers.push(quote! { <#field_type as RandConstruct>::uniform() });
            rand_construct_bounds.push(quote! { #field_type: RandConstruct });
        }
    }

    let has_primitive_field_type: bool = contains_primitive_type(&field_types);

    let expanded_rand_construct_impl = quote! {
        impl #generics RandConstruct for #name #generics
        where
            #(#rand_construct_bounds,)*
        {
            fn random() -> Self {
                Self(#(#field_random_initializers),*)
            }

            fn random_with_rng<R: rand::Rng + ?Sized>(rng: &mut R) -> Self {
                Self(#(
                    <#field_types as RandConstruct>::random_with_rng(rng)
                ),*)
            }

            fn uniform() -> Self {
                Self(#(#field_uniform_initializers),*)
            }
        }
    };

    let expanded_rand_env_impl = quote! {
        impl #generics #name #generics {
            pub fn random_with_env<ENV>() -> Self
            where
                #( ENV: RandConstructProbabilityMapProvider<#field_types>, )*
            {
                Self(#(
                    #field_types::random_with_env::<ENV>()
                ),*)
            }

            pub fn random_uniform_with_env<ENV>() -> Self
            where
                #( ENV: RandConstructProbabilityMapProvider<#field_types>, )*
            {
                Self(#(
                    #field_types::random_uniform_with_env::<ENV>()
                ),*)
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
    fn test_derive_for_unnamed_fields_with_primitive_field() {
        let input: DeriveInput = parse_quote! {
            struct TestStruct(u8);
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Unnamed(ref fields_unnamed) = data_struct.fields {
                let output = derive_for_unnamed_fields(&input.ident, fields_unnamed, &input.generics);
                let output_str = token_stream_to_string(&output);

                // Assert that the output contains the expected impl blocks
                assert!(output_str.contains("impl RandConstruct for TestStruct"));
                assert!(!output_str.contains("random_with_env"));
            }
        }
    }

    #[test]
    fn test_derive_for_unnamed_fields_without_primitive_field() {
        let input: DeriveInput = parse_quote! {
            struct TestStruct(String);
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Unnamed(ref fields_unnamed) = data_struct.fields {
                let output = derive_for_unnamed_fields(&input.ident, fields_unnamed, &input.generics);
                let output_str = token_stream_to_string(&output);

                // Assert that the output contains both impl blocks
                assert!(output_str.contains("impl RandConstruct for TestStruct"));
                assert!(output_str.contains("random_with_env"));
            }
        }
    }

    #[test]
    fn test_derive_for_unnamed_fields_option_with_probability() {
        let input: DeriveInput = parse_quote! {
            struct MyStruct<T>(#[rand_construct(psome=0.8)] Option<T>);
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Unnamed(ref fields_unnamed) = data_struct.fields {
                let output = derive_for_unnamed_fields(&input.ident, fields_unnamed, &input.generics);
                let output_str = token_stream_to_string(&output);

                // Assert that the output contains the expected impl blocks
                assert!(output_str.contains("impl < T > RandConstruct for MyStruct < T >"));
                assert!(output_str.contains("where T : RandConstruct"));
                // Check that the probability is correctly used
                assert!(output_str.contains("if rand :: random :: < f64 > () < 0.8f64"));
            }
        }
    }

    #[test]
    fn test_derive_for_unnamed_fields_multiple_fields() {
        let input: DeriveInput = parse_quote! {
            struct MyStruct<T>(u8, #[rand_construct(psome=0.7)] Option<T>, String);
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Unnamed(ref fields_unnamed) = data_struct.fields {
                let output = derive_for_unnamed_fields(&input.ident, fields_unnamed, &input.generics);
                let output_str = token_stream_to_string(&output);

                // Check that the generated code includes all fields
                assert!(output_str.contains("impl < T > RandConstruct for MyStruct < T >"));
                assert!(output_str.contains("T : RandConstruct"));
                // Check that probabilities are correctly used
                assert!(output_str.contains("if rand :: random :: < f64 > () < 0.7f64"));
            }
        }
    }
}
