crate::ix!();

error_tree!{

    pub enum BuilderError {
        MissingOriginalBlock,
        MissingShouldFailAttr,
    }

    pub enum UnknownAttributeError {
        UnknownAttribute {
            attr: syn::Attribute,
        }
    }

    pub enum ShouldFailAttrError {
        // The attribute is not `should_fail`.
        NotShouldFailAttr,

        // The `message` or `trace` key is missing.
        ExpectedValueMissing,

        // The value provided for `message` or `trace` is of incorrect type.
        InvalidExpectedValueFormat,

        // Failed to parse the attribute arguments.
        MetaParseError(syn::Error),

        // Multiple `should_fail` attributes found.
        MultipleShouldFailAttrs,

        // Encountered an unknown attribute key.
        UnknownAttribute,
    }

    pub enum ShouldPanicAttrError {
        // The attribute is not `should_panic`.
        NotShouldPanicAttr,

        // The `expected` key is missing or the value is missing.
        ExpectedValueMissing,

        // The value provided for `expected` is not a string literal.
        InvalidExpectedValueFormat,

        // Failed to parse the attribute arguments.
        MetaParseError(syn::Error),

        // Multiple `should_panic` attributes found.
        MultipleShouldPanicAttrs,
    }

    pub enum TracedTestError {
        BuilderError(BuilderError),
        ShouldFailAttrError(ShouldFailAttrError),
        ShouldPanicAttrNotSupportedWithTracedTest,
        ShouldPanicAttrAccessError,
        TokenStream(TokenStream),
        LexError(proc_macro2::LexError),
        UnknownAttribute(UnknownAttributeError),
        Message(String),
        MissingOriginalBlock,
        MultipleShouldFailAttrs,
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

impl PartialEq for ShouldFailAttrError {

    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ShouldFailAttrError::NotShouldFailAttr,         ShouldFailAttrError::NotShouldFailAttr)         => true,
            (ShouldFailAttrError::ExpectedValueMissing,       ShouldFailAttrError::ExpectedValueMissing)       => true,
            (ShouldFailAttrError::InvalidExpectedValueFormat, ShouldFailAttrError::InvalidExpectedValueFormat) => true,
            (ShouldFailAttrError::MultipleShouldFailAttrs,   ShouldFailAttrError::MultipleShouldFailAttrs)   => true,

            // Ignoring the actual contents of `MetaParseError`
            (ShouldFailAttrError::MetaParseError(_),          ShouldFailAttrError::MetaParseError(_))          => true,
            _ => false,
        }
    }
}

impl Eq for ShouldFailAttrError {}
