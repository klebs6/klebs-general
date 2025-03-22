crate::ix!();

pub fn trim_to_60(s: String) -> String {
    if s.len() > 60 {
        let mut end = 60;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        let mut truncated = s[..end].to_string();
        truncated.push_str("...");
        truncated
    } else {
        s
    }
}
