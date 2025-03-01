// ---------------- [ File: src/extract_stem.rs ]
crate::ix!();

// Simple helper for extracting "stuff" from x!{stuff}
pub fn extract_stem(full_text: &str) -> Option<String> {
    let start = full_text.find('{')?;
    let end   = full_text.rfind('}')?;
    Some(full_text[start+1 .. end].trim().to_string())
}
