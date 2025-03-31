// ---------------- [ File: language-model-batch-workflow-derive/src/lmbw_parsed_input.rs ]
crate::ix!();

#[derive(Builder, Getters)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct LmbwParsedInput {
    /// The struct’s `Ident`.
    struct_ident: syn::Ident,

    /// All the generics declared on the struct.
    generics: syn::Generics,

    // Required fields:
    batch_client_field:    Option<syn::Ident>,
    batch_workspace_field: Option<syn::Ident>,
    custom_error_type:     Option<syn::Type>,

    // Optional => we *did* make it optional for JSON. 
    #[builder(default)]
    json_output_format_type: Option<syn::Type>,

    // ---------- NEW: a required model_type field -----------
    model_type_field: Option<syn::Ident>,

    // Optional fields: 
    #[builder(default)]
    process_batch_output_fn_field: Option<syn::Ident>,

    #[builder(default)]
    process_batch_error_fn_field: Option<syn::Ident>,
}
