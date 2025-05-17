// ---------------- [ File: ai-json-template-derive/src/is_justification_enabled.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn is_justification_disabled_for_field(field: &syn::Field) -> bool {
    trace!("Checking if #[justify=false] is present on field {:?}", field.ident);

    for attr in &field.attrs {
        trace!("Inspecting attribute: {:?}", attr.path().segments.last().map(|s| s.ident.to_string()));
        if attr.path().is_ident("justify") {
            match &attr.meta {
                syn::Meta::NameValue(syn::MetaNameValue {
                    value: syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(lb), ..
                    }),
                    ..
                }) => {
                    trace!("Found a justify attribute with boolean literal = {}", lb.value());
                    if lb.value() == false {
                        trace!("=> #[justify = false] found on field {:?}", field.ident);
                        return true;
                    }
                }
                other => {
                    debug!("Skipping non-boolean-literal justify attribute: {:?}", other);
                }
            }
        }
    }

    trace!("=> #[justify=false] not found on field {:?}", field.ident);
    false
}

#[tracing::instrument(level = "trace", skip_all)]
pub fn is_justification_disabled_for_inner(field: &syn::Field) -> bool {
    trace!("Checking if #[justify_inner=false] is present on field {:?}", field.ident);

    for attr in &field.attrs {
        trace!("Inspecting attribute: {:?}", attr.path().segments.last().map(|s| s.ident.to_string()));
        if attr.path().is_ident("justify_inner") {
            match &attr.meta {
                syn::Meta::NameValue(syn::MetaNameValue {
                    value: syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(lb), ..
                    }),
                    ..
                }) => {
                    trace!("Found a justify_inner attribute with boolean literal = {}", lb.value());
                    if lb.value() == false {
                        trace!("=> #[justify_inner = false] found on field {:?}", field.ident);
                        return true;
                    }
                }
                other => {
                    debug!("Skipping non-boolean-literal justify_inner attribute: {:?}", other);
                }
            }
        }
    }

    trace!("=> #[justify_inner=false] not found on field {:?}", field.ident);
    false
}

#[tracing::instrument(level = "trace", skip_all)]
pub fn is_justification_disabled_for_variant(variant: &syn::Variant) -> bool {
    trace!("Checking if #[justify=false] is present on variant {:?}", variant.ident);

    for attr in &variant.attrs {
        trace!("Inspecting attribute: {:?}", attr.path().segments.last().map(|s| s.ident.to_string()));
        if attr.path().is_ident("justify") {
            if let syn::Meta::NameValue(syn::MetaNameValue {
                value: syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Bool(lb), ..
                }),
                ..
            }) = &attr.meta {
                trace!("Found a justify attribute with boolean literal = {}", lb.value());
                if lb.value() == false {
                    trace!("=> #[justify = false] found on variant {:?}", variant.ident);
                    return true;
                }
            }
        }
    }

    trace!("=> #[justify=false] not found on variant {:?}", variant.ident);
    false
}

#[tracing::instrument(level = "trace", skip_all)]
pub fn is_justification_disabled_for_inner_variant(variant: &syn::Variant) -> bool {
    trace!("Checking if #[justify_inner=false] is present on variant {:?}", variant.ident);

    for attr in &variant.attrs {
        trace!("Inspecting attribute: {:?}", attr.path().segments.last().map(|s| s.ident.to_string()));
        if attr.path().is_ident("justify_inner") {
            if let syn::Meta::NameValue(syn::MetaNameValue {
                value: syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Bool(lb), ..
                }),
                ..
            }) = &attr.meta {
                trace!("Found a justify_inner attribute with boolean literal = {}", lb.value());
                if lb.value() == false {
                    trace!("=> #[justify_inner = false] found on variant {:?}", variant.ident);
                    return true;
                }
            }
        }
    }

    trace!("=> #[justify_inner=false] not found on variant {:?}", variant.ident);
    false
}
