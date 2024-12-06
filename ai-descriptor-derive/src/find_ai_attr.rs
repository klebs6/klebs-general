crate::ix!();

pub(crate) fn find_attr_with_tag(attrs: &[Attribute], tag: &str) -> Option<String> {
    for attr in attrs {
        if attr.path.is_ident(tag) {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                // Handle #[tag("value")]
                for nested in meta_list.nested.iter() {
                    if let NestedMeta::Lit(Lit::Str(lit_str)) = nested {
                        return Some(lit_str.value());
                    }
                }
            } else if let Ok(Meta::NameValue(MetaNameValue { lit: Lit::Str(lit_str), .. })) = attr.parse_meta() {
                // Handle #[tag = "value"]
                return Some(lit_str.value());
            }
        }
    }
    None

}

pub(crate) fn find_ai_attr(attrs: &[Attribute]) -> Option<String> {
    find_attr_with_tag(attrs,"ai")
}

pub(crate) fn find_open_attr(attrs: &[Attribute]) -> Option<String> {
    find_attr_with_tag(attrs,"open")
}


#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Attribute};

    #[test]
    fn test_find_ai_attr_with_valid_ai() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai("example text")])];
        assert_eq!(find_ai_attr(&attrs), Some("example text".to_string()));
    }

    #[test]
    fn test_find_ai_attr_with_non_ai() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[not_ai("example text")])];
        assert_eq!(find_ai_attr(&attrs), None);
    }

    #[test]
    fn test_find_ai_attr_with_malformed_ai() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai = "example text"])];
        assert_eq!(find_ai_attr(&attrs), Some("example text".to_string()));
    }

    #[test]
    fn test_find_ai_attr_with_invalid_literal() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai(123)])];
        assert_eq!(find_ai_attr(&attrs), None);
    }

    #[test]
    fn test_find_ai_attr_with_empty_attributes() {
        let attrs: Vec<Attribute> = vec![];
        assert_eq!(find_ai_attr(&attrs), None);
    }

    #[test]
    fn test_find_ai_attr_with_multiple_attributes() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[not_ai("other text")]),
            parse_quote!(#[ai("example text")]),
        ];
        assert_eq!(find_ai_attr(&attrs), Some("example text".to_string()));
    }

    #[test]
    fn test_find_ai_attr_with_multiple_ai_attributes() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[ai("first text")]),
            parse_quote!(#[ai("second text")]),
        ];
        assert_eq!(find_ai_attr(&attrs), Some("first text".to_string()));
    }
}
