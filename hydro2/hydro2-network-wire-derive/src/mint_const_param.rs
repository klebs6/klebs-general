// ---------------- [ File: hydro2-network-wire-derive/src/mint_const_param.rs ]
crate::ix!();

/// Mints a new const param using `OPC{fresh_index}`.
pub fn mint_const_param(span: Span, fresh_index: &mut usize) -> (GenericParam, Ident) {
    let param_ident = Ident::new(&format!("OPC{}", *fresh_index), span);
    *fresh_index += 1;
    let param: GenericParam = parse_quote! {
        const #param_ident : usize
    };
    (param, param_ident)
}

#[cfg(test)]
mod test_mint_const_param {
    use super::*;

    #[test]
    fn test_mint_const_param() {
        let mut index = 2;
        let (param, id) = mint_const_param(proc_macro2::Span::call_site(), &mut index);
        match &param {
            GenericParam::Const(cp) => {
                assert_eq!(cp.ident.to_string(), "OPC2");
            }
            _ => panic!("Expected a const param"),
        }
        assert_eq!(id.to_string(), "OPC2");
        assert_eq!(index, 3);
    }
}
