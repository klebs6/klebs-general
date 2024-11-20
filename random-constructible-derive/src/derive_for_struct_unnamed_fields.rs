crate::ix!();

pub fn derive_for_unnamed_fields(name: &Ident, fields_unnamed: &FieldsUnnamed) -> TokenStream2 {
    let field_types = fields_unnamed
        .unnamed
        .iter()
        .map(|f| f.ty.clone())
        .collect::<Vec<_>>();

    let has_primitive_field_type: bool = contains_primitive_type(&field_types);

    assert!(
        field_types.len() == 1,
        "Unnamed fields structs are expected to have exactly one field"
    );

    let field_type = &field_types[0];

    let expanded_rand_construct_impl = quote! {
        impl RandConstruct for #name
        where #field_type : RandConstruct
        {
            fn random() -> Self {
                Self(RandConstruct::random())
            }

            fn random_with_rng<R: rand::Rng + ?Sized>(rng: &mut R) -> Self {
                Self(RandConstruct::random_with_rng(rng))
            }

            fn uniform() -> Self {
                Self(RandConstruct::uniform())
            }
        }
    };

    let expanded_rand_env_impl = quote! {
        impl #name {
            pub fn random_with_env<ENV>() -> Self
            where
                ENV: RandConstructProbabilityMapProvider< #field_type >
            {
                Self(#field_type::random_with_env::<ENV>())
            }

            pub fn random_uniform_with_env<ENV>() -> Self
            where
                ENV: RandConstructProbabilityMapProvider< #field_type >
            {
                Self(#field_type::random_uniform_with_env::<ENV>())
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
                let output = derive_for_unnamed_fields(&input.ident, fields_unnamed);
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
                let output = derive_for_unnamed_fields(&input.ident, fields_unnamed);
                let output_str = token_stream_to_string(&output);

                // Assert that the output contains both impl blocks
                assert!(output_str.contains("impl RandConstruct for TestStruct"));
                assert!(output_str.contains("random_with_env"));
            }
        }
    }

    #[test]
    #[should_panic(expected = "Unnamed fields structs are expected to have exactly one field")]
    fn test_derive_for_unnamed_fields_multiple_fields() {
        let input: DeriveInput = parse_quote! {
            struct TestStruct(u8, String);
        };

        if let syn::Data::Struct(ref data_struct) = input.data {
            if let Fields::Unnamed(ref fields_unnamed) = data_struct.fields {
                // This should panic due to assertion
                derive_for_unnamed_fields(&input.ident, fields_unnamed);
            }
        }
    }
}
