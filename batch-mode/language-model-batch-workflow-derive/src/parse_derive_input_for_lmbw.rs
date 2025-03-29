// ---------------- [ File: src/parse_derive_input_for_lmbw.rs ]
crate::ix!();

pub fn parse_derive_input_for_lmbw(ast: &DeriveInput) -> Result<LmbwParsedInput, Error> {
    tracing::trace!("parse_derive_input_for_lmbw: start.");

    let struct_ident  = ast.ident.clone();
    let generics      = ast.generics.clone();

    // For the struct-level attribute `#[batch_error_type(...)]`.
    let mut custom_error_type: Option<Type> = None;
    // For optional `#[batch_json_output_format(...)]`
    let mut json_output_format_type: Option<Type> = None;

    // ----------------------------------------------
    // 1) Scan top-level attributes for:
    //    - `#[batch_error_type(...)]`
    //    - `#[batch_json_output_format(...)]`
    // ----------------------------------------------
    for attr in &ast.attrs {
        if attr.path().is_ident("batch_error_type") {
            // e.g. `#[batch_error_type(MyErr)]`
            let parsed_ty = attr.parse_args::<Type>()?;
            custom_error_type = Some(parsed_ty);

        } else if attr.path().is_ident("batch_json_output_format") {
            // e.g. `#[batch_json_output_format(MyConcreteOutput)]`
            //
            // If the user already specified it, that would be a conflict, 
            // but typically you'd just overwrite. Or you can error out 
            // if you want exactly one. We'll assume only one is allowed.
            if json_output_format_type.is_some() {
                return Err(Error::new_spanned(
                    attr, 
                    "Duplicate #[batch_json_output_format(...)] attribute found.",
                ));
            }

            let parsed_ty = attr.parse_args::<Type>()?;
            json_output_format_type = Some(parsed_ty);
        }
    }

    // ----------------------------------------------
    // 2) Ensure this is a named struct, not an enum or tuple struct
    // ----------------------------------------------
    let fields = match &ast.data {
        syn::Data::Struct(ds) => match &ds.fields {
            syn::Fields::Named(named) => &named.named,
            _ => {
                return Err(Error::new_spanned(
                    &ast.ident,
                    "LanguageModelBatchWorkflow derive only supports a named struct.",
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &ast.ident,
                "LanguageModelBatchWorkflow derive can only be used on a struct.",
            ));
        }
    };

    // ----------------------------------------------
    // 3) Look for the required field attributes
    //    `[batch_client]`, `[batch_workspace]`, `[model_type]`, etc.
    // ----------------------------------------------
    let mut batch_client_field:    Option<syn::Ident> = None;
    let mut batch_workspace_field: Option<syn::Ident> = None;
    let mut model_type_field:      Option<syn::Ident> = None;

    let mut process_batch_output_fn_field: Option<syn::Ident> = None;
    let mut process_batch_error_fn_field:  Option<syn::Ident> = None;

    for field in fields {
        let field_ident = match &field.ident {
            Some(id) => id.clone(),
            None => continue,
        };

        for attr in &field.attrs {
            if attr.path().is_ident("batch_client") {
                batch_client_field = Some(field_ident.clone());
            } else if attr.path().is_ident("batch_workspace") {
                batch_workspace_field = Some(field_ident.clone());
            } else if attr.path().is_ident("model_type") {
                model_type_field = Some(field_ident.clone());
            } else if attr.path().is_ident("custom_process_batch_output_fn") {
                process_batch_output_fn_field = Some(field_ident.clone());
            } else if attr.path().is_ident("custom_process_batch_error_fn") {
                process_batch_error_fn_field = Some(field_ident.clone());
            }
        }
    }

    // ----------------------------------------------
    // 4) Verify required fields exist
    // ----------------------------------------------
    if batch_client_field.is_none() {
        return Err(Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_client]` field.",
        ));
    }
    if batch_workspace_field.is_none() {
        return Err(Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_workspace]` field.",
        ));
    }
    if model_type_field.is_none() {
        return Err(Error::new_spanned(
            &ast.ident,
            "Missing required `#[model_type]` field.",
        ));
    }
    if custom_error_type.is_none() {
        return Err(Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_error_type(...)]` attribute on the struct.",
        ));
    }

    // ----------------------------------------------
    // 5) Build LmbwParsedInput
    // ----------------------------------------------
    let built = LmbwParsedInputBuilder::default()
        .struct_ident(struct_ident)
        .generics(generics)
        .batch_client_field(batch_client_field)
        .batch_workspace_field(batch_workspace_field)
        .custom_error_type(custom_error_type)
        .json_output_format_type(json_output_format_type)
        .model_type_field(model_type_field)
        .process_batch_output_fn_field(process_batch_output_fn_field)
        .process_batch_error_fn_field(process_batch_error_fn_field)
        .build()
        .map_err(|e| {
            Error::new_spanned(&ast.ident, format!("Builder error: {e}"))
        })?;

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
