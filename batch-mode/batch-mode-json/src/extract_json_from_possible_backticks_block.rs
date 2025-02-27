// ---------------- [ File: src/extract_json_from_possible_backticks_block.rs ]
crate::ix!();

/// Extracts JSON from a content string by removing surrounding ```json\n and \n``` markers if present,
/// and trims any leading or trailing whitespace from the input.
///
/// # Arguments
/// * `content` - The content string to process.
///
/// # Returns
/// * `&str` - The processed content string with markers and extra whitespace removed,
///   or the original trimmed content if markers are absent.
pub fn extract_json_from_possible_backticks_block(content: &str) -> &str {
    let trimmed_content = content.trim();
    if trimmed_content.starts_with("```json") {
        let json_start = 7; // Skip past "```json"
        let remaining_content = &trimmed_content[json_start..];

        // Try to find the end marker
        if let Some(end_marker_pos) = remaining_content.rfind("```") {
            let json_content = &remaining_content[..end_marker_pos].trim();
            if json_content.is_empty() {
                return "";
            }
            return json_content
                .trim_start_matches('\n')
                .trim_end_matches('\n')
                .trim();
        }

        // No closing backticks, treat everything after "```json" as content
        return remaining_content
            .trim_start_matches('\n')
            .trim_end_matches('\n')
            .trim();
    }

    // Not a JSON block, return the entire trimmed content
    trimmed_content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_with_markers() {
        let content = "```json\n{\"key\": \"value\"}\n```";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[test]
    fn test_extract_json_with_whitespace_and_markers() {
        let content = "   \n```json\n{\"key\": \"value\"}\n```\n   ";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[test]
    fn test_extract_json_no_markers() {
        let content = "   {\"key\": \"value\"}   ";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[test]
    fn test_extract_json_incomplete_markers() {
        let content = "```json{\"key\": \"value\"}";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}"); // Should gracefully handle missing end marker
    }

    #[test]
    fn test_extract_json_trailing_newline() {
        let content = "```json\n{\"key\": \"value\"}\n\n```";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[test]
    fn test_extract_json_empty_string() {
        let content = "   ";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, ""); // Empty input results in empty output
    }

    #[test]
    fn test_extract_json_no_json_after_marker() {
        let content = "```json```";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, ""); // Correctly returns empty for no content
    }
}
