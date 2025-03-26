// ---------------- [ File: src/syn_imports.rs ]
pub(crate) use syn::{
    Token,
    spanned::Spanned,
    parse::{
        Parse as SynParse, 
        ParseStream, 
        Result as SynParseResult
    },
    punctuated::Punctuated,
    parse_macro_input, 
    Attribute, 
    ItemFn,
    Ident, 
    LitStr,
    Lit,
};
