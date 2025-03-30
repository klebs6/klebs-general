// ---------------- [ File: src/parse.rs ]
crate::ix!();

pub fn parse_probability_literal(lit: &Lit) -> f64 {
    match lit {
        Lit::Float(lit_float) => lit_float.base10_parse::<f64>().unwrap(),
        Lit::Int(lit_int) => lit_int.base10_parse::<f64>().unwrap(),
        _ => panic!("Expected a float or int literal in rand_construct(p = ...)"),
    }
}

pub fn is_option_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}

pub fn parse_some_probability(attrs: &[syn::Attribute]) -> Option<f64> {
    for attr in attrs {
        if attr.path.is_ident("rand_construct") {
            let meta = attr.parse_meta().ok()?;
            if let syn::Meta::List(meta_list) = meta {
                for nested_meta in meta_list.nested {
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = nested_meta {
                        if nv.path.is_ident("psome") {
                            if let syn::Lit::Float(lit_float) = &nv.lit {
                                return lit_float.base10_parse::<f64>().ok();
                            } else if let syn::Lit::Int(lit_int) = &nv.lit {
                                return lit_int.base10_parse::<f64>().ok();
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
pub fn token_stream_to_string(ts: &TokenStream2) -> String {
    ts.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::Lit;

    #[test]
    fn test_parse_probability_literal_with_float() {
        // Create a float literal
        let lit = Lit::Float(syn::LitFloat::new("3.14", proc_macro2::Span::call_site()));
        let result = parse_probability_literal(&lit);
        assert_eq!(result, 3.14);
    }

    #[test]
    fn test_parse_probability_literal_with_int() {
        // Create an int literal
        let lit = Lit::Int(syn::LitInt::new("42", proc_macro2::Span::call_site()));
        let result = parse_probability_literal(&lit);
        assert_eq!(result, 42.0);
    }

    #[test]
    #[should_panic(expected = "Expected a float or int literal in rand_construct(p = ...)")]
    fn test_parse_probability_literal_with_invalid_literal() {
        // Create a string literal (invalid input)
        let lit = Lit::Str(syn::LitStr::new("not_a_number", proc_macro2::Span::call_site()));
        parse_probability_literal(&lit); // This should panic
    }
}
