use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, Data, Fields, Error as SynError,
    LitStr,
};

//=======================================================
// NamedItem Derive Macro
//=======================================================

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

/// Holds configuration extracted from `#[named_item(...)]`.
struct NamedItemConfig {
    default_name: Option<String>,
    aliases: bool,
    default_aliases: Vec<String>,
    history: bool,
}

/// Parses attribute usage like:
///
/// ```rust
/// use named_item_derive::NamedItem;
/// use named_item::{Named, SetName,DefaultName};
///
/// #[derive(NamedItem)]
/// #[named_item(
///     default_name="...",
///     aliases="true",
///     default_aliases="foo,bar",
///     history="true"
/// )]
/// struct Demo {
///     name: String,
///     name_history: Vec<String>,
///     aliases: Vec<String>,
/// }
///
/// fn main() {
///     let mut x = Demo {
///         name: "initial".into(),
///         name_history: vec![],
///         aliases: vec![],
///     };
///     // Now we can call set_name, name_history, etc.
///     x.set_name("updated").unwrap();
///     assert_eq!(x.name(), "updated");
/// }
/// ```
fn parse_named_item_attrs(ast: &DeriveInput) -> syn::Result<NamedItemConfig> {
    let mut default_name = None;
    let mut aliases = false;
    let mut default_aliases = Vec::new();
    let mut history = false;

    for attr in &ast.attrs {
        if attr.path().is_ident("named_item") {
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
                    // FILTER OUT empty tokens:
                    default_aliases = lit
                        .value()
                        .split(',')
                        .filter(|tok| !tok.trim().is_empty())   // <-- NEW
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

fn impl_named_item(ast: &DeriveInput, cfg: &NamedItemConfig) -> syn::Result<TokenStream> {
    let struct_name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(ds) => &ds.fields,
        _ => {
            return Err(SynError::new_spanned(
                &ast.ident,
                "NamedItem can only be derived for a struct.",
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

    // must have name: String
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

    let fallback_name = cfg.default_name.clone().unwrap_or_else(|| struct_name.to_string());

    // Named, DefaultName, ResetName
    let baseline_impl = quote! {
        impl named_item::Named for #struct_name {
            fn name(&self) -> std::borrow::Cow<'_, str> {
                std::borrow::Cow::from(&self.name)
            }
        }

        impl named_item::DefaultName for #struct_name {
            fn default_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::from(#fallback_name)
            }
        }

        impl named_item::ResetName for #struct_name {}
    };

    // SetName:
    // If we want to allow empty name if it equals the default, do so:
    let setname_impl = if cfg.history {
        quote! {
            impl named_item::SetName for #struct_name {
                fn set_name(&mut self, name: &str) -> Result<(), named_item::NameError> {
                    // push history first
                    self.name_history.push(name.to_string());

                    // only error if empty and not the default
                    if name.is_empty() && name != &*Self::default_name() {
                        return Err(named_item::NameError::EmptyName);
                    }
                    self.name = name.to_owned();
                    Ok(())
                }
            }

            impl named_item::NameHistory for #struct_name {
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
        // no history
        quote! {
            impl named_item::SetName for #struct_name {
                fn set_name(&mut self, name: &str) -> Result<(), named_item::NameError> {
                    // only error if empty and not the default
                    if name.is_empty() && name != &*Self::default_name() {
                        return Err(named_item::NameError::EmptyName);
                    }
                    self.name = name.to_owned();
                    Ok(())
                }
            }
        }
    };

    // NamedAlias if aliases=true
    let alias_impl = if cfg.aliases {
        let arr_tokens = cfg.default_aliases.iter().map(|s| quote! { #s.to_owned() });
        quote! {
            impl named_item::NamedAlias for #struct_name {
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

            impl #struct_name {
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

    let expanded = quote! {
        #baseline_impl
        #setname_impl
        #alias_impl
    };

    Ok(expanded.into())
}
