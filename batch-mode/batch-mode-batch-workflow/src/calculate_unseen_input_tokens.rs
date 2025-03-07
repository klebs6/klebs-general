crate::ix!();

pub fn calculate_unseen_input_tokens(
    workspace:    &BatchWorkspace,
    input_tokens: &[CamelCaseTokenWithComment],
) -> Vec<CamelCaseTokenWithComment> {
    let mut unseen = Vec::new();

    for tok in input_tokens {

        let target_path = tok.target_path_for_ai_json_expansion(&workspace.target_dir);

        if !target_path.exists() {
            if let Some(similar_path) = find_similar_target_path(&workspace,&target_path) {
                warn!(
                    "Skipping token '{}': target path '{}' is similar to existing '{}'.",
                    tok.data(),
                    target_path.display(),
                    similar_path.display()
                );
                continue; // Skip this token
            }
            unseen.push(tok.clone());
        }
    }

    unseen
}

pub fn find_similar_target_path(workspace: &BatchWorkspace, target_path: &Path) -> Option<PathBuf> {

    use strsim::levenshtein;

    let existing_paths = workspace.get_target_directory_files();
    let target_str     = target_path.to_string_lossy();

    existing_paths
        .iter()
        .find(|&existing| levenshtein(&target_str, &existing.to_string_lossy()) <= 2)
        .cloned()
}
