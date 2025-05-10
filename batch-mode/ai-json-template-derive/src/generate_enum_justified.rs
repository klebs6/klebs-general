crate::ix!();

/// Creates the minimal:
///   - `[Enum]Justification` with enum_variant_justification: String
///   - `[Enum]Confidence` with enum_variant_confidence: f32
///   - `Justified[Enum]` containing `item: Enum`, `justification: EnumJustification`, `confidence: EnumConfidence`.
///
/// This is used as a basis for all the expansions with enumerations.
pub fn generate_enum_justified(
    ty_ident: &syn::Ident,
    span: proc_macro2::Span
) -> (
    proc_macro2::TokenStream, // enum justification
    proc_macro2::TokenStream, // enum confidence
    proc_macro2::TokenStream  // JustifiedEnum
) {
    let justification_ident = syn::Ident::new(
        &format!("{}Justification", ty_ident),
        span
    );
    let confidence_ident = syn::Ident::new(
        &format!("{}Confidence", ty_ident),
        span
    );
    let justified_ident = syn::Ident::new(
        &format!("Justified{}", ty_ident),
        span
    );

    let enum_just_struct = quote::quote! {
        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #justification_ident {
            #[getset(get="pub", set="pub")]
            enum_variant_justification: String,
        }
    };
    let enum_conf_struct = quote::quote! {
        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #confidence_ident {
            #[getset(get="pub", set="pub")]
            enum_variant_confidence: f32,
        }
    };
    let justified_enum = quote::quote! {
        #[derive(Builder, Debug, Default, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #justified_ident {
            #[getset(get="pub", set="pub")]
            item: #ty_ident,

            #[getset(get="pub", set="pub")]
            justification: #justification_ident,

            #[getset(get="pub", set="pub")]
            confidence: #confidence_ident,
        }

        impl #justified_ident {
            fn new(item: #ty_ident) -> Self {
                Self {
                    item,
                    justification: Default::default(),
                    confidence: Default::default(),
                }
            }
        }
    };

    (enum_just_struct, enum_conf_struct, justified_enum)
}
