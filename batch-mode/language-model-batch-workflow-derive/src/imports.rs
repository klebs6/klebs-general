// ---------------- [ File: src/imports.rs ]
pub(crate) use batch_mode_3p::*;
pub(crate) use proc_macro::TokenStream;
pub(crate) use proc_macro2::{TokenStream as TokenStream2};
pub(crate) use quote::{quote};

pub(crate) use syn::{
    DeriveInput,
    Type::Path as TypePath,
    GenericArgument::Type as GAType,
    PathArguments::AngleBracketed,
    TraitBound,
};

#[cfg(test)]
pub(crate) use syn::{parse_quote};
