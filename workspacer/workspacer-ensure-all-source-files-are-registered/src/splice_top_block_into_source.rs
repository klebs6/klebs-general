// ---------------- [ File: src/splice_top_block_into_source.rs ]
crate::ix!();

/// Rebuilds the final text by copying `old_text` except for old macros,
/// and inserting the `final_top_block` at `insertion_offset`.
/// This version ensures:
///    1) We don’t produce double newlines if final_top_block or old_text already ends with a newline.
///    2) If insertion_offset points exactly to a newline and we just forced a newline
///       for final_top_block, we skip copying old_text’s newline again.
pub fn splice_top_block_into_source(
    old_text:         &str,
    old_macros:       &[ExistingXMacro],
    insertion_offset: usize,
    final_top_block:  &str,
) -> String {
    trace!("Entering splice_top_block_into_source (insertion_offset={}, block_len={})",
           insertion_offset, final_top_block.len());

    let mut out = String::new();
    let mut pos = 0;
    let mut inserted_block = false;
    let mut macros_iter = old_macros.iter().peekable();

    while pos < old_text.len() {
        // Check if we should insert final_top_block here
        if !inserted_block && insertion_offset >= pos {
            let next_macro_start = macros_iter
                .peek()
                .map(|em| em.range().start().into())
                .unwrap_or(old_text.len());
            trace!("next_macro_start={}, insertion_offset={}", next_macro_start, insertion_offset);

            if insertion_offset <= next_macro_start {
                debug!("Inserting final_top_block at offset={}", insertion_offset);
                // Copy up to insertion_offset
                if insertion_offset > pos {
                    out.push_str(&old_text[pos..insertion_offset]);
                    pos = insertion_offset;
                }

                // Insert final_top_block
                if !final_top_block.is_empty() {
                    // If out doesn't end with a newline, add one
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                    // Push the block text
                    out.push_str(final_top_block);

                    // Ensure exactly one trailing newline after the block
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }

                    //
                    // [FIX] If old_text has a newline at insertion_offset,
                    // and we've already added a newline for final_top_block,
                    // skip that upcoming newline to avoid duplicates.
                    //
                    // i.e. if "abcde\n" has insertion_offset=5 => we wrote "\nBLOCK\n" 
                    // so final => "abcde\nBLOCK\n". If old_text[5] == '\n', 
                    // we skip that newline from old_text.
                    //
                    if pos < old_text.len() && old_text.as_bytes()[pos] == b'\n' {
                        debug!("Skipping original newline at old_text[{}], to avoid double-newline", pos);
                        pos += 1;
                    }
                }
                inserted_block = true;
            }
        }

        // Skip or copy around macros
        if let Some(m) = macros_iter.peek() {
            let m_start: usize = m.range().start().into();
            let m_end:   usize = m.range().end().into();
            trace!("macro range=({}..{})", m_start, m_end);

            if pos < m_start {
                let slice_end = m_start.min(old_text.len());
                debug!("Copying old_text[{}..{}]", pos, slice_end);
                out.push_str(&old_text[pos..slice_end]);
                pos = slice_end;
            } else if pos < m_end {
                debug!("Skipping macro region [{}..{}]", m_start, m_end);
                pos = m_end;
                macros_iter.next();
            } else {
                macros_iter.next();
            }
        } else {
            debug!("No more macros => copying remainder old_text[{}..]", pos);
            out.push_str(&old_text[pos..]);
            pos = old_text.len();
        }
    }

    // If block wasn't inserted yet, append at the end
    if !inserted_block && !final_top_block.is_empty() {
        debug!("Never inserted block => appending at end");
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(final_top_block);
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }

    debug!("splice_top_block_into_source done, final length={}", out.len());
    trace!("Exiting splice_top_block_into_source");
    out
}

#[cfg(test)]
mod test_splice_top_block_into_source {
    use super::*;
    use ra_ap_syntax::{TextSize};
    use std::iter;

    /// A helper to create an `ExistingXMacro` with a given `start`..`end` range (usize-based),
    /// and a given macro text. We'll convert the range to TextSize for you.
    fn make_macro(text: &str, start: usize, end: usize) -> ExistingXMacro {
        let start_sz = TextSize::from(start as u32);
        let end_sz   = TextSize::from(end as u32);

        ExistingXMacroBuilder::default()
            .text(text.to_string())
            .range(TextRange::new(start_sz, end_sz))
            .build()
            .unwrap()
    }

