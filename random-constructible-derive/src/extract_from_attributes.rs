// ---------------- [ File: src/extract_from_attributes.rs ]
crate::ix!();

pub fn extract_probability_from_attributes(attrs: &[syn::Attribute]) 
    -> Option<f64> 
{
    for attr in attrs {
        if let Some(probability) = extract_probability_from_attribute(attr) {
            return Some(probability);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Attribute, Path};

    #[test]
    fn test_extract_probability_from_attributes_with_valid_probability() {
        let attrs = vec![
            parse_quote!(#[other_construct(p = 0.5)]),
            parse_quote!(#[rand_construct(p = 0.85)]),
        ];

        let result = extract_probability_from_attributes(&attrs);
        assert_eq!(result, Some(0.85)); // Picks the first valid probability
    }

    #[test]
    fn test_extract_probability_from_attributes_with_no_valid_probability() {
        let attrs = vec![
            parse_quote!(#[other_construct(p = 0.5)]),
            parse_quote!(#[different_construct(q = 0.85)]),
            parse_quote!(#[rand_construct(q = 0.85)]),//wrong key q
        ];

        let result = extract_probability_from_attributes(&attrs);
        assert_eq!(result, None); // No "rand_construct"
    }

    #[test]
    fn test_extract_probability_from_attributes_with_empty_attrs() {
        let attrs: Vec<Attribute> = vec![]; // No attributes
        let result = extract_probability_from_attributes(&attrs);
        assert_eq!(result, None);
    }
}
