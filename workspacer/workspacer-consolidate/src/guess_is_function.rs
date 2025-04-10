crate::ix!();

/// A small helper to decide if a `CrateInterfaceItem<T>` should be rendered as a function.
/// 1) If `item.syntax_kind()` returns `Some(SyntaxKind::FN)`, we’re definitely a function.
/// 2) Otherwise, if item.syntax_kind() is `None` (e.g. a mock in tests), we look at the generated signature:
///    - If the signature *starts* with something like `fn ` or `pub fn ` or `async fn `, treat it as function.
/// This way, tests that pass in a mock item with a “fn signature” can still see the body displayed.
#[tracing::instrument(level = "trace", skip(item, signature))]
pub fn guess_is_function<T: MaybeHasSyntaxKind>(item: &T, signature: &str) -> bool {
    // 1) If real AST kind says FN => definitely a function
    if let Some(kind) = item.syntax_kind() {
        return kind == SyntaxKind::FN;
    }

    // 2) If `None`, we fallback by checking the signature text for a “fn ” pattern
    let trimmed = signature.trim_start();
    if trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("const fn ")
        || trimmed.starts_with("pub(crate) fn ")
        || trimmed.starts_with("pub(in ")
    {
        trace!("guess_is_function: Found a fallback match for fn signature => returning true.");
        return true;
    }

    // Otherwise, not a function
    false
}
