// ---------------- [ File: token-expander-axis-derive/src/try_parse_name_value.rs ]
crate::ix!();

/// A small helper struct for `#[system_message_goal = "some text"]`.
pub struct SystemMessageGoalNameValue {
    eq: Token![=],
    msg: LitStr,
}

impl Parse for SystemMessageGoalNameValue {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Expect `=` then a string literal
        let eq: Token![=] = input.parse()?;
        let msg: LitStr = input.parse()?;
        Ok(Self { eq, msg })
    }
}

/// Attempt to parse name-value style:
/// `#[system_message_goal = "some text"]`.
pub fn try_parse_name_value(attr: &Attribute) -> SynResult<Option<LitStr>> {
    // Parse just the arguments to the attribute (the stuff after `#[system_message_goal`)
    // using our small parse struct above.
    let parsed = attr.parse_args_with(SystemMessageGoalNameValue::parse);

    match parsed {
        Ok(val) => Ok(Some(val.msg)), // we got a string
        Err(_) => Ok(None),           // not name-value style
    }
}

