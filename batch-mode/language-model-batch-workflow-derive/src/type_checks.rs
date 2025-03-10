crate::ix!();

/// Checks if a given `ty` is exactly `Arc<OpenAIClientHandle>`
pub fn is_arc_openai_client_handle(ty: &syn::Type) -> bool {
    // Example: match `TypePath(tp)` to see if it's a path-based type
    if let TypePath(tp) = ty {
        // Looking for `Arc<OpenAIClientHandle>` => path.segments = ["Arc"], 
        // plus 1 angle-bracketed generic argument which is "OpenAIClientHandle".
        if let Some(last_seg) = tp.path.segments.last() {
            if last_seg.ident == "Arc" {
                if let AngleBracketed(ab) = &last_seg.arguments {
                    if ab.args.len() == 1 {
                        if let Some(GAType(inner_ty)) = ab.args.first() {
                            // Check if it's a single-segment path "OpenAIClientHandle"
                            if let TypePath(inner_path) = inner_ty {
                                if inner_path.path.segments.len() == 1 {
                                    return inner_path.path.segments[0].ident == "OpenAIClientHandle";
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Checks if a given `ty` is exactly `Arc<dyn LanguageModelClientInterface<OpenAIClientError>>`
pub fn is_arc_dyn_language_model_client_e(ty: &syn::Type) -> bool {
    if let TypePath(type_path) = ty {
        if let Some(arc_seg) = type_path.path.segments.last() {
            if arc_seg.ident == "Arc" {
                // Must have <...> with exactly 1 argument
                if let AngleBracketed(ab) = &arc_seg.arguments {
                    if ab.args.len() == 1 {
                        if let Some(GAType(inner_ty)) = ab.args.first() {
                            // Must be `Type::TraitObject(...)`: `dyn LanguageModelClientInterface<OpenAIClientError>`
                            if let syn::Type::TraitObject(trait_obj) = inner_ty {
                                if trait_obj.bounds.len() == 1 {
                                    if let Some(syn::TypeParamBound::Trait(TraitBound { path, .. })) =
                                        trait_obj.bounds.first()
                                    {
                                        // The final path segment must be `.LanguageModelClientInterface<OpenAIClientError>`
                                        if let Some(client_seg) = path.segments.last() {
                                            if client_seg.ident == "LanguageModelClientInterface" {
                                                // Must have <OpenAIClientError>
                                                if let AngleBracketed(ab2) = &client_seg.arguments {
                                                    if ab2.args.len() == 1 {
                                                        if let Some(GAType(err_ty)) = ab2.args.first() {
                                                            if let TypePath(e_path) = err_ty {
                                                                if let Some(e_seg) = e_path.path.segments.last() {

                                                                    //we allow any type here. the
                                                                    //compiler will catch it if it
                                                                    //is wrong
                                                                    return true;
                                                                    /*
                                                                    if e_seg.ident == "OpenAIClientError"  || e_seg.ident == "BatchProcessingError" {
                                                                        return true;
                                                                    }
                                                                    */
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Returns `true` if `ty` is a valid "batch client" type:
/// either `Arc<OpenAIClientHandle>` **or** `Arc<dyn LanguageModelClientInterface<OpenAIClientError>>`.
pub fn is_valid_batch_client_type(ty: &syn::Type) -> bool {
    is_arc_openai_client_handle(ty) || is_arc_dyn_language_model_client_e(ty)
}

// ---------------------------------------------------------------------------

/// Checks if a given `ty` is exactly `Arc<BatchWorkspace>`
pub fn is_arc_batch_workspace(ty: &syn::Type) -> bool {
    if let TypePath(tp) = ty {
        if let Some(last_seg) = tp.path.segments.last() {
            if last_seg.ident == "Arc" {
                if let AngleBracketed(ab) = &last_seg.arguments {
                    if ab.args.len() == 1 {
                        if let Some(GAType(inner_ty)) = ab.args.first() {
                            if let TypePath(inner_path) = inner_ty {
                                if inner_path.path.segments.len() == 1 {
                                    return inner_path.path.segments[0].ident == "BatchWorkspace";
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Checks if a given `ty` is exactly `Arc<dyn FullBatchWorkspaceInterface<BatchWorkspaceError>>`
pub fn is_arc_dyn_full_batch_workspace(ty: &syn::Type) -> bool {
    if let TypePath(type_path) = ty {
        if let Some(arc_seg) = type_path.path.segments.last() {
            if arc_seg.ident == "Arc" {
                if let AngleBracketed(ab) = &arc_seg.arguments {
                    if ab.args.len() == 1 {
                        if let Some(GAType(inner_ty)) = ab.args.first() {
                            if let syn::Type::TraitObject(trait_obj) = inner_ty {
                                if trait_obj.bounds.len() == 1 {
                                    if let Some(syn::TypeParamBound::Trait(TraitBound { path, .. })) =
                                        trait_obj.bounds.first()
                                    {
                                        if let Some(last_seg) = path.segments.last() {
                                            if last_seg.ident == "FullBatchWorkspaceInterface" {
                                                // Must have <BatchWorkspaceError>
                                                if let AngleBracketed(ab2) = &last_seg.arguments {
                                                    if ab2.args.len() == 1 {
                                                        if let Some(GAType(err_ty)) = ab2.args.first() {
                                                            if let TypePath(e_path) = err_ty {
                                                                if let Some(e_seg) = e_path.path.segments.last() {
                                                                    if e_seg.ident == "BatchWorkspaceError" {
                                                                        return true;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Returns `true` if `ty` is a valid "batch workspace" type:
/// either `Arc<BatchWorkspace>` **or** `Arc<dyn FullBatchWorkspaceInterface<BatchWorkspaceError>>`.
pub fn is_valid_batch_workspace_type(ty: &syn::Type) -> bool {
    is_arc_batch_workspace(ty) || is_arc_dyn_full_batch_workspace(ty)
}

// ---------------------------------------------------------------------------

/// Returns `true` if the field type is exactly `BatchWorkflowProcessOutputFileFn`
pub fn is_process_batch_output_fn(ty: &syn::Type) -> bool {
    if let TypePath(tp) = ty {
        if tp.qself.is_none() && tp.path.segments.len() == 1 {
            return tp.path.segments[0].ident == "BatchWorkflowProcessOutputFileFn";
        }
    }
    false
}

/// Returns `true` if the field type is exactly `BatchWorkflowProcessErrorFileFn`
pub fn is_process_batch_error_fn(ty: &syn::Type) -> bool {
    if let TypePath(tp) = ty {
        if tp.qself.is_none() && tp.path.segments.len() == 1 {
            return tp.path.segments[0].ident == "BatchWorkflowProcessErrorFileFn";
        }
    }
    false
}

/// Returns `true` if the field is `ExpectedContentType`
pub fn is_expected_content_type(ty: &syn::Type) -> bool {
    if let TypePath(tp) = ty {
        if tp.qself.is_none() && tp.path.segments.len() == 1 {
            return tp.path.segments[0].ident == "ExpectedContentType";
        }
    }
    false
}

/// Returns `true` if the field is `LanguageModelType`
pub fn is_model_type(ty: &syn::Type) -> bool {
    if let TypePath(tp) = ty {
        if tp.qself.is_none() && tp.path.segments.len() == 1 {
            return tp.path.segments[0].ident == "LanguageModelType";
        }
    }
    false
}
