// ---------------- [ File: ai-json-template-derive/src/imports.rs ]
pub(crate) use proc_macro::TokenStream;
pub(crate) use proc_macro2::{Span,TokenStream as TokenStream2};
pub(crate) use quote::{quote};
pub(crate) use export_magic::*;
pub(crate) use tracing::{info,warn,error,trace,debug};
pub(crate) use getset::*;

#[cfg(test)] pub(crate) use tracing_setup::{EventPrinter,colored,Flushable,setup_buffered_tracing};
#[cfg(test)] pub(crate) use traced_test::traced_test;
#[cfg(test)] pub(crate) use pretty_assertions::assert_eq as pretty_assert_eq;
#[cfg(test)] pub(crate) use syn::{parse_quote};
pub(crate) use derive_builder::*;

pub(crate) use syn::{
    parse_macro_input,
    Data,
    DataStruct,
    DeriveInput,
    Fields,
    Attribute,
    MetaNameValue,
    Expr,
    ExprLit,
    ExprCall,
    ExprAssign,
    ExprParen,
    parse2,
    MetaList,
    Meta,
    Lit,
    Ident,
    spanned::Spanned,
    punctuated::Punctuated,
    Token,
    parse::Parse,
    parse::ParseStream,
    token::Colon,
    Field,
    FieldsNamed,
    FieldsUnnamed,
    DataEnum,
    Visibility,
    Variant,
    FieldMutability,
    Type,
};
