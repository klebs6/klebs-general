// ---------------- [ File: batch-mode-token-expansion-axis-derive/src/try_parse_name_value.rs ]
crate::ix!();

/// A small helper struct for `#[system_message_goal = "some text"]`.
pub struct NameValue {
    _eq: Token![=],
    msg: LitStr,
}

impl Parse for NameValue {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Expect `=` then a string literal
        let _eq: Token![=] = input.parse()?;
        let msg: LitStr = input.parse()?;
        Ok(Self { _eq, msg })
    }
}

#[tracing::instrument(level = "trace", skip(attr))]
pub fn try_parse_name_value(attr: &Attribute) -> SynResult<Option<LitStr>> {
    /*
       Syn 2.0 no longer provides `attr.parse_meta()` or `attr.tokens`.
       Instead we use the public `attr.meta` field (Option<syn::Meta>).

       We want to parse attributes of the form:
           #[system_message_goal = "some text"]
       which in Syn 2.0 should appear as `Meta::NameValue(MetaNameValue { path, value, .. })`.
    */

    let meta_opt = &attr.meta;
    if let syn::Meta::NameValue(mnv) = meta_opt {
        // Check the path is ident "system_message_goal"
        if mnv.path.is_ident("system_message_goal") {
            // The value is an expression; in this pattern we expect a string literal.
            // Example: #[system_message_goal = "Name-Value Goal"]
            if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(ref s), .. }) = mnv.value {
                debug!("Parsed system_message_goal via name-value syntax: {:?}", s.value());
                return Ok(Some(s.clone()));
            }
            trace!("NameValue was found, but not a string literal.");
        }
    }

    trace!("Either not NameValue or path not 'system_message_goal' or not string literal; returning None.");
    Ok(None)
}
