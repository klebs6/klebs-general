crate::ix!();

/// Generates match arms for the `default_weight` function of an enum.
///
/// # Arguments
///
/// * `variant_idents` - A slice of variant identifiers.
/// * `probs` - A slice of probabilities corresponding to each variant.
/// * `variant_fields` - A slice of fields corresponding to each variant.
///
/// # Returns
///
/// A vector of `TokenStream2` representing the match arms.
pub fn generate_match_arms(
    variant_idents: &[Ident],
    probs: &[f64],
    variant_fields: &[Fields],
) -> Vec<TokenStream2> {
    variant_idents
        .iter()
        .zip(probs.iter())
        .zip(variant_fields.iter())
        .map(|((ident, prob), fields)| {
            match fields {
                Fields::Named(_) => quote! {
                    Self::#ident { .. } => #prob,
                },
                Fields::Unnamed(_) => quote! {
                    Self::#ident(..) => #prob,
                },
                Fields::Unit => quote! {
                    Self::#ident => #prob,
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Fields, FieldsNamed, FieldsUnnamed};

    #[test]
    fn test_generate_match_arms() {
        // Define variant identifiers
        let variant_idents: Vec<Ident> = vec![
            parse_quote! { UnitVariant },
            parse_quote! { UnnamedVariant },
            parse_quote! { NamedVariant },
        ];

        // Define probabilities
        let probs: Vec<f64> = vec![1.0, 2.0, 3.0];

        // Define variant fields
        let variant_fields: Vec<Fields> = vec![
            // Unit variant
            Fields::Unit,
            // Unnamed variant with fields
            Fields::Unnamed(parse_quote! {
                (i32, String)
            }),
            // Named variant with fields
            Fields::Named(parse_quote! {
                {
                    x: f64,
                    y: bool
                }
            }),
        ];

        // Generate match arms
        let match_arms = generate_match_arms(&variant_idents, &probs, &variant_fields);

        // Convert TokenStreams to strings for assertion
        let match_arm_strings: Vec<String> = match_arms.iter().map(|ts| ts.to_string()).collect();

        // Expected match arms
        let expected_match_arms = vec![
            "Self :: UnitVariant => 1f64 ,",
            "Self :: UnnamedVariant (..) => 2f64 ,",
            "Self :: NamedVariant { .. } => 3f64 ,",
        ];

        // Assert that the generated match arms match the expected ones
        assert_eq!(match_arm_strings, expected_match_arms);
    }
}