    /// 1) If we have no macros at all, we simply insert `final_top_block` at `insertion_offset`.
    ///    If `insertion_offset` is within the text, we split there; if it's beyond text len, we append at the end.
    #[traced_test]
    fn test_no_macros_simple_insertion() {
        let old_text = "hello world";
        let old_macros = vec![];

        // Let's pick an insertion_offset in the middle of "hello"
        let insertion_offset = 3;
        let final_top_block  = "// block\nx!{some_stem}";

        let result = splice_top_block_into_source(old_text, &old_macros, insertion_offset, final_top_block);

        // We'll see the substring up to offset=3 => "hel", then a newline + final_top_block + newline,
        // then the rest => "lo world"
        let expected = format!(
            "hel\n{}\nlo world",
            final_top_block
        );
        assert_eq!(result, expected, "Should splice final_top_block at offset=3");
    }

    /// 2) If `final_top_block` is empty => we skip inserting anything,
    ///    but we still proceed with skipping macros, etc. 
    ///    We'll confirm that if no macros => the final equals old_text if insertion_offset is less than len.
    #[traced_test]
    fn test_empty_final_top_block() {
        let old_text = "some text with no macros";
        let old_macros = vec![];
        let insertion_offset = 5;
        let final_top_block = "";

        let result = splice_top_block_into_source(old_text, &old_macros, insertion_offset, final_top_block);
        // Because final_top_block is empty, we won't splice anything. 
        // So we just get old_text as-is.
        assert_eq!(result, old_text, "No change if final block is empty");
    }

    /// 3) If insertion_offset is beyond old_text.len() => we append final_top_block at end (with a preceding newline if needed).
    #[traced_test]
    fn test_insertion_offset_beyond_eof() {
        let old_text = "hello\nworld";
        let old_macros = vec![];
        let insertion_offset = 999; 
        let final_top_block = "// top block\nx!{stuff}";

        let result = splice_top_block_into_source(old_text, &old_macros, insertion_offset, final_top_block);

        // We'll see the entire old_text + newline + final_top_block + newline
        let expected = format!(
            "{}\n{}\n",
            old_text, 
            final_top_block
        );
        assert_eq!(result, expected);
    }

    /// 4) If we have macros, we skip them in the final output. 
    ///    We'll place final_top_block at insertion_offset if that is < next_macro_start.
    #[traced_test]
    fn test_skip_macros_and_insert_before_first_macro() {
        let old_text = "abcDEFGHIJK"; 
        // Let's say there's a macro from offset=3..6 => "DEF"
        // We'll skip that substring in final output. 
        // We'll put insertion_offset=2 => "ab" + block + "c" => note the 'D'..'F' is removed
        let old_macros = vec![
            make_macro("x!{macro_1}", 3, 6) // covers substring "DEF"
        ];
        let insertion_offset = 2; // after "ab"
        let final_top_block = "// inserted\nx!{something}";

        let result = splice_top_block_into_source(old_text, &old_macros, insertion_offset, final_top_block);

        // We expect:
        //   - copy old_text[0..2] => "ab"
        //   - newline + final_top_block + newline
        //   - skip macro region [3..6] => that's "DEF"
        //   - copy old_text[2..3]? Actually notice the offset=2 => we copied 0..2,
        //     then we haven't advanced pos to 3 yet. We'll see the logic in the loop:
        //     Actually the code will skip the region from 3..6, so we'll see the portion [2..3] => "c"
        //   - after skipping the macro, pos=6 => copy the remainder => old_text[6..] => "GHIJK"

        // So final => "ab\n// inserted\nx!{something}\ncGHIJK"
        let expected = "ab\n// inserted\nx!{something}\ncGHIJK";
        assert_eq!(result, expected);
    }

