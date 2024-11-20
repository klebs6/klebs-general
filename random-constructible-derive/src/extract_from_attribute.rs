crate::ix!();

pub fn extract_probability_from_attribute(attr: &syn::Attribute) 
    -> Option<f64> 
{
    if attr.path.is_ident("rand_construct") {
        if let Ok(syn::Meta::List(meta_list)) = attr.parse_meta() {
            return extract_probability_from_meta_list(&meta_list);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Attribute, Meta, MetaList, NestedMeta, Path};

    #[test]
    fn test_extract_probability_with_valid_attribute() {

        let attr: Attribute = parse_quote!(#[rand_construct(p = 0.85)]);

        let result = extract_probability_from_attribute(&attr);

        assert_eq!(result, Some(0.85));
    }

    #[test]
    fn test_extract_probability_with_non_matching_attribute() {

        let attr: Attribute = parse_quote!(#[other_construct(p = 0.85)]);

        let result = extract_probability_from_attribute(&attr);

        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_probability_with_invalid_meta() {

        let attr: Attribute = parse_quote!(#[rand_construct(invalid_meta_format)]);

        let result = extract_probability_from_attribute(&attr);

        assert_eq!(result, None);
    }
}
