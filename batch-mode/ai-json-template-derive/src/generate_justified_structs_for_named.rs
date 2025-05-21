// ---------------- [ File: ai-json-template-derive/src/generate_justified_structs_for_named.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_justified_structs_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    span: proc_macro2::Span
) -> proc_macro2::TokenStream {
    trace!(
        "Entering generate_justified_structs_for_named for '{}'",
        ty_ident
    );

    let justified_ident = syn::Ident::new(&format!("Justified{}", ty_ident), span);

    let mut flattened_fields = Vec::new();

    for field in &named_fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => {
                warn!("Skipping unnamed field in a named struct?");
                continue;
            }
        };

        let justified_ty = crate::justified_type(&field.ty);

        // Collect any original #[serde(...)] attributes
        let original_attrs = &field.attrs;
        let serde_attrs: Vec<_> = original_attrs
            .iter()
            .filter(|attr| attr.path().is_ident("serde"))
            .collect();

        // Check if `#[justify(false)]` => skip top-level conf & just
        let skip_field_self = crate::is_justification_disabled_for_field(field);

        // Always store the base field:
        flattened_fields.push(quote::quote! {
            #( #serde_attrs )*
            #field_ident : #justified_ty,
        });

        // If skip_field_self is false => add `xxx_confidence : f64, xxx_justification : String`
        if !skip_field_self {
            let conf_ident = syn::Ident::new(
                &format!("{}_confidence", field_ident),
                field_ident.span()
            );
            let just_ident = syn::Ident::new(
                &format!("{}_justification", field_ident),
                field_ident.span()
            );

            flattened_fields.push(quote::quote! {
                #conf_ident : f64,
                #just_ident : String,
            });
        }
    }

    let flattened_struct = quote::quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
        #[getset(get="pub", set="pub")]
        pub struct #justified_ident {
            #(#flattened_fields)*
        }
    };

    debug!(
        "generate_justified_structs_for_named => built flattened struct '{}'",
        justified_ident
    );
    trace!("Exiting generate_justified_structs_for_named");
    flattened_struct
}
