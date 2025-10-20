crate::ix!();

/// ---------------------------------------------------------------------------
/// Clip a string to at most `max_chars` characters (not bytes).
/// ---------------------------------------------------------------------------
pub fn clip_to_chars(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        tracing::debug!("Input length {} chars, no clipping needed", input.chars().count());
        input.to_owned()
    } else {
        let clipped: String = input.chars().take(max_chars).collect();
        tracing::warn!(
            "Clipped input text from {} chars to {} chars",
            input.chars().count(),
            max_chars
        );
        clipped
    }
}
