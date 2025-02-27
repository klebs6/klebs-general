// ---------------- [ File: src/maybe_reuse_lifetime.rs ]
crate::ix!();

/// Checks if a `GenericArgument::Lifetime(lt)` is already in `wire_gens`.
/// If yes, returns `Some(ident)`, else `None`.
pub fn maybe_reuse_lifetime(
    lt_ident: &Ident,
    wire_gens: &Generics,
) -> Option<Ident> {
    let found = wire_gens.params.iter().any(|gp| match gp {
        GenericParam::Lifetime(lp) => lp.lifetime.ident == *lt_ident,
        _ => false,
    });
    if found {
        Some(lt_ident.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod test_maybe_reuse_lifetime {
    use super::*;
    #[test]
    fn test_maybe_reuse_lifetime_notfound() {
        let wire_ast: DeriveInput = parse_quote! { struct Wire<T> {} };
        let wire_gens = wire_ast.generics;
        let lt_ident = Ident::new("a", proc_macro2::Span::call_site());

        let found = maybe_reuse_lifetime(&lt_ident, &wire_gens);
        assert!(found.is_none(), "Should not find lifetime 'a'");
    }

    #[test]
    fn test_maybe_reuse_lifetime_found() {
        let wire_ast: DeriveInput = parse_quote! { struct Wire<'a> {} };
        let wire_gens = wire_ast.generics;
        let lt_ident = Ident::new("a", proc_macro2::Span::call_site());

        let found = maybe_reuse_lifetime(&lt_ident, &wire_gens);
        assert!(found.is_some(), "Should find lifetime 'a'");
        assert_eq!(found.unwrap().to_string(), "a");
    }
}
