//! named-item-derive — a derive macro for implementing `Named`, `SetName`, etc.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, Data, Fields, Error as SynError,
    LitStr,
};

/// The attribute macro to derive Named, DefaultName, SetName, etc. behaviors.
#[proc_macro_derive(NamedItem, attributes(named_item))]
pub fn derive_named_item(input: TokenStream) -> TokenStream {

    let ast = parse_macro_input!(input as DeriveInput);

    // Parse the user-provided #[named_item(...)] attributes
    let config = match parse_named_item_attrs(&ast) {
        Ok(cfg) => cfg,
        Err(e) => return e.to_compile_error().into(),
    };

    // Generate the final code
    match impl_named_item(&ast, &config) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Configuration extracted from `#[named_item(...)]`.
struct NamedItemConfig {
    /// Optional default name if `default_name="foo"`.
    default_name: Option<String>,
    /// If `aliases="true"`, the struct must have `aliases: Vec<String>`.
    aliases: bool,
    /// If `default_aliases="foo,bar"`, we store them here.
    default_aliases: Vec<String>,
    /// If `history="true"`, the struct must have `name_history: Vec<String>`.
    history: bool,
}

fn parse_named_item_attrs(ast: &DeriveInput) -> syn::Result<NamedItemConfig> {
    let mut default_name = None;
    let mut aliases = false;
    let mut default_aliases = Vec::new();
    let mut history = false;

    for attr in &ast.attrs {
        if attr.path().is_ident("named_item") {
            // parse_nested_meta helps parse name="value" pairs
            attr.parse_nested_meta(|meta| {
                let p = &meta.path;
                if p.is_ident("default_name") {
                    let lit: LitStr = meta.value()?.parse()?;
                    default_name = Some(lit.value());
                } else if p.is_ident("aliases") {
                    let lit: LitStr = meta.value()?.parse()?;
                    aliases = lit.value().to_lowercase() == "true";
                } else if p.is_ident("default_aliases") {
                    let lit: LitStr = meta.value()?.parse()?;
                    default_aliases = lit
                        .value()
                        .split(',')
                        .filter(|tok| !tok.trim().is_empty())
                        .map(|s| s.trim().to_string())
                        .collect();
                } else if p.is_ident("history") {
                    let lit: LitStr = meta.value()?.parse()?;
                    history = lit.value().to_lowercase() == "true";
                }
                Ok(())
            })?;
        }
    }

    Ok(NamedItemConfig {
        default_name,
        aliases,
        default_aliases,
        history,
    })
}

/// Generate the trait implementations for the given struct:
/// - Named
/// - DefaultName
/// - ResetName
/// - SetName
/// - NameHistory (optionally)
/// - NamedAlias (optionally)
fn impl_named_item(ast: &DeriveInput, cfg: &NamedItemConfig) -> syn::Result<TokenStream> {
    let struct_name = &ast.ident;

    // ### 1) Capture generics
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // ### 2) We only support normal "struct { name: String, ... }"
    let fields = match &ast.data {
        Data::Struct(ds) => &ds.fields,
        _ => {
            return Err(SynError::new_spanned(
                &ast.ident,
                "NamedItem can only be derived on a struct.",
            ));
        }
    };

    let named_fields = match fields {
        Fields::Named(f) => &f.named,
        _ => {
            return Err(SynError::new_spanned(
                &ast.ident,
                "NamedItem requires a struct with named fields.",
            ));
        }
    };

    // Must have `name: String`
    let name_field = named_fields.iter().find(|field| {
        field.ident.as_ref().map(|id| id == "name").unwrap_or(false)
    });
    if name_field.is_none() {
        return Err(SynError::new_spanned(
            &ast.ident,
            "Struct must have `name: String`.",
        ));
    }
    let name_ty = &name_field.unwrap().ty;
    let is_string = match name_ty {
        syn::Type::Path(tp) => {
            tp.path.segments.last().map(|seg| seg.ident == "String").unwrap_or(false)
        }
        _ => false,
    };
    if !is_string {
        return Err(SynError::new_spanned(
            name_ty,
            "`name` field must be `String`",
        ));
    }

    // require name_history if history=true
    if cfg.history {
        let hist_field = named_fields.iter().find(|field| {
            field.ident.as_ref().map(|id| id == "name_history").unwrap_or(false)
        });
        if hist_field.is_none() {
            return Err(SynError::new_spanned(
                &ast.ident,
                "history=true but no `name_history: Vec<String>` field found.",
            ));
        }
    }

    // require aliases if aliases=true
    if cfg.aliases {
        let alias_field = named_fields.iter().find(|field| {
            field.ident.as_ref().map(|id| id == "aliases").unwrap_or(false)
        });
        if alias_field.is_none() {
            return Err(SynError::new_spanned(
                &ast.ident,
                "aliases=true but no `aliases: Vec<String>` field found.",
            ));
        }
    }

    // Fallback name if none is provided
    let fallback_name = cfg.default_name.clone().unwrap_or_else(|| struct_name.to_string());

    // ### 3) Generate the "baseline" Named, DefaultName, ResetName
    let baseline_impl = quote! {
        impl #impl_generics Named for #struct_name #ty_generics #where_clause {
            fn name(&self) -> std::borrow::Cow<'_, str> {
                std::borrow::Cow::from(&self.name)
            }
        }

        impl #impl_generics DefaultName for #struct_name #ty_generics #where_clause {
            fn default_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::from(#fallback_name)
            }
        }

        impl #impl_generics ResetName for #struct_name #ty_generics #where_clause {}
    };

    // ### 4) If we have history=true, we push to `name_history` each time we rename
    let setname_impl = if cfg.history {
        quote! {
            impl #impl_generics SetName for #struct_name #ty_generics #where_clause {
                fn set_name(&mut self, name: &str) -> Result<(), NameError> {
                    // push history first
                    self.name_history.push(name.to_string());

                    // forbid empty if not default
                    if name.is_empty() && name != &*Self::default_name() {
                        return Err(NameError::EmptyName);
                    }
                    self.name = name.to_owned();
                    Ok(())
                }
            }

            impl #impl_generics NameHistory for #struct_name #ty_generics #where_clause {
                fn add_name_to_history(&mut self, name: &str) {
                    self.name_history.push(name.to_string());
                }

                fn name_history(&self) -> Vec<std::borrow::Cow<'_, str>> {
                    self.name_history
                        .iter()
                        .map(|s| std::borrow::Cow::from(&s[..]))
                        .collect()
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics SetName for #struct_name #ty_generics #where_clause {
                fn set_name(&mut self, name: &str) -> Result<(), NameError> {
                    // forbid empty if not default
                    if name.is_empty() && name != &*Self::default_name() {
                        return Err(NameError::EmptyName);
                    }
                    self.name = name.to_owned();
                    Ok(())
                }
            }
        }
    };

    // ### 5) If aliases=true, implement NamedAlias
    let alias_impl = if cfg.aliases {
        let arr_tokens = cfg.default_aliases.iter().map(|s| quote! { #s.to_owned() });
        quote! {
            impl #impl_generics NamedAlias for #struct_name #ty_generics #where_clause {
                fn add_alias(&mut self, alias: &str) {
                    self.aliases.push(alias.to_string());
                }
                fn aliases(&self) -> Vec<std::borrow::Cow<'_, str>> {
                    self.aliases
                        .iter()
                        .map(|s| std::borrow::Cow::from(&s[..]))
                        .collect()
                }
                fn clear_aliases(&mut self) {
                    self.aliases.clear();
                }
            }

            impl #impl_generics #struct_name #ty_generics #where_clause {
                pub fn default_aliases() -> Vec<std::borrow::Cow<'static, str>> {
                    vec![
                        #(std::borrow::Cow::from(#arr_tokens)),*
                    ]
                }
            }
        }
    } else {
        quote!()
    };

    // ### 6) Combine expansions
    let expanded = quote! {
        #baseline_impl
        #setname_impl
        #alias_impl
    };

    Ok(expanded.into())
}
