crate::ix!();

/// Generates the implementation block for the `RandConstructEnum` trait.
///
/// # Arguments
///
/// * `name` - The identifier of the enum.
/// * `variant_constructors` - A vector of constructors for each variant.
/// * `match_arms` - A vector of match arms for the `default_weight` function.
/// * `probs` - A slice of probabilities corresponding to each variant.
///
/// # Returns
///
/// A `TokenStream2` representing the implementation block.
pub fn generate_random_constructible_enum_impl(
    with_env:             bool,
    name:                 &Ident,
    variant_constructors: &[TokenStream2],
    match_arms:           &[TokenStream2],
    probs:                &[f64],

) -> TokenStream2 {

    let rand_construct_enum_impl = quote! {
        impl RandConstructEnum for #name {
            fn all_variants() -> Vec<Self> {
                vec![
                    #(#variant_constructors),*
                ]
            }

            fn default_weight(&self) -> f64 {
                match self {
                    #(#match_arms)*
                }
            }

            fn create_default_probability_map() -> std::sync::Arc<std::collections::HashMap<#name, f64>> {
                use once_cell::sync::Lazy;
                use std::sync::Arc;
                use std::collections::HashMap;
                static PROBABILITY_MAP: Lazy<Arc<HashMap<#name, f64>>> = Lazy::new(|| {
                    let mut map = HashMap::new();
                    #(
                        map.insert(#variant_constructors, #probs);
                    )*
                    Arc::new(map)
                });

                Arc::clone(&PROBABILITY_MAP)
            }
        }
    };

    match with_env {
        true => quote!{
            impl RandConstructEnumWithEnv for #name {}
            #rand_construct_enum_impl
        },
        false => quote!{
            #rand_construct_enum_impl
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generate_impl() {
        let name: Ident = parse_quote! { MyEnum };

        let variant_constructors: Vec<TokenStream2> = vec![
            quote! { MyEnum::VariantA },
            quote! { MyEnum::VariantB( <i32 as RandConstruct>::random(), <String as RandConstruct>::random() ) },
        ];

        let match_arms: Vec<TokenStream2> = vec![
            quote! { Self::VariantA => 1.0, },
            quote! { Self::VariantB(..) => 2.0, },
        ];

        let probs: Vec<f64> = vec![1.0, 2.0];

        let impl_with_env = true;

        let impl_block 
            = generate_random_constructible_enum_impl(
                impl_with_env,
                &name, 
                &variant_constructors, 
                &match_arms, 
                &probs
            );

        // Convert TokenStream2 to string
        let impl_string = impl_block.to_string();

        println!("impl_string: {:#?}", impl_string);

        // Check if the impl_string contains expected patterns
        assert!(impl_string.contains("impl RandConstructEnum for MyEnum"));
        assert!(impl_string.contains("fn all_variants () -> Vec < Self >"));
        assert!(impl_string.contains("fn default_weight (& self) -> f64"));
        assert!(impl_string.contains("fn create_default_probability_map ()"));
        assert!(impl_string.contains("Self :: VariantA => 1.0 ,"));
        assert!(impl_string.contains("Self :: VariantB (..) => 2.0 ,"));
    }
}
