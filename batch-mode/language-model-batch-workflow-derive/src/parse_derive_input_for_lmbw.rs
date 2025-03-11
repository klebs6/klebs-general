// ---------------- [ File: src/parse_derive_input_for_lmbw.rs ]
crate::ix!();

pub fn parse_derive_input_for_lmbw(ast: &syn::DeriveInput) -> Result<LmbwParsedInput, syn::Error> {
    trace!("parse_derive_input_for_lmbw: start.");

    let struct_ident  = ast.ident.clone();
    let generics      = ast.generics.clone();

    // For the struct-level attribute #[batch_error_type(MyErr)].
    let mut custom_error_type: Option<syn::Type> = None;
    // For optional #[batch_json_output_format(Foo)] => sets content type to JSON.
    let mut json_output_format_type: Option<syn::Type> = None;

    // We'll parse top-level attributes for batch_error_type and batch_json_output_format.
    for attr in &ast.attrs {
        if let Ok(meta) = attr.parse_meta() {
            let path_ident = meta.path().get_ident().map(|i| i.to_string());
            match (path_ident, meta) {
                (Some(name), syn::Meta::List(list)) if name == "batch_error_type" => {
                    // e.g. #[batch_error_type(MyErr)]
                    match attr.parse_args::<syn::Type>() {
                        Ok(t) => custom_error_type = Some(t),
                        Err(e) => {
                            return Err(syn::Error::new_spanned(
                                &list,
                                format!("Cannot parse #[batch_error_type(...)] attribute: {}", e),
                            ));
                        }
                    }
                },
                (Some(name), syn::Meta::List(list)) if name == "batch_json_output_format" => {
                    // e.g. #[batch_json_output_format(MyOutputType)]
                    match attr.parse_args::<syn::Type>() {
                        Ok(t) => json_output_format_type = Some(t),
                        Err(e) => {
                            return Err(syn::Error::new_spanned(
                                &list,
                                format!("Cannot parse #[batch_json_output_format(...)] attribute: {}", e),
                            ));
                        }
                    }
                },
                _ => {}
            }
        }
    }

    // Must be a named struct.
    let fields = match &ast.data {
        syn::Data::Struct(ds) => match &ds.fields {
            syn::Fields::Named(named) => &named.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &ast.ident,
                    "LanguageModelBatchWorkflow derive only supports a named struct.",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &ast.ident,
                "LanguageModelBatchWorkflow derive only supports a struct.",
            ));
        }
    };

    // We'll gather known attributes from fields.
    let mut batch_client_field:    Option<syn::Ident> = None;
    let mut batch_workspace_field: Option<syn::Ident> = None;
    let mut system_message_field:  Option<syn::Ident> = None;
    let mut model_type_field:      Option<syn::Ident> = None;

    let mut process_batch_output_fn_field: Option<syn::Ident> = None;
    let mut process_batch_error_fn_field:  Option<syn::Ident> = None;

    for field in fields {
        let field_ident = match &field.ident {
            Some(id) => id.clone(),
            None => continue,
        };

        for attr in &field.attrs {
            if let Ok(syn::Meta::Path(path)) = attr.parse_meta() {
                if path.is_ident("batch_client") {
                    batch_client_field = Some(field_ident.clone());
                } else if path.is_ident("batch_workspace") {
                    batch_workspace_field = Some(field_ident.clone());
                } else if path.is_ident("custom_process_batch_output_fn") {
                    process_batch_output_fn_field = Some(field_ident.clone());
                } else if path.is_ident("custom_process_batch_error_fn") {
                    process_batch_error_fn_field = Some(field_ident.clone());
                } else if path.is_ident("system_message") {
                    system_message_field = Some(field_ident.clone());
                } else if path.is_ident("model_type") {
                    model_type_field = Some(field_ident.clone());
                }
            }
        }
    }

    // Check for required fields:
    if batch_client_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_client]` field.",
        ));
    }
    if batch_workspace_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_workspace]` field.",
        ));
    }
    if system_message_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[system_message]` field.",
        ));
    }
    if model_type_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[model_type]` field.",
        ));
    }
    if custom_error_type.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_error_type(...)]` attribute on the struct.",
        ));
    }

    // Build the final object.
    let built = LmbwParsedInputBuilder::default()
        .struct_ident(struct_ident)
        .generics(generics)
        .batch_client_field(batch_client_field)
        .batch_workspace_field(batch_workspace_field)
        .custom_error_type(custom_error_type)
        .json_output_format_type(json_output_format_type)
        .system_message_field(system_message_field)
        .model_type_field(model_type_field)
        .process_batch_output_fn_field(process_batch_output_fn_field)
        .process_batch_error_fn_field(process_batch_error_fn_field)
        .build()
        .map_err(|e| syn::Error::new_spanned(&ast.ident, format!("Builder error: {}", e)))?;

    Ok(built)
}

// ===========================[ CHANGED ITEM #1 ]===========================
// The entire pair of test functions in `test_parse_derive_input_for_lmbw`,
// so they correctly handle that `parse_derive_input_for_lmbw` now returns 
// `Result<LmbwParsedInput, syn::Error>` and also that certain fields are 
// strictly required.
//
// 1. `verifies_named_struct_parsing` is updated so that it unwraps the `Ok` result
//    (since that struct does contain all required fields).
//
// 2. `handles_struct_lacking_attributes` is updated to expect an `Err`, 
//    because the missing fields cause a compile error in the macro logic now.
#[cfg(test)]
mod test_parse_derive_input_for_lmbw {
    use super::*;

    #[traced_test]
    fn verifies_named_struct_parsing() {
        info!("Starting verifies_named_struct_parsing test for parse_derive_input_for_lmbw.");

        // This struct has all required fields/attributes, so we expect success.
        // Specifically, we have:
        //   #[batch_client]
        //   #[batch_workspace]
        //   #[model_type]
        // plus optional custom_process_batch_* attributes.
        let ast: DeriveInput = parse_quote! {
            #[batch_error_type(MyCustomError)]
            struct Dummy {
                #[batch_client]
                some_client: std::sync::Arc<OpenAIClientHandle>,

                #[batch_workspace]
                some_workspace: std::sync::Arc<BatchWorkspace>,

                #[custom_process_batch_output_fn]
                pbo: BatchWorkflowProcessOutputFileFn,

                #[custom_process_batch_error_fn]
                pbe: BatchWorkflowProcessErrorFileFn,

                #[model_type]
                mt: LanguageModelType,

                #[system_message]
                sm: String,
            }
        };

        let parsed = match parse_derive_input_for_lmbw(&ast) {
            Ok(x) => x,
            Err(e) => {
                panic!("Expected parse to succeed, but got error: {}", e);
            }
        };

        // Now that we have an `LmbwParsedInput`, we check the relevant fields:
        assert!(
            parsed.batch_client_field().is_some(),
            "Should have found batch_client field."
        );
        assert!(
            parsed.batch_workspace_field().is_some(),
            "Should have found batch_workspace field."
        );
        assert!(
            parsed.process_batch_output_fn_field().is_some(),
            "Should have found custom_process_batch_output_fn field."
        );
        assert!(
            parsed.process_batch_error_fn_field().is_some(),
            "Should have found custom_process_batch_error_fn field."
        );
        assert!(
            parsed.model_type_field().is_some(),
            "Should have found model_type field."
        );
    }

    #[traced_test]
    fn handles_struct_lacking_attributes() {
        info!("Starting handles_struct_lacking_attributes test for parse_derive_input_for_lmbw.");

        // This struct is missing the required fields/attributes, so we expect 
        // the parse to fail with an error, not succeed. The older test code 
        // tried to check if they were “none,” but that’s not valid anymore 
        // because we enforce a compile error if they’re missing.
        let ast: DeriveInput = parse_quote! {
            // no batch_error_type, batch_client, batch_workspace, etc.
            struct NoAttrs {
                field_a: i32,
                field_b: String,
            }
        };

        let result = parse_derive_input_for_lmbw(&ast);
        assert!(
            result.is_err(),
            "Should have error because we lack required fields (batch_client, batch_workspace, etc.)"
        );
        // If desired, you can inspect the error message:
        if let Err(e) = result {
            info!("Got expected error: {}", e);
            // You could do further checks on e if you want, e.g. `assert!(format!("{}", e).contains("Missing required"))`
        }
    }
}
