// ---------------- [ File: workspacer-file-filter/src/apply.rs ]
crate::ix!();

pub async fn apply_text_filter_to_files(
    model_type:        &LanguageModelType,
    list_path:         impl AsRef<Path>,
    user_instructions: &str,
    plant:             bool,
    config:            &FileFilterConfig
) -> Result<(), AiFileFilterError>
{
    // 1) Read the list of paths from the `list_path` file
    let p = list_path.as_ref();
    let content = tokio::fs::read_to_string(p).await.map_err(|io_err| {
        AiFileFilterError::IoError {
            io_error: std::sync::Arc::new(io_err),
            context: format!("Failed to read file list at {}", p.display()),
        }
    })?;

    // 2) Collect the lines (paths) into a vector, skipping empty lines
    let mut raw_paths = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            warn!("Skipping empty line #{} in file list", idx);
            continue;
        }
        raw_paths.push(PathBuf::from(trimmed));
    }

    // 3) If no valid lines, bail early
    if raw_paths.is_empty() {
        info!("No valid files were provided, so nothing to do.");
        return Ok(());
    }

    // ----------------------------------------------------------------
    // 4) Enforce uniqueness of final basenames (file_stem).
    //
    //    For example, if we have "src/foo.rs" and "tests/foo.rs",
    //    both have final basename "foo", so we error out.
    //
    //    This is done before building the AiFileFilterRequest so that
    //    we fail fast if there's a collision.
    // ----------------------------------------------------------------
    use std::collections::HashMap;
    let mut stem_to_path_map: HashMap<String, PathBuf> = HashMap::new();

    for path in &raw_paths {
        // Extract the final file name (no directories). E.g. from "src/foo.rs" => "foo.rs"
        let file_name = match path.file_name() {
            Some(fname) => fname.to_string_lossy().to_string(),
            None => {
                let msg = format!("Path '{}' has no valid file name", path.display());
                return Err(AiFileFilterError::IoError {
                    io_error: std::sync::Arc::new(std::io::Error::new(std::io::ErrorKind::Other, &*msg)),
                    context:  msg,
                });
            }
        };

        // Now extract the "stem" portion, ignoring extension. E.g. "foo.rs" => "foo"
        let raw_stem = match path.file_stem() {
            Some(stem) if !stem.is_empty() => stem.to_string_lossy().to_string(),
            _ => {
                let msg = format!(
                    "Path '{}' has no valid file stem (maybe empty or starts with '.')",
                    path.display()
                );
                return Err(AiFileFilterError::IoError {
                    io_error: std::sync::Arc::new(std::io::Error::new(std::io::ErrorKind::Other, &*msg)),
                    context:  msg,
                });
            }
        };

        // Check if we already have that stem in the map
        if let Some(existing_path) = stem_to_path_map.get(&raw_stem) {
            // If so, throw an error because we want uniqueness
            let msg = format!(
                "Collision: the file '{}' and '{}' both have the same stem '{}'.",
                existing_path.display(),
                path.display(),
                raw_stem
            );
            error!("{}", msg);
            return Err(AiFileFilterError::GenericError);
        } else {
            stem_to_path_map.insert(raw_stem, path.clone());
        }
    }

    // ----------------------------------------------------------------
    // 5) Now that we've validated uniqueness, we can build requests.
    // ----------------------------------------------------------------
    let mut requests = Vec::new();
    for path_obj in raw_paths {
        let req = AiFileFilterRequest::async_try_from_path(path_obj, user_instructions, config).await?;
        requests.push(req);
    }

    // 6) Acquire the AiFileFilter object and process
    let mut writer = AiFileFilter::with_model(model_type).await?;
    execute_ai_file_filter_requests(&mut writer, &requests, plant).await?;

    info!("apply_text_filter_to_files completed successfully.");
    Ok(())
}
