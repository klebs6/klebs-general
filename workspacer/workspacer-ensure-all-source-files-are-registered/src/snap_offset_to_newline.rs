// ---------------- [ File: src/snap_offset_to_newline.rs ]
crate::ix!();

/// Rounds an offset up to the next newline if it's in the middle of a line,
/// without exceeding `earliest_offset`.
pub fn snap_offset_to_newline(
    initial_offset:  usize,
    earliest_offset: usize,
    old_text:        &str,
) -> usize {
    trace!("Entering snap_offset_to_newline with initial_offset={}, earliest_offset={}", 
           initial_offset, earliest_offset);

    if initial_offset >= old_text.len() {
        debug!("initial_offset >= old_text.len(), returning it as-is");
        return initial_offset;
    }
    if old_text.chars().nth(initial_offset) == Some('\n') {
        debug!("Already at a newline => returning initial_offset unchanged");
        return initial_offset;
    }

    trace!("Searching for next newline from offset={}", initial_offset);
    if let Some(rel_nl_pos) = old_text[initial_offset..].find('\n') {
        let abs_nl = initial_offset + rel_nl_pos + 1;
        let snapped = abs_nl.min(earliest_offset);
        debug!("Found newline at abs_nl={}, clamped to {} => returning {}", abs_nl, earliest_offset, snapped);
        snapped
    } else {
        debug!("No newline found => returning old_text.len()");
        old_text.len()
    }
}

#[cfg(test)]
mod test_snap_offset_to_newline {
    use super::*;

    /// 1) If `initial_offset` >= `old_text.len()`, we return `initial_offset` (i.e. append to end).
    #[traced_test]
    fn test_offset_beyond_eof() {
        let text = "Hello\nWorld\n";
        let initial_offset = 50; // bigger than text.len()
        let earliest_offset = 999; // doesn't matter since we're beyond EOF

        let result = snap_offset_to_newline(initial_offset, earliest_offset, text);
        assert_eq!(
            result, 
            initial_offset, 
            "If offset is beyond EOF, we just return it unchanged"
        );
    }

    /// 2) If `old_text.chars().nth(initial_offset) == Some('\n')`, we keep the offset as is
    #[traced_test]
    fn test_offset_at_newline_already() {
        let text = "Line1\nLine2\nLine3\n";
        // Let's pick the offset at the exact newline after "Line1"
        // "Line1" is length 5 => after that, offset=5. There's a newline char at index=5 indeed 
        let offset_line1_newline = 5;

        let earliest_offset = 100; // big enough not to clamp
        let result = snap_offset_to_newline(offset_line1_newline, earliest_offset, text);

        assert_eq!(
            result,
            offset_line1_newline,
            "We do not shift if we're exactly on a newline"
        );
        assert_eq!(
            text.chars().nth(result),
            Some('\n'),
            "The chosen offset should remain at that newline character"
        );
    }

    /// 3) If `initial_offset` is in the middle of a line => we search for the next newline and clamp to `earliest_offset`.
    #[traced_test]
    fn test_offset_in_middle_of_line() {
        let text = "Line1\nLine2\nLine3";
        // Let's pick an offset that points to the "L" of "Line2", so we'd expect to jump to
        // the newline after "Line2".
        // text = "Line1\nLine2\nLine3"
        //         ^----^
        // indices: 0123456...
        // "Line1" => length 5 + newline => offset 6 is 'L' of "Line2"
        let initial_offset = 6;

        let earliest_offset = 999; // no real clamping
        let result = snap_offset_to_newline(initial_offset, earliest_offset, text);

        // We want to see it jump to the newline after "Line2", i.e. after "Line2\n"
        // "Line2" is 5 chars, plus the newline => that newline is at index 11
        // So next newline is index 11 => we add 1 => 12 => but let's check carefully:
        // Actually the substring from offset=6 is "Line2\nLine3"
        // We find the next '\n' at index=6 + 5 => 11, so the newline is at 11 => +1 => 12
        // We'll see if we do that or if the function returns 12. 
        // Actually the function does: abs_nl = initial_offset + rel_nl_pos + 1
        // rel_nl_pos would be 5 from "Line2\n" => so abs_nl=6+5+1=12
        // That sets the offset to 12, which is the index after '\n'.
        // So let's confirm the char at index=12 is 'L' of "Line3".
        assert_eq!(result, 12);
        assert_eq!(
            text.chars().nth(result),
            Some('L'),
            "Offset should now point to the 'L' of 'Line3', i.e. we've snapped to the line after 'Line2'"
        );
    }

