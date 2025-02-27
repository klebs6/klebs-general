// ---------------- [ File: hydro2-network-wire-derive/src/mint_lifetime_param.rs ]
crate::ix!();

/// Mint a new lifetime param using exactly the user’s `'x`.
/// Return `(GenericParam, minted_ident)`.
pub fn mint_lifetime_param(user_lifetime: &Ident) -> (GenericParam, Ident) {
    // 1) Build a string `'x`
    let lifetime_str = format!("'{}", user_lifetime);

    // 2) Parse that string as a generic param. For example, `'x` -> LifetimeParam -> GenericParam::Lifetime.
    let param: syn::GenericParam = syn::parse_str(&lifetime_str)
        .expect("should parse as a LifetimeParam");

    // 3) The minted Ident is just `x` (without the quote)
    (param, user_lifetime.clone())
}

#[cfg(test)]
mod test_mint_lifetime_param {
    use super::*;

    #[test]
    fn test_mint_lifetime_param() {
        let lt_ident = Ident::new("x", proc_macro2::Span::call_site());
        let (param, minted_id) = mint_lifetime_param(&lt_ident);
        match &param {
            GenericParam::Lifetime(lp) => {
                assert_eq!(lp.lifetime.ident, lt_ident, "Should keep user ident 'x'");
            }
            _ => panic!("Expected a lifetime param"),
        }
        assert_eq!(minted_id.to_string(), "x");
    }
}
