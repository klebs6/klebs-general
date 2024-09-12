crate::ix!();

error_tree!{

    pub enum ShouldPanicAttrError {

        // The attribute is not `should_panic`.
        NotShouldPanicAttr,

        // Failed to parse the meta information of the attribute.
        MetaParseError(syn::Error),

        // The `expected` key is missing.
        ExpectedValueMissing,

        // The value provided for `expected` is not a string literal.
        InvalidExpectedValueFormat,

        // Multiple `should_panic` attributes found.
        MultipleShouldPanicAttrs,
    }

    pub enum TracedTestError {
        ShouldPanicAttrError(ShouldPanicAttrError),
        TokenStream(TokenStream),
        Message(String),
        LexError(proc_macro2::LexError),
    }
}

impl PartialEq for ShouldPanicAttrError {

    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ShouldPanicAttrError::NotShouldPanicAttr,         ShouldPanicAttrError::NotShouldPanicAttr)         => true,
            (ShouldPanicAttrError::ExpectedValueMissing,       ShouldPanicAttrError::ExpectedValueMissing)       => true,
            (ShouldPanicAttrError::InvalidExpectedValueFormat, ShouldPanicAttrError::InvalidExpectedValueFormat) => true,
            (ShouldPanicAttrError::MultipleShouldPanicAttrs,   ShouldPanicAttrError::MultipleShouldPanicAttrs)   => true,

            // Ignoring the actual contents of `MetaParseError`
            (ShouldPanicAttrError::MetaParseError(_),          ShouldPanicAttrError::MetaParseError(_))          => true,
            _ => false,
        }
    }
}

impl Eq for ShouldPanicAttrError {}

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
