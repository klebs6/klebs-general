// ---------------- [ File: ai-json-template-derive/src/generate_justified_structs_for_named.rs ]
crate::ix!();

/// Generates for a named struct `Foo`:
///   - `FooJustification` (one field per original field, type=String or nested),
///   - `FooConfidence`,
///   - `JustifiedFoo`,
///   - plus the item/justification/confidence accessor expansions.
pub fn generate_justified_structs_for_named(
    ty_ident:     &syn::Ident,
    named_fields: &syn::FieldsNamed,
    span:         proc_macro2::Span
) -> (
    proc_macro2::TokenStream, // justification struct
    proc_macro2::TokenStream, // confidence struct
    proc_macro2::TokenStream, // justified struct
    proc_macro2::TokenStream, // accessor expansions
) {
    let justification_ident             = syn::Ident::new(&format!("{}Justification", ty_ident), span);
    let confidence_ident                = syn::Ident::new(&format!("{}Confidence", ty_ident), span);
    let justified_ident                 = syn::Ident::new(&format!("Justified{}", ty_ident), span);

    let mut justification_struct_fields = Vec::new();
    let mut confidence_struct_fields    = Vec::new();
    let mut errs                        = quote::quote!();
    let mut field_mappings              = Vec::new();

    gather_justification_and_confidence_fields(
        named_fields,
        &mut justification_struct_fields,
        &mut confidence_struct_fields,
        &mut errs,
        &mut field_mappings,
    );

    let just_ts = quote::quote! {
        #errs
        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        pub struct #justification_ident {
            #(#justification_struct_fields)*
        }
    };
    let conf_ts = quote::quote! {
        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        pub struct #confidence_ident {
            #(#confidence_struct_fields)*
        }
    };
    let justified_ts = quote::quote! {
        #[derive(Builder, Debug, Default, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        pub struct #justified_ident {
            item:          #ty_ident,
            justification: #justification_ident,
            confidence:    #confidence_ident,
        }

        impl #justified_ident {
            pub fn new(item: #ty_ident) -> Self {
                Self {
                    item,
                    justification: Default::default(),
                    confidence: Default::default(),
                }
            }
        }
    };

    // Now gather the three sets of item/just/conf accessor expansions
    let (item_acc, just_acc, conf_acc) =
        gather_item_accessors(named_fields, ty_ident, &field_mappings);

    let accessor_ts = quote::quote! {
        impl #justified_ident {
            #(#item_acc)*
            #(#just_acc)*
            #(#conf_acc)*
        }
    };

    (just_ts, conf_ts, justified_ts, accessor_ts)
}
