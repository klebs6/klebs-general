// ---------------- [ File: src/leading_spaces.rs ]
crate::ix!();

/// Returns how many leading space characters are at the start of `line`.
pub fn leading_spaces(line: &str) -> usize {
    let mut count = 0;
    for c in line.chars() {
        if c == ' ' {
            count += 1;
        } else {
            break;
        }
    }
    count
}
