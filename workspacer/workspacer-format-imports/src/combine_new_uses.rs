// ---------------- [ File: src/combine_new_uses.rs ]
crate::ix!();

pub fn combine_new_uses_with_remainder(new_uses_block: &str, remainder: &str) -> String {
    // If there's no actual new uses, return remainder as is.
    // If there *are* some new uses, place them + a newline + the remainder.
    let trimmed_new = new_uses_block.trim();
    if trimmed_new.is_empty() {
        remainder.to_string()
    } else {
        format!("{}\n{}", trimmed_new, remainder.trim_start())
    }
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

    #[test]
    fn test_empty_new_uses() {
        let new_uses = "";
        let remainder = "hello world";
        let out = combine_new_uses_with_remainder(new_uses, remainder);
        // Now we expect no extra newline => "hello world" exactly.
        assert_eq!(out, "hello world");
    }

    // etc...
}
