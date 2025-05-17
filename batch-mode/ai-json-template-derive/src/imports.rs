// ---------------- [ File: ai-json-template-derive/src/imports.rs ]
pub(crate) use proc_macro::TokenStream;
pub(crate) use proc_macro2::{Literal,Span,TokenStream as TokenStream2};

pub(crate) use export_magic::*;
pub(crate) use tracing::{info,warn,error,trace,debug};
pub(crate) use getset::*;

#[cfg(test)] pub(crate) use tracing_setup::{EventPrinter,colored,Flushable,setup_buffered_tracing};
#[cfg(test)] pub(crate) use traced_test::traced_test;
#[cfg(test)] pub(crate) use pretty_assertions::assert_eq as pretty_assert_eq;
#[cfg(test)] pub(crate) use syn::{parse_quote};

pub(crate) use derive_builder::*;
pub(crate) use disable_macro::disable;

#[allow(unused_imports)]
pub(crate) use quote::{ToTokens,quote};

#[allow(unused_imports)]
pub(crate) use syn::{
    parse_str,
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
    ItemEnum,
    ItemImpl,
    Item,
    ItemStruct,
    token::Brace,
    token::PathSep,
    PathSegment,
    ExprBlock,
    ExprMethodCall,
    Stmt,
    Block,
};
pub(crate) use serde_json::json;

/// A small helper that tries to parse generated Rust code with `syn`
/// and fails the test if `syn` returns an error.
/// This ensures expansions didn't produce partial tokens like "count:: u32".
pub fn assert_tokens_parse_ok(ts: &proc_macro2::TokenStream) {
    let code_str = ts.to_string();
    // Attempt to parse it as a top-level file (or item).
    // If there's a syntax error (like "count:: u32"), parsing should fail.
    let parse_result: syn::Result<syn::File> = parse_str(&code_str);
    if let Err(e) = parse_result {
        panic!(
            "Generated tokens failed to parse!\nError: {}\n\nGenerated code:\n{}",
            e, code_str
        );
    }
}
