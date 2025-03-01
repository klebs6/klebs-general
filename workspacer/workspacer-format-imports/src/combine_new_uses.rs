crate::ix!();

// -------------------------------------------------------------------------
// 6) Combine new uses block with remainder
// -------------------------------------------------------------------------
pub fn combine_new_uses_with_remainder(new_uses_block: &str, remainder: &str) -> String {
    let trimmed_new_uses = new_uses_block.trim();
    if trimmed_new_uses.is_empty() {
        // no actual new uses => just return remainder exactly as is
        return remainder.to_string();
    }
    // otherwise, place the new uses block + a single blank line + remainder
    format!(
        "{}\n{}",
        trimmed_new_uses,
        remainder.trim_start()
    )
}

#[cfg(test)]
mod test_combine_new_uses_with_remainder {
    use super::combine_new_uses_with_remainder;

    /// 1) Both strings are non-empty => new uses on top, remainder after
    #[test]
    fn test_both_non_empty() {
        let new_uses = "some lines\nsome lines\n";
        let remainder = "fn main() {}";
        let out = combine_new_uses_with_remainder(new_uses, remainder);
        // Should be `some lines\nsome lines\n\nfn main() {}`
        assert!(out.starts_with("some lines\nsome lines\n"));
        assert!(out.contains("fn main() {}"));
    }

    /// 2) If new uses is empty => just remainder
    #[test]
    fn test_empty_new_uses() {
        let new_uses = "";
        let remainder = "hello world";
        let out = combine_new_uses_with_remainder(new_uses, remainder);
        assert_eq!(out, "\nhello world");
    }

    // etc...
}
