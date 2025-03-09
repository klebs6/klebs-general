// ---------------- [ File: src/parse_derive_input_for_lmbw.rs ]
crate::ix!();

// The entire `parse_derive_input_for_lmbw` function in `parse_derive_input_for_lmbw.rs`.
// We now return `Result<LmbwParsedInput, syn::Error>` instead of a bare `LmbwParsedInput`.
// After we collect fields, we do explicit checks for your required fields:
//
//     - #[batch_client] => `batch_client_field.is_none()` -> error
//     - #[batch_workspace] => same
//     - #[expected_content_type] => same
//     - #[model_type] => same
//     - #[batch_error_type(...)] => if you truly require a custom error in *all* cases,
//       then we also require that. Otherwise, if you truly can default to `TokenExpanderError`,
//       you’d keep it optional. 
//
// We'll assume your “TODO” means you want an actual error if not provided.
pub fn parse_derive_input_for_lmbw(ast: &DeriveInput) -> Result<LmbwParsedInput, syn::Error> {
    trace!("parse_derive_input_for_lmbw: start.");

    let struct_ident = ast.ident.clone();
    let generics     = ast.generics.clone();
    let where_clause = ast.generics.where_clause.clone();

    // Start with placeholders for all fields (some optional, some required).
    // We'll fill them in as we parse attributes.
    let mut batch_client_field:            Option<Ident> = None;
    let mut batch_workspace_field:         Option<Ident> = None;
    let mut expected_content_type_field:   Option<Ident> = None;
    let mut model_type_field:              Option<Ident> = None;
    let mut custom_error_type:             Option<Type>  = None;

    // The truly optional ones:
    let mut process_batch_output_fn_field: Option<Ident> = None;
    let mut process_batch_error_fn_field:  Option<Ident> = None;

    // Check top-level struct attributes for `#[batch_error_type(...)]`.
    // If you truly want to *require* a custom error type always, we’ll ensure 
    // that if it's missing, we fail. If you want to fallback to `TokenExpanderError`,
    // then remove the check below. For now, we assume the "TODO" says it's required.
    for attr in &ast.attrs {
        if let Ok(meta) = attr.parse_meta() {
            match meta {
                Meta::Path(path) => {
                    // e.g. `#[batch_error_type]` with no args => we’ll produce an error below.
                    if path.is_ident("batch_error_type") {
                        warn!("`#[batch_error_type]` with no parentheses is not valid. Will fail later.");
                    }
                }
                Meta::List(meta_list) => {
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
                    // We don’t expect something like `#[batch_error_type = "foo"]`
                }
            }
        }
    }

    // If it’s a named struct, walk its fields:
    if let syn::Data::Struct(ref data_struct) = ast.data {
        if let Fields::Named(ref named_fields) = data_struct.fields {
            for field in &named_fields.named {
                let field_ident = match &field.ident {
                    Some(id) => id,
                    None => continue,
                };
                for attr in &field.attrs {
                    if let Ok(Meta::Path(path)) = attr.parse_meta() {
                        if path.is_ident("batch_client") {
                            batch_client_field = Some(field_ident.clone());
                        } else if path.is_ident("batch_workspace") {
                            batch_workspace_field = Some(field_ident.clone());
                        } else if path.is_ident("custom_process_batch_output_fn") {
                            process_batch_output_fn_field = Some(field_ident.clone());
                        } else if path.is_ident("custom_process_batch_error_fn") {
                            process_batch_error_fn_field = Some(field_ident.clone());
                        } else if path.is_ident("expected_content_type") {
                            expected_content_type_field = Some(field_ident.clone());
                        } else if path.is_ident("model_type") {
                            model_type_field = Some(field_ident.clone());
                        }
                    } else if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                        let path = &meta_list.path;
                        if path.is_ident("batch_client") {
                            batch_client_field = Some(field_ident.clone());
                        } else if path.is_ident("batch_workspace") {
                            batch_workspace_field = Some(field_ident.clone());
                        } else if path.is_ident("custom_process_batch_output_fn") {
                            process_batch_output_fn_field = Some(field_ident.clone());
                        } else if path.is_ident("custom_process_batch_error_fn") {
                            process_batch_error_fn_field = Some(field_ident.clone());
                        } else if path.is_ident("expected_content_type") {
                            expected_content_type_field = Some(field_ident.clone());
                        } else if path.is_ident("model_type") {
                            model_type_field = Some(field_ident.clone());
                        }
                    }
                }
            }
        }
    } else {
        warn!("LanguageModelBatchWorkflow derive: Not a named struct. Will generate no code.");
        // We can either return an empty struct or a compile error. We do:
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "LanguageModelBatchWorkflow derive only supports named structs.",
        ));
    }

    trace!("parse_derive_input_for_lmbw: done collecting relevant fields.");

    // ========= Enforce required fields with nice compiler errors ==========

    if batch_client_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_client]`. You must annotate exactly one field with `#[batch_client]`."
        ));
    }

    if batch_workspace_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_workspace]`. You must annotate exactly one field with `#[batch_workspace]`."
        ));
    }

    if expected_content_type_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[expected_content_type]`. You must annotate exactly one field with `#[expected_content_type]`."
        ));
    }

    if model_type_field.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[model_type]`. You must annotate exactly one field with `#[model_type]`."
        ));
    }

    if custom_error_type.is_none() {
        return Err(syn::Error::new_spanned(
            &ast.ident,
            "Missing required `#[batch_error_type(...)]` attribute on the struct. Provide a custom error type."
        ));
    }

    // If all required fields are present, we can build:
    let built = LmbwParsedInputBuilder::default()
        .struct_ident(struct_ident)
        .generics(generics)
        .where_clause(where_clause)
        .batch_client_field(batch_client_field)
        .batch_workspace_field(batch_workspace_field)
        .expected_content_type_field(expected_content_type_field)
        .model_type_field(model_type_field)
        .custom_error_type(custom_error_type)
        // optional fields
        .process_batch_output_fn_field(process_batch_output_fn_field)
        .process_batch_error_fn_field(process_batch_error_fn_field)
        .build()
        .map_err(|e| {
            syn::Error::new_spanned(&ast.ident, format!("Builder error: {}", e))
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
        //   #[expected_content_type]
        //   #[model_type]
        // plus optional custom_process_batch_* attributes.
        let ast: DeriveInput = parse_quote! {
            #[batch_error_type(MyCustomError)]
            struct Dummy {
                #[batch_client]
                some_client: std::sync::Arc<()>,
                #[batch_workspace]
                some_workspace: std::sync::Arc<()>,

                #[custom_process_batch_output_fn]
                pbo: fn(),

                #[custom_process_batch_error_fn]
                pbe: fn(),

                #[expected_content_type]
                ect: (),

                #[model_type]
                mt: (),
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
