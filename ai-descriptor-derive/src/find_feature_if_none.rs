// ---------------- [ File: ai-descriptor-derive/src/find_feature_if_none.rs ]
crate::ix!();

pub(crate) fn find_feature_if_none(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path.is_ident("ai") {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                for nested_meta in meta_list.nested {
                    if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit: Lit::Str(lit_str), .. })) = nested_meta {
                        if path.is_ident("feature_if_none") {
                            return Some(lit_str.value());
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Attribute};

    #[test]
    fn test_find_feature_if_none_with_valid_attribute() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai(feature_if_none = "No side effects.")])];
        assert_eq!(find_feature_if_none(&attrs), Some("No side effects.".to_string()));
    }

    #[test]
    fn test_find_feature_if_none_with_non_ai_attribute() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[other(feature_if_none = "Text")])];
        assert_eq!(find_feature_if_none(&attrs), None);
    }

    #[test]
    fn test_find_feature_if_none_with_different_key() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai(other_key = "Text")])];
        assert_eq!(find_feature_if_none(&attrs), None);
    }

    #[test]
    fn test_find_feature_if_none_with_incorrect_format() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai("Text")])];
        assert_eq!(find_feature_if_none(&attrs), None);
    }

    #[test]
    fn test_find_feature_if_none_with_invalid_literal() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai(feature_if_none = 123)])];
        assert_eq!(find_feature_if_none(&attrs), None);
    }

    #[test]
    fn test_find_feature_if_none_with_empty_attributes() {
        let attrs: Vec<Attribute> = vec![];
        assert_eq!(find_feature_if_none(&attrs), None);
    }

    #[test]
    fn test_find_feature_if_none_with_multiple_attributes() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[other_key("Something")]),
            parse_quote!(#[ai(feature_if_none = "No side effects.")]),
        ];
        assert_eq!(find_feature_if_none(&attrs), Some("No side effects.".to_string()));
    }

    #[test]
    fn test_find_feature_if_none_with_multiple_ai_attributes() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[ai(other_key = "Text")]),
            parse_quote!(#[ai(feature_if_none = "First match")]),
            parse_quote!(#[ai(feature_if_none = "Second match")]),
        ];
        assert_eq!(find_feature_if_none(&attrs), Some("First match".to_string()));
    }
}
