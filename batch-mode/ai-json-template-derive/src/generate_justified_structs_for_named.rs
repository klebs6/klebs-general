// ---------------- [ File: ai-json-template-derive/src/generate_justified_structs_for_named.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_justified_structs_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    span: proc_macro2::Span
) -> proc_macro2::TokenStream
{
    trace!(
        "Entering generate_justified_structs_for_named for '{}'",
        ty_ident
    );

    // Example: For `struct Coordinate { x: f32, y: f32 }`, we’ll create a flattened
    // `JustifiedCoordinate` with fields:
    //   x: f32,
    //   x_confidence: f64,
    //   x_justification: String,
    //   y: f32,
    //   y_confidence: f64,
    //   y_justification: String,
    //
    // We'll skip separate `CoordinateJustification` / `CoordinateConfidence`.

    let justified_ident = syn::Ident::new(&format!("Justified{}", ty_ident), span);

    // Collect flattened fields for the new `JustifiedFoo`.
    // For each original field: 
    //   - keep the same name + type 
    //   - add `_confidence: f64` 
    //   - add `_justification: String`
    let mut flattened_fields = Vec::new();

    for field in &named_fields.named {

        let field_ident = match &field.ident {
            Some(id) => id,
            None => {
                warn!("Skipping unnamed field in named struct?");
                continue;
            }
        };

        let justified_ty = justified_type(&field.ty);

        // 1) Gather all attributes from the original field
        let original_attrs = &field.attrs;

        // 2) Filter only the ones that start with `#[serde(...)]`
        let serde_attrs: Vec<_> = original_attrs
            .iter()
            .filter(|attr| attr.path().is_ident("serde"))
            .collect();


        // The original field itself
        flattened_fields.push(quote::quote! {
            #( #serde_attrs )*
            #field_ident: #justified_ty,
        });

        // The confidence + justification
        let conf_ident = syn::Ident::new(
            &format!("{}_confidence", field_ident),
            field_ident.span()
        );
        let just_ident = syn::Ident::new(
            &format!("{}_justification", field_ident),
            field_ident.span()
        );

        // We use `f64` for confidence, `String` for justification
        flattened_fields.push(quote::quote! {
            #conf_ident: f64,
            #just_ident: String,
        });
    }

    // Now build the struct with those flattened fields
    // We keep it `pub struct ...` for final usage in user code, 
    // but remember you said “never use pub members.” 
    // If you want them private + getset, adapt as needed:
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
