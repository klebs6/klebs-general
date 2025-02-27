// ---------------- [ File: src/mint_type_param.rs ]
crate::ix!();

/// Mints a new type param using `OpTy{fresh_index}`.
pub fn mint_type_param(span: Span, fresh_index: &mut usize) -> (GenericParam, Ident) {
    let param_ident = Ident::new(&format!("OpTy{}", *fresh_index), span);
    *fresh_index += 1;
    let param: GenericParam = parse_quote! { #param_ident };
    (param, param_ident)
}

#[cfg(test)]
mod test_mint_type_param {
    use super::*;

    #[test]
    fn test_mint_type_param() {
        let mut index = 0;
        let (param, id) = mint_type_param(proc_macro2::Span::call_site(), &mut index);
        match &param {
            GenericParam::Type(tp) => {
                assert_eq!(tp.ident.to_string(), "OpTy0");
            }
            _ => panic!("Expected a type param"),
        }
        assert_eq!(id.to_string(), "OpTy0");
        assert_eq!(index, 1, "Should have incremented index to 1");
    }
}