    /// 5) If insertion_offset is after a macro's start, we wait for the next cycle when pos >= insertion_offset 
    ///    or if insertion_offset > next_macro_start => we skip that macro first, etc.
    ///    This checks that the code doesn't do partial splices in the middle of skipping a macro.
    #[traced_test]
    fn test_insertion_offset_after_macro_start() {
        // We'll contrive a scenario:
        // old_text = "ABCDEFG"
        // macro => offset=2..5 => "CDE"
        // insertion_offset=3 => means we want to place the block at offset=3,
        //   but offset=3 is inside the macro region [2..5].
        // So the logic will skip the entire macro region first, then see if insertion_offset <= next_macro_start
        // or not. Because insertion_offset=3 < macro_start=2 ?  Actually we see we find next_macro_start=2 => insertion_offset=3 => insertion_offset>2 => we skip macro first.
        let old_text = "ABCDEFG";
        let old_macros = vec![
            make_macro("x!{macro_skip}", 2, 5), // "CDE"
        ];
        let insertion_offset = 3;
        let final_top_block = "XBLOCK";

        let result = splice_top_block_into_source(old_text, &old_macros, insertion_offset, final_top_block);

        // Let's step through:
        //   pos=0, insertion_offset=3 => next_macro_start=2
        //   insertion_offset <= next_macro_start? => 3 <=2? false => skip it. So we copy old_text[0..2] => "AB"
        //   now pos=2, we skip the macro region [2..5], pos=5
        //   no more macros => we copy old_text[5..] => "FG" => so out="ABFG"
        //   we never inserted the block => so at the end we do =>  out += "\nXBLOCK\n"
        //
        // So final => "ABFG\nXBLOCK\n"
        // The insertion_offset=3 got overshadowed by skipping the macro at [2..5].
        let expected = "ABFG\nXBLOCK\n";
        assert_eq!(result, expected);
    }

    /// 6) If insertion_offset < pos in the loop, we can't insert in the middle => we eventually do it at the end if not inserted.
    #[traced_test]
    fn test_insertion_offset_already_passed() {
        // old_text = "ABCDEFG"
        // no macros
        // We'll do insertion_offset=2, but in the logic we can contrive that pos advances beyond 2 
        // if code doesn't insert. Actually let's do no macros => the code tries to insert at offset=2 
        // but if the loop iteration passes pos=2 for some reason, 
        // let's see how the code handles it. 
        // Actually the code says if insertion_offset >= pos => we try to do it. So first iteration pos=0 => insertion_offset=2 => 2>=0 => but we do it only if insertion_offset <= next_macro_start => next_macro_start=old_text.len()=7 => yes => so we do it. 
        // That means we do expect the insertion to happen. 
        //
        // We need a scenario that fails to insert? Actually let's introduce a macro that starts before insertion_offset => the code sees next_macro_start=2 => insertion_offset=2 => we do it, we might do partial copy. Let's skip that scenario:
        // We'll demonstrate that if for some reason the code doesn't insert, it appends at the end. 
        // We'll do insertion_offset=2, but let's say the macro is at 1..3 => we skip it. 
        // Then pos=3 => we never see insertion_offset <= next_macro_start => no insertion => at the end we do it. 
        let old_text = "ABCDEFG";
        let old_macros = vec![
            make_macro("x!{macro_at_1_3}", 1, 3), // "B", offset=1..3 is "BC"? Actually that'll skip "BC"
        ];
        let insertion_offset = 2;
        let final_top_block = "INSERTED_BLOCK";

        // Step logic:
        //   pos=0, insertion_offset=2 => next_macro_start=1 => insertion_offset<=1 => 2<=1 => false => skip insertion
        //   copy old_text[0..1] => "A", pos=1
        //   skip macro region [1..3], pos=3
        //   done macro => copy old_text[3..] => "DEFG", pos=7
        //   no insertion yet => final => "ADEFG\nINSERTED_BLOCK\n"
        let result = splice_top_block_into_source(old_text, &old_macros, insertion_offset, final_top_block);
        let expected = "ADEFG\nINSERTED_BLOCK\n";
        assert_eq!(result, expected);
    }