    /// 4) If there's no newline at or after `initial_offset`, we return `old_text.len()`.
    #[traced_test]
    fn test_no_newline_after_offset() {
        let text = "No final newline here";
        // We'll pick an offset somewhere in the middle
        let initial_offset = 3; // the substring is " final newline here" 
        // there's no newline at all, so we go to text.len()

        let earliest_offset = 999;
        let result = snap_offset_to_newline(initial_offset, earliest_offset, text);
        assert_eq!(result, text.len(), "We go to the end of text if no newline is found");
    }

    /// 5) We clamp so that the final offset does not exceed `earliest_offset`.
    #[traced_test]
    fn test_clamp_to_earliest_offset() {
        // We'll create a scenario: next newline is after earliest_offset, 
        // so we do not want to exceed earliest_offset
        // text => "AAAA\nBBBB\n" => length=10
        // We'll pick an initial_offset in the middle of "BBBB", next newline is the final one at index=9
        // but if earliest_offset=8 => we clamp to 8 (i.e. we won't jump all the way to 10).
        let text = "AAAA\nBBBB\n"; 
        // indices: A(0) A(1) A(2) A(3) \n(4) B(5) B(6) B(7) B(8) \n(9)
        let initial_offset = 5; // at 'B'
        let earliest_offset = 8;

        let result = snap_offset_to_newline(initial_offset, earliest_offset, text);
        // we find next newline from offset=5 => that's the one at offset=9 => +1 => 10
        // but we clamp to min(10,8)=> 8
        assert_eq!(result, 8, "We clamp to earliest_offset=8");
    }

    /// 6) If `initial_offset` is exactly `earliest_offset`, we still see if we are on a newline or not.
    ///    If not on a newline, we try to snap. But the min(...) might clamp us right back to earliest_offset => effectively no movement.
    #[traced_test]
    fn test_offset_equals_earliest_offset() {
        let text = "Line1\nLine2\n";
        // Suppose earliest_offset=6 => that's the 'L' of "Line2".
        // initial_offset=6 => not a newline => next newline is index=11 => +1 =>12 => min(12,6) =>6 => no movement 
        let earliest_offset = 6;
        let initial_offset = 6;

        let result = snap_offset_to_newline(initial_offset, earliest_offset, text);
        assert_eq!(
            result,
            6,
            "We remain at offset=6 because snapping tries to go to 12 then clamp => 6"
        );
        assert_eq!(text.chars().nth(result), Some('L'));
    }

    /// 7) If `initial_offset` is 0 and there's an immediate newline => we remain 0 if text starts with newline
    ///    Otherwise, we jump to next newline or end of file. 
    #[traced_test]
    fn test_offset_zero_behavior() {
        let text = "\nHello\nWorld";
        // length=12
        // index 0 => newline => we remain 0
        let earliest_offset = 12;
        let initial_offset = 0;

        let result = snap_offset_to_newline(initial_offset, earliest_offset, text);
        assert_eq!(result, 0, "We remain at 0 if text[0] is newline");
    }

    /// 8) Integration scenario: We place offset in the middle of the first line, there's a newline after that line,
    ///    but earliest_offset is the start of the second line => we might clamp or not. 
    ///    Let's see how we handle a typical scenario
    #[traced_test]
    fn test_integration_middle_first_line() {
        let text = "FirstLine\nSecondLine\nThirdLine";
        // Indexing => "FirstLine\nSecondLine\nThirdLine"
        // We pick offset=2 => "rstLine\nSecondLine..."
        // next newline from offset=2 is at offset=9 => +1 => 10
        // earliest_offset=15 => bigger than 10 => no clamp => => 10
        let initial_offset = 2;
        let earliest_offset = 15;
        let result = snap_offset_to_newline(initial_offset, earliest_offset, text);

        // we expect 10, which should be the index after the '\n' that ends "FirstLine"
        assert_eq!(result, 10, "Should snap to the line after 'FirstLine'");
        assert_eq!(text.chars().nth(result), Some('S'), "Should point to 'S' in 'SecondLine'");
    }

    /// 9) If there's no newline in the entire text => we end at text.len()
    #[traced_test]
    fn test_no_newline_in_text() {
        let text = "CompletelyOneLineNoNewline";
        let initial_offset = 10;
        let earliest_offset = text.len();
        let result = snap_offset_to_newline(initial_offset, earliest_offset, text);
        assert_eq!(result, text.len());
    }

    /// 10) If the offset is on the final newline char => we keep it as is
    #[traced_test]
    fn test_offset_at_final_newline() {
        let text = "A\nB\nC\n";
        // length=6 => indices => A(0), \n(1), B(2), \n(3), C(4), \n(5)
        let initial_offset = 5; 
        assert_eq!(text.chars().nth(initial_offset), Some('\n'));
        let earliest_offset = 999;

        let result = snap_offset_to_newline(initial_offset, earliest_offset, text);
        assert_eq!(result, 5, "Remain at the final newline offset=5");
    }
}
