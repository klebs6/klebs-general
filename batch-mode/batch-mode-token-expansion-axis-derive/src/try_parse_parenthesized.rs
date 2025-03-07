// ---------------- [ File: token-expander-axis-derive/src/try_parse_parenthesized.rs ]
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

/// Attempt to parse parentheses style:
/// `#[system_message_goal("some text")]`.
pub fn try_parse_parenthesized(attr: &Attribute) -> SynResult<Option<LitStr>> {
    // parse_args_with(ParenthesizedMessage) will expect exactly
    // parentheses containing a string literal. e.g. `("some text")`
    match attr.parse_args_with(ParenthesizedMessage::parse) {
        Ok(val) => Ok(Some(val.msg)),
        Err(_) => Ok(None), // not parentheses style
    }
}
