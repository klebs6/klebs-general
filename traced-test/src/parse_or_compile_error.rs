// ---------------- [ File: src/parse_or_compile_error.rs ]
crate::ix!();

pub(crate) fn parse_or_compile_error(block: TokenStream2) -> Result<syn::Block, TracedTestError> {
    Ok(syn::parse2::<syn::Block>(block)?)
}

#[cfg(test)]
mod parse_or_compile_error_tests {
    use super::*;
    use syn::Block;
    use proc_macro2::TokenStream;
    use quote::quote;

    #[test]
    fn test_parse_valid_block() {
        // A valid block of code
        let block: TokenStream = quote! {
            {
                let x = 42;
                x + 1;
            }
        };

        let parsed_block = parse_or_compile_error(block);
        assert!(parsed_block.is_ok(), "Expected the block to parse successfully");
        assert!(!parsed_block.unwrap().stmts.is_empty(), "Expected the block to have statements");
    }

    #[test]
    fn test_parse_empty_block() {
        // An empty block of code
        let block: TokenStream = quote! {
            {}
        };

        let parsed_block = parse_or_compile_error(block);
        assert!(parsed_block.is_ok(), "Expected the empty block to parse successfully");
        assert!(parsed_block.unwrap().stmts.is_empty(), "Expected the block to be empty");
    }

    #[test]
    fn test_parse_invalid_block() {
        // An invalid block of code (syntax error)
        let block: TokenStream = quote! {
            {
                let x = 42
                x + 1;
            }
        };

        let parsed_block = parse_or_compile_error(block);
        assert!(parsed_block.is_err(), "Expected the block parsing to fail");
    }
}
