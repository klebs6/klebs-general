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
    trace!("Entering extract_json_from_possible_backticks_block");
    let trimmed_content = content.trim();
    debug!("Trimmed content: '{}'", trimmed_content);

    if trimmed_content.starts_with("```json") {
        debug!("Detected start marker '```json'");
        let json_start = 7; // Skip past "```json"
        let remaining_content = &trimmed_content[json_start..];
        debug!("Remaining after start marker: '{}'", remaining_content);

        // Try to find the end marker
        if let Some(end_marker_pos) = remaining_content.rfind("```") {
            debug!("Detected end marker at position: {}", end_marker_pos);
            let json_content = &remaining_content[..end_marker_pos].trim();
            debug!("Extracted potential JSON content: '{}'", json_content);

            if json_content.is_empty() {
                debug!("JSON content is empty after trimming");
                return "";
            }

            let final_content = json_content
                .trim_start_matches('\n')
                .trim_end_matches('\n')
                .trim();
            trace!("Final extracted JSON content with markers: '{}'", final_content);
            return final_content;
        }

        // No closing backticks, treat everything after "```json" as content
        warn!("No end marker found. Returning everything after '```json'.");
        let final_content = remaining_content
            .trim_start_matches('\n')
            .trim_end_matches('\n')
            .trim();
        trace!("Final extracted JSON content without closing marker: '{}'", final_content);
        return final_content;
    }

    // Not a JSON block, return the entire trimmed content
    trace!("No JSON block found; returning trimmed content: '{}'", trimmed_content);
    trimmed_content
}

#[cfg(test)]
mod extract_json_from_possible_backticks_block_tests {
    use super::*;

    #[traced_test]
    fn test_extract_json_with_markers() {
        let content = "```json\n{\"key\": \"value\"}\n```";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[traced_test]
    fn test_extract_json_with_whitespace_and_markers() {
        let content = "   \n```json\n{\"key\": \"value\"}\n```\n   ";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[traced_test]
    fn test_extract_json_no_markers() {
        let content = "   {\"key\": \"value\"}   ";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[traced_test]
    fn test_extract_json_incomplete_markers() {
        let content = "```json{\"key\": \"value\"}";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[traced_test]
    fn test_extract_json_trailing_newline() {
        let content = "```json\n{\"key\": \"value\"}\n\n```";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[traced_test]
    fn test_extract_json_empty_string() {
        let content = "   ";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "");
    }

    #[traced_test]
    fn test_extract_json_no_json_after_marker() {
        let content = "```json```";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "");
    }

    #[traced_test]
    fn test_extract_json_with_crlf_line_endings() {
        // Simulate Windows-style line endings
        let content = "```json\r\n{\"key\": \"value\"}\r\n```";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[traced_test]
    fn test_extract_json_whitespace_after_marker() {
        let content = "```json     \n{\"key\": \"value\"}\n```";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[traced_test]
    fn test_extract_json_random_text_instead_of_json() {
        let content = "```json\nsome random text\n```";
        let result = extract_json_from_possible_backticks_block(content);
        assert_eq!(result, "some random text");
    }
}
