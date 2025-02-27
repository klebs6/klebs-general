// ---------------- [ File: src/operator_key_values.rs ]
crate::ix!();

/// We parse content like:
///   execute="foo", opcode="BasicOpCode::Bar", input0="T", ...
pub struct OperatorKeyValues {
    pub pairs: Vec<(Ident, Lit)>,
}

impl Parse for OperatorKeyValues {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut pairs = Vec::new();
        while !input.is_empty() {
            // key
            let key_ident: Ident = input.parse()?;
            // =
            let _eq: Token![=] = input.parse()?;
            // "value"
            let val: Lit = input.parse()?;

            pairs.push((key_ident, val));

            // If there's a trailing comma, consume it, else break
            if input.peek(Token![,]) {
                let _comma: Token![,] = input.parse()?;
            } else {
                break;
            }
        }
        Ok(OperatorKeyValues { pairs })
    }
}
