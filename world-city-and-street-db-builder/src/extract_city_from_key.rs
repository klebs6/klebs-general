// ---------------- [ File: src/extract_city_from_key.rs ]
crate::ix!();

pub fn extract_city_from_key(key_str: &str) -> Option<String> {
    trace!("extract_city_from_key: analyzing key='{}'", key_str);

    let parts: Vec<&str> = key_str.splitn(3, ':').collect();
    if parts.len() < 3 {
        warn!(
            "extract_city_from_key: key='{}' does not contain at least 3 parts; ignoring",
            key_str
        );
        return None;
    }

    let city_part = parts[2];
    if city_part.is_empty() {
        warn!(
            "extract_city_from_key: key='{}' has empty city substring after second colon; ignoring",
            key_str
        );
        return None;
    }

    Some(city_part.to_string())
}

#[cfg(test)]
mod extract_city_from_key_tests {
    use super::*;

    /// Helper to enable capturing the logs if you want to assert the `warn!` message.
    /// We’ll demonstrate typical usage with the `traced_test` crate, or you can do another approach.
    
    #[traced_test]
    fn test_extract_city_from_key_proper_structure() {
        let key = "C2Z:US:baltimore";
        let city_opt = extract_city_from_key(key);
        assert!(city_opt.is_some());
        assert_eq!(city_opt.unwrap(), "baltimore");
        // No warning logs expected
        // assert!(logs_contain("analyzing key='C2Z:US:baltimore'"));
        // assert!(!logs_contain("does not contain 3 parts"));
    }

    #[traced_test]
    fn test_extract_city_from_key_not_enough_parts() {
        let key = "C2Z:US";
        let city_opt = extract_city_from_key(key);
        assert!(city_opt.is_none(), "Missing the 3rd part => None");
        // Expect a warning in logs
        // assert!(logs_contain("does not contain 3 parts; ignoring"));
    }

    #[traced_test]
    fn test_extract_city_from_key_more_than_3_parts() {
        // e.g. "C2Z:US:baltimore:somethingExtra"
        // with `splitn(3, ':')` => parts => ["C2Z", "US", "baltimore:somethingExtra"]
        // => city => "baltimore:somethingExtra"
        let key = "C2Z:US:baltimore:somethingExtra";
        let city_opt = extract_city_from_key(key);
        assert!(city_opt.is_some());
        assert_eq!(city_opt.unwrap(), "baltimore:somethingExtra");
        // no warning
        // assert!(!logs_contain("does not contain 3 parts"));
    }

    #[traced_test]
    fn test_extract_city_from_key_empty_city() {
        // "C2Z:US:" => 3 parts => ["C2Z","US",""] => city => ""
        let key = "C2Z:US:";
        let city_opt = extract_city_from_key(key);
        assert!(city_opt.is_none());
    }

    #[traced_test]
    fn test_extract_city_from_key_no_colons_at_all() {
        let key = "NoColonsHere";
        let city_opt = extract_city_from_key(key);
        assert!(city_opt.is_none());
        // assert!(logs_contain("does not contain 3 parts; ignoring"));
    }

    #[traced_test]
    fn test_extract_city_from_key_prefix_not_c2z() {
        // The function does not specifically check "C2Z". 
        // If it has 3 parts, it returns the third part. 
        let key = "ABC:XYZ:mycity";
        let city_opt = extract_city_from_key(key);
        assert!(city_opt.is_some());
        assert_eq!(city_opt.unwrap(), "mycity");
        // no warning
        // assert!(!logs_contain("does not contain 3 parts"));
    }

    #[traced_test]
    fn test_extract_city_from_key_special_chars() {
        let key = "C2Z:US:baltimore!!! city??";
        let city_opt = extract_city_from_key(key);
        assert_eq!(city_opt, Some("baltimore!!! city??".to_string()));
    }
}
