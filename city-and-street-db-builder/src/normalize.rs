crate::ix!();

pub fn normalize(s: &str) -> String {
    // 1. Trim leading/trailing whitespace
    let trimmed = s.trim();
    
    // 2. Convert to lowercase
    let lower = trimmed.to_lowercase();

    // 3. Replace punctuation with spaces.
    //    This ensures that something like "BALTIMORE---CITY" 
    //    becomes "baltimore   city" before we normalize spaces.
    let replaced: String = lower.chars()
        .map(|c| if c.is_ascii_punctuation() { ' ' } else { c })
        .collect();

    // 4. Convert all consecutive whitespace into a single space
    let parts: Vec<&str> = replaced.split_whitespace().collect();
    parts.join(" ")
}

#[cfg(test)]
mod normalize_tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("Baltimore"), "baltimore");
        assert_eq!(normalize("  baltimore  "), "baltimore");
        assert_eq!(normalize("BALTIMORE!!"), "baltimore");
        assert_eq!(normalize("Baltimore, MD"), "baltimore md");
        assert_eq!(normalize("  Baltimore   City  "), "baltimore city");
        assert_eq!(normalize("BALTIMORE---CITY"), "baltimore city"); // this one fails
        assert_eq!(normalize("Baltimore   ,,,   City"), "baltimore city");
    }
}
