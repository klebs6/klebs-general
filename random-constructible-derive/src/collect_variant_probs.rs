// ---------------- [ File: src/collect_variant_probs.rs ]
crate::ix!();

/// Collects variant identifiers, probabilities, and fields from the enum variants.
///
/// # Arguments
///
/// * `variants` - A reference to `Punctuated<Variant, Comma>`.
///
/// # Returns
///
/// A tuple containing:
/// - A vector of variant identifiers.
/// - A vector of probabilities.
/// - A vector of variant fields.
pub fn collect_variant_probs(
    variants: &Punctuated<Variant, Comma>,
) -> (Vec<Ident>, Vec<f64>, Vec<Fields>) {
    let mut variant_idents = Vec::new();
    let mut probs = Vec::new();
    let mut variant_fields = Vec::new();

    for variant in variants {
        let variant_ident = variant.ident.clone();

        // Extract the probability from the attribute or default value
        let prob = extract_probability_from_attributes(&variant.attrs).unwrap_or(1.0);

        // Collect the fields of the variant for later use
        let fields = variant.fields.clone();

        variant_idents.push(variant_ident);
        probs.push(prob);
        variant_fields.push(fields);
    }

    (variant_idents, probs, variant_fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, punctuated::Punctuated, token::Comma, Variant};

    #[test]
    fn test_collect_variant_probs() {
        // Define sample variants and collect them into a Punctuated<Variant, Comma>
        let variants: Punctuated<Variant, Comma> = parse_quote! {
            #[rand_construct(p = 0.8)]
            UnitVariant,
            UnnamedVariant(i32, String),
            #[rand_construct(p = 0.3)]
            NamedVariant { x: f64, y: bool }
        };

        // Collect variant data
        let (variant_idents, probs, variant_fields) = collect_variant_probs(&variants);

        // Expected identifiers
        let expected_idents: Vec<String> = vec![
            "UnitVariant".to_string(),
            "UnnamedVariant".to_string(),
            "NamedVariant".to_string(),
        ];

        // Expected probabilities
        let expected_probs: Vec<f64> = vec![0.8, 1.0, 0.3];

        // Convert identifiers to strings for comparison
        let ident_strings: Vec<String> = variant_idents.iter().map(|id| id.to_string()).collect();

        assert_eq!(ident_strings, expected_idents);
        assert_eq!(probs, expected_probs);

        // Additional checks for variant fields can be added if necessary
        assert!(matches!(variant_fields[0], Fields::Unit));
        assert!(matches!(variant_fields[1], Fields::Unnamed(_)));
        assert!(matches!(variant_fields[2], Fields::Named(_)));
    }
}
