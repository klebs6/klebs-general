// ---------------- [ File: src/try_parse_name_value.rs ]
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
    // We expect something like: #[system_message_goal = "Some text"]
    // in which case attr.path() is "system_message_goal",
    // and attr.tokens should look like `= "Some text"`.

    if !attr.path().is_ident("system_message_goal") {
        return Ok(None);
    }

    // Grab the raw tokens after the attribute path:
    let tokens = attr.tokens.clone();
    let token_string = tokens.to_string();
    trace!("Name-value tokens for system_message_goal: {:?}", token_string);

    // If it doesn't start with '=' then it's not in name-value form:
    if !token_string.trim_start().starts_with('=') {
        trace!("No '=' found at start of tokens, so not name-value style.");
        return Ok(None);
    }

    // Attempt to parse as `= <string_literal>` using our NameValue struct:
    let parsed = match syn::parse2::<NameValue>(tokens) {
        Ok(val) => val,
        Err(e) => {
            trace!("Failed to parse name-value for system_message_goal: {}", e);
            return Ok(None);
        }
    };

    debug!("Parsed system_message_goal via name-value syntax: {:?}", parsed.msg.value());
    Ok(Some(parsed.msg))
}
