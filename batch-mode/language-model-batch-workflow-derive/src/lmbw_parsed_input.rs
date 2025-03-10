// ---------------- [ File: src/lmbw_parsed_input.rs ]
crate::ix!();

#[derive(Builder, Getters)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct LmbwParsedInput {
    /// The struct’s `Ident`. 
    /// Always supplied by `parse_derive_input_for_lmbw`, so no default is needed.
    struct_ident: Ident,

    /// All the generics declared on the struct.
    /// Also always supplied, so no default is needed.
    generics: Generics,

    // =========== Required Fields (No `#[builder(default)]`) ===========
    // If missing, we give a compile error.
    batch_client_field:          Option<Ident>,
    batch_workspace_field:       Option<Ident>,
    expected_content_type_field: Option<Ident>,
    model_type_field:            Option<Ident>,
    custom_error_type:           Option<Type>,

    // =========== Optional Fields (Keep `#[builder(default)]`) ===========
    #[builder(default)]
    process_batch_output_fn_field: Option<Ident>,

    #[builder(default)]
    process_batch_error_fn_field: Option<Ident>,
}
