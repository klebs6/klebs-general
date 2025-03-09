// ---------------- [ File: src/imports.rs ]
pub(crate) use batch_mode_3p::*;
pub(crate) use proc_macro::TokenStream;
pub(crate) use proc_macro2::{Span, TokenStream as TokenStream2};
pub(crate) use quote::{quote};

pub(crate) use syn::{
    parse_quote,
    parse_macro_input,
    DeriveInput,
    Fields,
    Meta,
    Ident,
    Generics,
    WhereClause,
    MetaList,
    Type,
    spanned::Spanned,
    // Import the specific variants we need:
    Type::Path as TypePath,
    Type::TraitObject,
    GenericArgument::Type as GAType,
    PathArguments::AngleBracketed,
    TypeParamBound,
    TraitBound,
};
