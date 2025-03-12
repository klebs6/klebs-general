// ---------------- [ File: src/comma_separated_expression.rs ]
crate::ix!();

/// A wrapper for a comma-separated list of expressions.
#[derive(Getters,Debug)]
#[getset(get="pub")]
pub struct CommaSeparatedExpressions {
    // Using getset for field access if necessary, but kept private.
    expressions: Punctuated<Expr, Token![,]>,
}

impl Parse for CommaSeparatedExpressions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        trace!("Starting to parse comma-separated expressions.");
        let expressions = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        debug!("Successfully parsed {} expression(s).", expressions.len());
        Ok(CommaSeparatedExpressions { expressions })
    }
}


#[cfg(test)]
mod validate_comma_separated_expressions {
    use super::*;

    #[traced_test]
    fn test_empty_comma_separated_expressions() {
        let input: proc_macro2::TokenStream = "".parse().expect("Failed to parse input tokens");
        let result = syn::parse2::<CommaSeparatedExpressions>(input);
        assert!(
            result.is_ok(),
            "Parsing an empty input should succeed and produce an empty expression list"
        );
        let list = result.expect("Expected an empty expression list");
        assert_eq!(
            list.expressions.len(),
            0,
            "The expression list should be empty for empty input"
        );
    }

    #[traced_test]
    fn test_multiple_comma_separated_expressions() {
        let input: proc_macro2::TokenStream = "1, 2, 3".parse().expect("Failed to parse input tokens");
        let result = syn::parse2::<CommaSeparatedExpressions>(input);
        assert!(
            result.is_ok(),
            "Parsing multiple expressions should succeed and yield the correct count"
        );
        let list = result.expect("Expected a list of expressions");
        assert_eq!(
            list.expressions.len(),
            3,
            "The expression list should contain exactly three expressions"
        );
    }
}