    /// 7) Multiple macros => we skip them each, we also do insertion if offset is in between
    #[traced_test]
    fn test_multiple_macros_and_insertion_in_between() {
        // old_text => "abcdefghij"
        // macros => 
        //   1) offset=2..4 => "cd"
        //   2) offset=6..8 => "gh"
        // insertion_offset=5 => somewhere between the two macros
        // final_top_block => "BBB"
        //
        // We proceed:
        //   pos=0 => next_macro_start=2 => insertion_offset=5>2 => skip insertion => copy "ab"
        //   skip macro [2..4] => "cd"
        //   pos=4 => next_macro_start=6 => insertion_offset=5>=4 => yes => insertion_offset<=6 => yes => so we copy old_text[4..5] => "e" => actually the code checks if insertion_offset>pos => yes => we do partial copy "e"? Let's step carefully:
        //   Actually the code does the insertion check prior to copying up to macro => we see insertion_offset=5, pos=4 => next_macro_start=6 => 5<=6 => so we do partial copy => old_text[4..5] => "e". Then splice block => then newline => pos=5 => 
        //   Then we continue => we see macro at [2..4] is done, next macro is [6..8].
        //   from pos=5 => next_macro_start=6 => copy old_text[5..6] => "f"
        //   skip macro [6..8] => "gh"
        //   copy remainder => old_text[8..] => "ij"
        // final => "abefij" with the block spliced at offset=5 => which is after "abe". 
        // Actually let's do it carefully in code. 
        let old_text = "abcdefghij";
        let old_macros = vec![
            make_macro("x!{macro_cd}", 2, 4), // skip "cd"
            make_macro("x!{macro_gh}", 6, 8), // skip "gh"
        ];
        let insertion_offset = 5; // i.e. after "abcde"
        let final_top_block = "BBB";

        let result = splice_top_block_into_source(old_text, &old_macros, insertion_offset, final_top_block);
        // We expect:
        //   "ab" (copy up to macro1) skip macro1 => pos=4 => next_macro_start=6 => insertion_offset=5 >= pos=4 => insertion_offset<=6 => yes => 
        //   copy old_text[4..5]="e", out= "abe"
        //   insert block => out="abe\nBBB\n"
        //   pos=5 => continue => now next_macro_start=6 => copy old_text[5..6]="f" => out="abe\nBBB\nf"
        //   skip macro2 => skip "gh", pos=8 => copy remainder => "ij" => out="abe\nBBB\nfij"
        let expected = "abe\nBBB\nfij";
        assert_eq!(result, expected);
    }

    /// 8) If final_top_block is not empty but we do insertion at the very end because insertion_offset > old_text.len()
    #[traced_test]
    fn test_insertion_offset_after_eof_but_macros_exist() {
        let old_text = "abcde";
        // macro => offset=1..2 => skip "b"
        let old_macros = vec![
            make_macro("x!{macro_b}", 1,2)
        ];
        let insertion_offset = 999;
        let final_top_block = "INSERT_BLOCK";

        // We'll skip [1..2], so the final text before insertion is => "acde"
        // Then we append => out="acde\nINSERT_BLOCK\n"
        let result = splice_top_block_into_source(old_text, &old_macros, insertion_offset, final_top_block);
        let expected = "acde\nINSERT_BLOCK\n";
        assert_eq!(result, expected);
    }

    /// 9) If we do succeed in inserting the block in the middle, we do place a newline if the text doesn't end with newline.
    ///    We'll confirm that if `out` already ends with a newline, we skip adding an extra one.
    #[traced_test]
    fn test_prevent_double_newline() {
        let old_text = "abcde";
        // no macros
        let old_macros = vec![];
        let insertion_offset = 5; // after 'abcde'
        // but let's contrive that old_text had a trailing newline => so effectively "abcde\n" 
        // Actually let's do it simpler: old_text doesn't have a trailing newline => so we do add one if final_top_block is not empty
        let final_top_block = "BLOCK";

        let result = splice_top_block_into_source(old_text, &old_macros, insertion_offset, final_top_block);
        // => "abcde\nBLOCK\n"
        let expected = "abcde\nBLOCK\n";
        assert_eq!(result, expected);

        // now let's do a variation: old_text ends with newline => "abcde\n"
        let old_text2 = "abcde\n";
        let result2 = splice_top_block_into_source(old_text2, &old_macros, insertion_offset, final_top_block);
        // The code: if !out.ends_with('\n') => out.push('\n'); 
        // but out ends with '\n' => so we skip that step => we just add "BLOCK\n"
        let expected2 = "abcde\nBLOCK\n";
        assert_eq!(result2, expected2);
    }

    /// 10) If we never inserted in the loop, but final_top_block is empty => we do nothing at the end.
    ///    (We tested a scenario with an empty block earlier, but let's just confirm the logic).
    #[traced_test]
    fn test_no_insertion_and_empty_block_at_end() {
        let old_text = "ABCDEFG";
        // We'll skip [2..5], insertion_offset=2 => we never insert. final_top_block = "" => 
        // final => "ABFG"
        let old_macros = vec![
            make_macro("x!{macro_skip}", 2,5)
        ];
        let insertion_offset = 2;
        let final_top_block = "";

        let result = splice_top_block_into_source(old_text, &old_macros, insertion_offset, final_top_block);
        let expected = "ABFG";
        assert_eq!(result, expected);
    }
}
