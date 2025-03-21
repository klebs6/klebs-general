// ---------------- [ File: src/try_parse_parenthesized.rs ]
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
    trace!("Attempting parentheses/single-literal parse for attribute: {:?}", attr);

    // Must be system_message_goal
    if !attr.path().is_ident("system_message_goal") {
        return Ok(None);
    }

    // For parentheses style, we can parse a single literal string out of the attribute.
    match attr.parse_args::<LitStr>() {
        Ok(lit_str) => {
            debug!("Successfully parsed parentheses style literal: {:?}", lit_str.value());
            Ok(Some(lit_str))
        }
        Err(e) => {
            trace!("Failed parentheses-literal parse: {}", e);
            Ok(None)
        }
    }
}
