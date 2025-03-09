// ---------------- [ File: src/parse_derive_input_for_lmbw.rs ]
crate::ix!();

/// Parse the `#[derive(LanguageModelBatchWorkflow)]` input with a two-phase approach:
///  1. Collect which fields carry which attributes (without type checks).
///  2. Check for missing required attributes (batch_client, batch_workspace, etc.).
///  3. If present, then run the type-check for each attribute's field.
///
pub fn parse_derive_input_for_lmbw(ast: &DeriveInput) -> Result<LmbwParsedInput, syn::Error> {
    trace!("parse_derive_input_for_lmbw: start.");

    let struct_ident  = ast.ident.clone();
    let generics      = ast.generics.clone();
    let where_clause  = ast.generics.where_clause.clone();

    // ----------- Placeholders for attributes -------------
    let mut batch_client_field:            Option<Ident> = None;
    let mut batch_workspace_field:         Option<Ident> = None;
    let mut expected_content_type_field:   Option<Ident> = None;
    let mut model_type_field:              Option<Ident> = None;
    let mut custom_error_type:             Option<Type>  = None;

    // Optional fields:
    let mut process_batch_output_fn_field: Option<Ident> = None;
    let mut process_batch_error_fn_field:  Option<Ident> = None;

    // ========== (1) Parse top-level struct attributes for #[batch_error_type(...)] ==========
    for attr in &ast.attrs {
        if let Ok(meta) = attr.parse_meta() {
            match meta {
                Meta::Path(path) => {
                    if path.is_ident("batch_error_type") {
                        // e.g. #[batch_error_type], no parentheses => might fail later
                        warn!(
                            "`#[batch_error_type]` with no parentheses is not valid; ignoring for now."
                        );
                    }
                }
                Meta::List(meta_list) => {
                    // e.g. #[batch_error_type(MyErr)]
                    if meta_list.path.is_ident("batch_error_type") {
                        match attr.parse_args::<Type>() {
                            Ok(t) => {
                                custom_error_type = Some(t);
                            }
                            Err(e) => {
                                return Err(syn::Error::new_spanned(
                                    &meta_list,
                                    format!("Cannot parse #[batch_error_type(...)] attribute: {}", e),
                                ));
                            }
                        }
                    }
                }
                Meta::NameValue(_) => {
                    // e.g. #[batch_error_type = "..."], not expected
                }
            }
        }
    }

    // ========== (2) Must be a named struct. Gather attributes from each field. ==========
    let fields = match &ast.data {
        syn::Data::Struct(ds) => match &ds.fields {
            Fields::Named(named) => &named.named,
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

    // ---------- Phase 1: Collect which field is which attribute (NO type checks here!) ----------
    // We'll do the actual type checks in Phase 2 below, *after* we confirm whether any are missing.
    // This approach ensures we see “Missing required #[batch_client]” if it’s absent, even if
    // some other field is typed incorrectly.
    //
    // So here, we only record “Oh, this field is the batch_client_field” or “this field is the batch_workspace”, etc.

    let mut field_idents_and_attrs: Vec<(Ident, Vec<&str>)> = Vec::new();

    for field in fields {
        let field_ident = match &field.ident {
            Some(id) => id.clone(),
            None => continue,
        };

        let mut attr_names_for_this_field = Vec::new();

        for attr in &field.attrs {
            if let Ok(Meta::Path(path)) = attr.parse_meta() {
                if path.is_ident("batch_client") {
                    attr_names_for_this_field.push("batch_client");
                }
                else if path.is_ident("batch_workspace") {
                    attr_names_for_this_field.push("batch_workspace");
                }
                else if path.is_ident("custom_process_batch_output_fn") {
                    attr_names_for_this_field.push("custom_process_batch_output_fn");
                }
                else if path.is_ident("custom_process_batch_error_fn") {
                    attr_names_for_this_field.push("custom_process_batch_error_fn");
                }
                else if path.is_ident("expected_content_type") {
                    attr_names_for_this_field.push("expected_content_type");
                }
                else if path.is_ident("model_type") {
                    attr_names_for_this_field.push("model_type");
                }
            }
        }

        if !attr_names_for_this_field.is_empty() {
            field_idents_and_attrs.push((field_ident, attr_names_for_this_field));
        }
    }

    // Now we interpret which field got which attribute:
    for (ident, attr_names) in &field_idents_and_attrs {
        for attr_name in attr_names {
            match *attr_name {
                "batch_client" => {
                    batch_client_field = Some(ident.clone());
                }
                "batch_workspace" => {
                    batch_workspace_field = Some(ident.clone());
                }
                "custom_process_batch_output_fn" => {
                    process_batch_output_fn_field = Some(ident.clone());
                }
                "custom_process_batch_error_fn" => {
                    process_batch_error_fn_field = Some(ident.clone());
                }
                "expected_content_type" => {
                    expected_content_type_field = Some(ident.clone());
                }
                "model_type" => {
                    model_type_field = Some(ident.clone());
                }
                _ => {}
            }
        }
    }

    // ========== (3) Check for missing required attributes. ==========

    if batch_client_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_client]`. You must annotate exactly one field with #[batch_client].",
        ));
    }
    if batch_workspace_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_workspace]`.",
        ));
    }
    if expected_content_type_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[expected_content_type]`.",
        ));
    }
    if model_type_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[model_type]`.",
        ));
    }
    if custom_error_type.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_error_type(...)]` attribute on the struct. Provide a custom error type.",
        ));
    }

    // ========== (4) Type-check whichever attributes we DID find. ==========

    // We'll need to look up the actual `syn::Field` in order to do type checks. Let's build a map from
    // field-name -> Type, then for each attribute we found, do the check.
    let mut field_map = std::collections::HashMap::new();
    for field in fields {
        if let Some(ref fid) = field.ident {
            field_map.insert(fid.to_string(), &field.ty);
        }
    }

    // If we found a #[batch_client], let's check that field's type:
    if let Some(ref bc_ident) = batch_client_field {
        let bc_ty = field_map.get(&bc_ident.to_string()).unwrap(); // guaranteed to exist
        if !is_valid_batch_client_type(bc_ty) {
            return Err(syn::Error::new_spanned(
                bc_ty,
                "The #[batch_client] field must be either Arc<OpenAIClientHandle> \
                 or Arc<dyn LanguageModelClientInterface<OpenAIClientError>>",
            ));
        }
    }

    // If we found a #[batch_workspace], check that type:
    if let Some(ref bw_ident) = batch_workspace_field {
        let bw_ty = field_map.get(&bw_ident.to_string()).unwrap();
        if !is_valid_batch_workspace_type(bw_ty) {
            return Err(syn::Error::new_spanned(
                bw_ty,
                "The #[batch_workspace] field must be either Arc<BatchWorkspace> \
                 or Arc<dyn FullBatchWorkspaceInterface<BatchWorkspaceError>>",
            ));
        }
    }

    // If we found a #[custom_process_batch_output_fn], type-check it:
    if let Some(ref outfn_ident) = process_batch_output_fn_field {
        let outfn_ty = field_map.get(&outfn_ident.to_string()).unwrap();
        if !is_process_batch_output_fn(outfn_ty) {
            return Err(syn::Error::new_spanned(
                outfn_ty,
                "The #[custom_process_batch_output_fn] field must be BatchWorkflowProcessOutputFileFn",
            ));
        }
    }

    // If we found a #[custom_process_batch_error_fn], type-check it:
    if let Some(ref errfn_ident) = process_batch_error_fn_field {
        let errfn_ty = field_map.get(&errfn_ident.to_string()).unwrap();
        if !is_process_batch_error_fn(errfn_ty) {
            return Err(syn::Error::new_spanned(
                errfn_ty,
                "The #[custom_process_batch_error_fn] field must be BatchWorkflowProcessErrorFileFn",
            ));
        }
    }

    // If we found a #[expected_content_type], check that it's `ExpectedContentType`
    if let Some(ref ect_ident) = expected_content_type_field {
        let ect_ty = field_map.get(&ect_ident.to_string()).unwrap();
        if !is_expected_content_type(ect_ty) {
            return Err(syn::Error::new_spanned(
                ect_ty,
                "The #[expected_content_type] field must be ExpectedContentType",
            ));
        }
    }

    // If we found a #[model_type], check that it's `LanguageModelType`
    if let Some(ref mt_ident) = model_type_field {
        let mt_ty = field_map.get(&mt_ident.to_string()).unwrap();
        if !is_model_type(mt_ty) {
            return Err(syn::Error::new_spanned(
                mt_ty,
                "The #[model_type] field must be LanguageModelType",
            ));
        }
    }

    // ========== (5) All good => build the final object. ==========

    let built = LmbwParsedInputBuilder::default()
        .struct_ident(struct_ident)
        .generics(generics)
        .where_clause(where_clause)
        .batch_client_field(batch_client_field)
        .batch_workspace_field(batch_workspace_field)
        .expected_content_type_field(expected_content_type_field)
        .model_type_field(model_type_field)
        .custom_error_type(custom_error_type)
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
        //   #[expected_content_type]
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

                #[expected_content_type]
                ect: ExpectedContentType,

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
            parsed.expected_content_type_field().is_some(),
            "Should have found expected_content_type field."
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
