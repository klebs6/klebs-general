// ---------------- [ File: workspacer-format-imports/src/combine_new_uses.rs ]
crate::ix!();

/**
  The key fix: remove the `trim_start()` on the remainder.  
  We want to preserve leading blank lines in the remainder so that if 
  there was a blank line after some block comment, it stays in the final output.
*/
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
        // Keep remainder fully intact
        return remainder.to_string();
    } else {
        debug!("Some new uses => combining with remainder");
        // Instead of trimming the remainder’s start, 
        // keep it as-is so any leading blank lines remain.
        let combined = format!("{}\n{}", trimmed_new, remainder);
        info!("combine_new_uses_with_remainder => done => returning combined");
        combined
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
