pub(crate) use export_magic::*;
pub(crate) use proc_macro::TokenStream;
pub(crate) use quote::quote;
pub(crate) use syn::parse_macro_input;
pub(crate) use syn::spanned::Spanned;
pub(crate) use syn::{
    Attribute, 
    Data, 
    DeriveInput, 
    Expr, 
    ExprAssign, 
    ExprLit, 
    ExprPath, 
    Lit, 
    Variant,
};
