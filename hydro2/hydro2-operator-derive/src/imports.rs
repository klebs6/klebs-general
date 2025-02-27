// ---------------- [ File: hydro2-operator-derive/src/imports.rs ]

pub(crate) use derive_builder::Builder;
pub(crate) use export_magic::*;
pub(crate) use getset::*;
pub(crate) use hydro2_3p::*;
pub(crate) use proc_macro2::Span;
pub(crate) use proc_macro2::TokenStream;
pub(crate) use quote::{ToTokens,quote,format_ident};
pub(crate) use std::collections::BTreeMap;
pub(crate) use syn::{Attribute, Data, DataStruct, DeriveInput, Error as SynError, Field, Fields, GenericArgument, GenericParam, Ident, Lifetime, LifetimeParam, Lit, LitInt, LitStr, Meta, Path, PathArguments, Token, Type, parse::{Parse,ParseStream}, parse_macro_input, parse_quote, spanned::Spanned, };

pub trait IsUnitType {
    fn is_unit_type(&self) -> bool;
}

/// Collapses all runs of whitespace in `s` into single spaces
/// and trims leading/trailing whitespace.
#[cfg(test)] pub fn normalize_whitespace(s: &str) -> String { s.split_whitespace().collect::<Vec<_>>().join(" ") }

pub fn ty_to_canonical_string(ty: &syn::Type) -> String {
    let raw = quote::quote! { #ty }.to_string();
    raw.split_whitespace().collect()
}

impl IsUnitType for syn::Type {
    fn is_unit_type(&self) -> bool {

        if ty_to_canonical_string(self) == "()".to_string() {
            return true;
        }
        false
    }
}
