// ---------------- [ File: src/imports.rs ]
/// Collapses all runs of whitespace in `s` into single spaces
/// and trims leading/trailing whitespace.
//pub(crate) use disable_macro::disable;
//pub(crate) use std::str::FromStr;
pub(crate) use error_tree::error_tree;
pub(crate) use export_magic::*;
pub(crate) use getset::*;
pub(crate) use hydro2_3p::*;
pub(crate) use proc_macro2::{Span,TokenStream};
pub(crate) use proc_macro::TokenStream as RawTokenStream;
pub(crate) use quote::{ToTokens, quote};
pub(crate) use std::collections::HashMap;
pub(crate) use syn::{
    Attribute,
    ConstParam,
    DeriveInput,
    Error as SynError,
    GenericArgument,
    GenericParam,
    Generics,
    Ident,
    Path,
    PathArguments,
    Type,
    TypeParam,
    WhereClause,
    parse::Parse,
    parse_macro_input,
    parse_quote,
    parse_str,
    spanned::Spanned,
};

pub(crate) use traced_test::traced_test;
pub(crate) use tracing::*;
pub(crate) use tracing_setup::*;

#[cfg(test)]
pub fn normalize_whitespace(s: &str) -> String { s.split_whitespace().collect::<Vec<_>>().join(" ") }
