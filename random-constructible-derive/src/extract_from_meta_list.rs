// ---------------- [ File: src/extract_from_meta_list.rs ]
crate::ix!();

pub fn extract_probability_from_meta_list(meta_list: &syn::MetaList) 
    -> Option<f64> 
{
    for nested_meta in &meta_list.nested {
        if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. })) = nested_meta {
            if path.is_ident("p") {
                return Some(parse_probability_literal(lit));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Meta, MetaList, NestedMeta, Path};

    #[test]
    fn test_extract_probability_with_valid_probability() {
        let meta_list = MetaList {
            path: Path::from(syn::Ident::new("test", proc_macro2::Span::call_site())),
            paren_token: syn::token::Paren(proc_macro2::Span::call_site()),
            nested: syn::punctuated::Punctuated::from_iter(vec![
                NestedMeta::Meta(Meta::NameValue(syn::MetaNameValue {
                    path: Path::from(syn::Ident::new("p", proc_macro2::Span::call_site())),
                    lit: syn::Lit::Float(syn::LitFloat::new("0.75", proc_macro2::Span::call_site())),
                    eq_token: syn::token::Eq(proc_macro2::Span::call_site()),
                })),
            ]),
        };

        let result = extract_probability_from_meta_list(&meta_list);
        assert_eq!(result, Some(0.75));
    }

    #[test]
    fn test_extract_probability_with_missing_probability() {
        let meta_list = MetaList {
            path: Path::from(syn::Ident::new("test", proc_macro2::Span::call_site())),
            paren_token: syn::token::Paren(proc_macro2::Span::call_site()),
            nested: syn::punctuated::Punctuated::from_iter(vec![
                NestedMeta::Meta(Meta::NameValue(syn::MetaNameValue {
                    path: Path::from(syn::Ident::new("q", proc_macro2::Span::call_site())), // Not "p"
                    lit: syn::Lit::Float(syn::LitFloat::new("0.75", proc_macro2::Span::call_site())),
                    eq_token: syn::token::Eq(proc_macro2::Span::call_site()),
                })),
            ]),
        };

        let result = extract_probability_from_meta_list(&meta_list);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_probability_with_no_meta_name_value() {
        let meta_list = MetaList {
            path: Path::from(syn::Ident::new("test", proc_macro2::Span::call_site())),
            paren_token: syn::token::Paren(proc_macro2::Span::call_site()),
            nested: syn::punctuated::Punctuated::new(), // No nested items
        };

        let result = extract_probability_from_meta_list(&meta_list);
        assert_eq!(result, None);
    }
}
