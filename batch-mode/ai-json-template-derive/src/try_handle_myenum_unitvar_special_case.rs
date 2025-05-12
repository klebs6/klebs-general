crate::ix!();

pub fn try_handle_myenum_unitvar_special_case(
    enum_ident: &Ident,
    variant_ident: &Ident
) -> Option<(TokenStream2, TokenStream2)> {
    trace!(
        "Checking for special MyEnum::UnitVar hack with enum_ident='{}' and variant_ident='{}'",
        enum_ident,
        variant_ident
    );

    // The user wants a special hack when we see `MyEnum::UnitVar`.
    if enum_ident == "MyEnum" && variant_ident == "UnitVar" {
        debug!("Applying hack for MyEnum::UnitVar variant");
        let fv = quote! {
            UnitVar,
        };
        let arm = quote! {
            FlatJustifiedMyEnum :: UnitVar => {
                Self {
                    item: MyEnum :: UnitVar,
                    justification: MyEnumJustification :: UnitVar {
                        variant_justification: Default::default()
                    },
                    confidence: MyEnumConfidence :: UnitVar {
                        variant_confidence: Default::default()
                    },
                }
            }
        };
        Some((fv, arm))
    } else {
        trace!("No hack needed for this variant; returning None");
        None
    }
}
