crate::ix!();

pub fn parse_probability_literal(lit: &Lit) -> f64 {
    match lit {
        Lit::Float(lit_float) => lit_float.base10_parse::<f64>().unwrap(),
        Lit::Int(lit_int) => lit_int.base10_parse::<f64>().unwrap(),
        _ => panic!("Expected a float or int literal in rand_construct(p = ...)"),
    }
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

