// ---------------- [ File: ai-json-template-derive/src/is_justification_enabled.rs ]
crate::ix!();

/// If `#[justify=false]`, then `is_justification_enabled` returns false.
/// Otherwise, returns true.
#[inline]
pub fn is_justification_enabled(field: &syn::Field) -> bool {
    !is_justification_disabled_for_field(field)
}

/// Check if `#[justify=false]` is present on the field.
/// Means “skip justification/confidence placeholders *for this field itself*.”
#[inline]
pub fn is_justification_disabled_for_field(field: &syn::Field) -> bool {
    for attr in &field.attrs {
        if attr.path().is_ident("justify") {
            match &attr.meta {
                // looking for `#[justify = false]`
                syn::Meta::NameValue(syn::MetaNameValue {
                    value: syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(lb), ..
                    }),
                    ..
                }) => {
                    if lb.value() == false {
                        tracing::trace!("Found #[justify = false] on field {:?}", field.ident);
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    false
}

/// Check if `#[justify_inner=false]` is present on the field.
/// Means “call AiJsonTemplate (not AiJsonTemplateWithJustification) for the child type.”
pub fn is_justification_disabled_for_inner(field: &syn::Field) -> bool {
    for attr in &field.attrs {
        if attr.path().is_ident("justify_inner") {
            match &attr.meta {
                // looking for `#[justify_inner = false]`
                syn::Meta::NameValue(syn::MetaNameValue {
                    value: syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(lb), ..
                    }),
                    ..
                }) => {
                    if lb.value() == false {
                        tracing::trace!("Found #[justify_inner = false] on field {:?}", field.ident);
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    false
}

/// Check if `#[justify=false]` is present on this variant. 
/// Means we skip "variant_justification"/"variant_confidence" placeholders
/// as well as skip calling `AiJsonTemplateWithJustification` on fields, 
/// falling back to `AiJsonTemplate`.
pub fn is_justification_disabled_for_variant(variant: &syn::Variant) -> bool {
    for attr in &variant.attrs {
        if attr.path().is_ident("justify") {
            if let syn::Meta::NameValue(syn::MetaNameValue {
                value: syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Bool(lb), ..
                }),
                ..
            }) = &attr.meta {
                if lb.value() == false {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if `#[justify_inner=false]` is present on this variant.
/// Means we do produce "variant_justification"/"variant_confidence" placeholders
/// for the variant itself, but the variant's fields are generated using 
/// `AiJsonTemplate::to_template()` instead of 
/// `AiJsonTemplateWithJustification::to_template_with_justification()`.
pub fn is_justification_disabled_for_inner_variant(variant: &syn::Variant) -> bool {
    for attr in &variant.attrs {
        if attr.path().is_ident("justify_inner") {
            if let syn::Meta::NameValue(syn::MetaNameValue {
                value: syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Bool(lb), ..
                }),
                ..
            }) = &attr.meta {
                if lb.value() == false {
                    return true;
                }
            }
        }
    }
    false
}
