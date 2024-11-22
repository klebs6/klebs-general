crate::ix!();

pub(crate) fn has_ai_display(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path.is_ident("ai") {
            if let Ok(Meta::List(MetaList { nested, .. })) = attr.parse_meta() {
                for meta in nested {
                    if let NestedMeta::Meta(Meta::Path(path)) = meta {
                        if path.is_ident("Display") {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Attribute};

    #[test]
    fn test_has_ai_display_with_display() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai(Display)])];
        assert_eq!(has_ai_display(&attrs), true);
    }

    #[test]
    fn test_has_ai_display_without_display() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai])];
        assert_eq!(has_ai_display(&attrs), false);
    }

    #[test]
    fn test_has_ai_display_with_ai_text() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai("Text")])];
        assert_eq!(has_ai_display(&attrs), false);
    }

    #[test]
    fn test_has_ai_display_with_other_key() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[ai(other_key = "Value")])];
        assert_eq!(has_ai_display(&attrs), false);
    }

    #[test]
    fn test_has_ai_display_with_multiple_attributes() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[ai("Text")]),
            parse_quote!(#[ai(Display)]),
        ];
        assert_eq!(has_ai_display(&attrs), true);
    }

    #[test]
    fn test_has_ai_display_with_empty_attributes() {
        let attrs: Vec<Attribute> = vec![];
        assert_eq!(has_ai_display(&attrs), false);
    }
}
