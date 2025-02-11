// ---------------- [ File: src/normalize.rs ]
// ---------------- [ File: src/normalize.rs ]
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

    #[traced_test]
    fn test_normalize() {
        assert_eq!(normalize("Baltimore"), "baltimore");
        assert_eq!(normalize("  baltimore  "), "baltimore");
        assert_eq!(normalize("BALTIMORE!!"), "baltimore");
        assert_eq!(normalize("Baltimore, MD"), "baltimore md");
        assert_eq!(normalize("  Baltimore   City  "), "baltimore city");
        assert_eq!(normalize("BALTIMORE---CITY"), "baltimore city"); // this one fails
        assert_eq!(normalize("Baltimore   ,,,   City"), "baltimore city");
    }

    /// Tests the “happy path” scenarios already in the code.
    /// Specifically checks trimming, lowercasing, and punctuation -> space.
    #[traced_test]
    fn test_basic_cases() {
        assert_eq!(normalize("Baltimore"), "baltimore");
        assert_eq!(normalize("  baltimore  "), "baltimore");
        assert_eq!(normalize("BALTIMORE!!"), "baltimore");
        assert_eq!(normalize("Baltimore, MD"), "baltimore md");
        assert_eq!(normalize("  Baltimore   City  "), "baltimore city");
        // The original comment said “this one fails,” but presumably your code 
        // now handles “---” => multiple spaces => collapsed => “baltimore city”
        assert_eq!(normalize("BALTIMORE---CITY"), "baltimore city");
        assert_eq!(normalize("Baltimore   ,,,   City"), "baltimore city");
    }

    /// Test empty string => expect empty after normalization.
    #[traced_test]
    fn test_empty_string() {
        assert_eq!(normalize(""), "");
    }

    /// Test a string with only whitespace => expect empty.
    #[traced_test]
    fn test_only_whitespace() {
        assert_eq!(normalize("   "), "");
        assert_eq!(normalize("\t\n\r"), "");
    }

    /// Test a string with repeated punctuation => they all become spaces => collapsed.
    #[traced_test]
    fn test_repeated_punctuation() {
        // "!!!" => " " => then trimmed => ""
        assert_eq!(normalize("!!!"), "");
        // "???Hello???" => "   hello   " => "hello"
        assert_eq!(normalize("???Hello???"), "hello");
        // Mixed punctuation and letters
        assert_eq!(normalize("###Hello--World!!!"), "hello world");
    }

    /// Test numeric and alphanumeric strings => ensure they remain except that punctuation is removed.
    #[traced_test]
    fn test_numeric_and_alphanumeric() {
        // "Route66" => "route66"
        assert_eq!(normalize("Route66"), "route66");
        // "1234---5678" => "1234   5678" => "1234 5678"
        assert_eq!(normalize("1234---5678"), "1234 5678");
        // "Flat#4" => "flat 4"
        assert_eq!(normalize("Flat#4"), "flat 4");
    }

    /// Test a string containing ASCII punctuation at the edges.
    #[traced_test]
    fn test_leading_trailing_punctuation() {
        // "!!!Hello..." => "   hello   " => "hello"
        assert_eq!(normalize("!!!Hello..."), "hello");
    }

    /// Test a scenario with multiple consecutive punctuation and spaces => 
    /// ensures all collapsing happens properly.
    #[traced_test]
    fn test_complex_collapsing() {
        let input = " - -  BALTIMORE,,   CITY---MD ??? ";
        // Step by step:
        //  1) Trim => " - -  BALTIMORE,,   CITY---MD ???"
        //  2) Lower => " - -  baltimore,,   city---md ???"
        //  3) Punct => "   baltimore   city   md     "
        //  4) Collapse => "baltimore city md"
        assert_eq!(normalize(input), "baltimore city md");
    }

    /// Test a large string to ensure performance is acceptable and logic is correct.
    #[traced_test]
    fn test_long_string() {
        // Construct a 500-character string with punctuation
        let repeated = "ABC-123, ".repeat(50); // ~8 chars * 50 => 400 plus spaces => about 450
        let input = format!("  {}!?!?  ", repeated); // add some punctuation
        let output = normalize(&input);

        // We can do some basic checks:
        assert!(!output.contains(","), "No commas left after punctuation removal");
        assert!(!output.contains("!"),  "No exclamation left after punctuation removal");
        // Contains at least 'abc-123' => 'abc 123' after normalization
        assert!(output.contains("abc 123"), "Should see normalized alphanumeric chunks");
    }

    /// Test that non-ASCII punctuation is *not* removed because we only check `is_ascii_punctuation`.
    /// For instance, an em dash (—) will remain as is (converted to lower if it's in letter form, but dash isn't a letter).
    #[traced_test]
    fn test_unicode_punctuation() {
        // "City—State" => the '—' is NOT ASCII punctuation => remains unchanged
        // after lowercasing => "city—state"
        // But spaces remain as is, we only treat ASCII punctuation with `.is_ascii_punctuation()`.
        let input = "City—State";
        let out = normalize(input);
        assert_eq!(out, "city—state", "Em-dash remains because it's not ASCII punctuation");
    }

    /// A final test verifying the function is stable if we call it multiple times.
    /// i.e. `normalize(normalize(s)) == normalize(s)`.
    #[traced_test]
    fn test_idempotence() {
        let inputs = vec![
            "   Baltimore---CITY  ",
            "HELLO###WORLD   123!",
            "",
            "City—State" // note that em dash remains
        ];
        for s in inputs {
            let once = normalize(s);
            let twice = normalize(&once);
            assert_eq!(once, twice, "normalize(...) is idempotent");
        }
    }
}
