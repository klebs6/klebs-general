// ---------------- [ File: src/combine_new_uses.rs ]
crate::ix!();

pub fn combine_new_uses_with_remainder(new_uses_block: &str, remainder: &str) -> String {
    info!("combine_new_uses_with_remainder => start");
    debug!(
        "new_uses_block.len()={}, remainder.len()={}",
        new_uses_block.len(),
        remainder.len()
    );

    let trimmed_new = new_uses_block.trim();
    if trimmed_new.is_empty() {
        debug!("No new uses => returning remainder as is");
        info!("combine_new_uses_with_remainder => done => returning remainder");
        return remainder.to_string();
    } else {
        debug!("Some new uses => combining with remainder");
        let combined = format!("{}\n{}", trimmed_new, remainder.trim_start());
        info!("combine_new_uses_with_remainder => done => returning combined");
        return combined;
    }
}

#[cfg(test)]
mod test_combine_new_uses_with_remainder {
    use super::*;

    #[test]
    fn test_both_non_empty() {
        info!("test_both_non_empty => start");
        let new_uses = "some lines\nsome lines\n";
        let remainder = "fn main() {}";
        let out = combine_new_uses_with_remainder(new_uses, remainder);
        assert!(out.starts_with("some lines\nsome lines\n"));
        assert!(out.contains("fn main() {}"));
        info!("test_both_non_empty => success");
    }

    #[test]
    fn test_empty_new_uses() {
        info!("test_empty_new_uses => start");
        let new_uses = "";
        let remainder = "hello world";
        let out = combine_new_uses_with_remainder(new_uses, remainder);
        assert_eq!(out, "hello world");
        info!("test_empty_new_uses => success");
    }
}
