// ---------------- [ File: src/syn_imports.rs ]
pub(crate) use syn::{
    parse::{Parse as SynParse, ParseStream, Result as SynParseResult},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Ident, ItemFn, Lit, LitStr, Token,
};
