// ---------------- [ File: batch-mode-token-expansion-axis-derive/src/try_parse_parenthesized.rs ]
crate::ix!();

/// A helper struct to parse `("some text")` inside the attribute.
pub struct ParenthesizedMessage {
    // We'll store the parentheses tokens but won't really need them.
    _paren: syn::token::Paren,
    msg: LitStr,
}

impl Parse for ParenthesizedMessage {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let content;
        // parse the surrounding parentheses
        let paren = parenthesized!(content in input);
        // parse a single string literal from within those parentheses
        let msg: LitStr = content.parse()?;
        Ok(Self { _paren: paren, msg })
    }
}

#[tracing::instrument(level = "trace", skip(attr))]
pub fn try_parse_parenthesized(attr: &Attribute) -> SynResult<Option<LitStr>> {
    /*
       We want to parse attributes of the form:
           #[system_message_goal("some text")]
       which in Syn 2.0 appears as `Meta::List(MetaList { path, tokens, .. })`.

       The attribute's `tokens` will be `("some text")`, so we can parse it as a single LitStr.
    */

    let meta_opt = &attr.meta;
    if let syn::Meta::List(list) = meta_opt {
        // Check path
        if list.path.is_ident("system_message_goal") {
            // Attempt to parse the token content as a single string literal
            if list.tokens.is_empty() {
                trace!("List tokens are empty; cannot parse parentheses string.");
                return Ok(None);
            }
            match syn::parse2::<syn::LitStr>(list.tokens.clone()) {
                Ok(lit_str) => {
                    debug!("Parsed system_message_goal via parentheses form: {:?}", lit_str.value());
                    return Ok(Some(lit_str));
                }
                Err(e) => {
                    trace!("Failed to parse parentheses-literal: {}", e);
                    return Ok(None);
                }
            }
        }
    }

    trace!("Either not Meta::List or path not 'system_message_goal'; returning None.");
    Ok(None)
}
