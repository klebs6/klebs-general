// ---------------- [ File: src/unify_new_and_existing_ranges.rs ]
crate::ix!();

/// Merges newly provided house‐number ranges with the existing set.
/// Uses `merge_house_number_range` for each new range.
pub fn unify_new_and_existing_ranges(
    mut current: Vec<HouseNumberRange>,
    new_ranges: &[HouseNumberRange],
) -> Vec<HouseNumberRange> {
    trace!(
        "unify_new_and_existing_ranges: merging {} new ranges into {} existing",
        new_ranges.len(),
        current.len()
    );

    for rng in new_ranges {
        current = merge_house_number_range(current, rng);
    }
    current
}

#[cfg(test)]
#[disable]
mod test_unify_new_and_existing_ranges {
    use super::*;
    use tracing::trace;

    /// A convenience function for building a `HouseNumberRange`.
    fn hnr(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// A helper to pretty-print a vector of HouseNumberRange for test failure messages.
    fn fmt_ranges(ranges: &[HouseNumberRange]) -> String {
        let strs: Vec<String> = ranges
            .iter()
            .map(|r| format!("[{}..={}]", r.start(), r.end()))
            .collect();
        strs.join(", ")
    }

    #[test]
    fn test_no_new_ranges() {
        // If `new_ranges` is empty, the original `current` should remain unchanged.
        let current = vec![hnr(1, 10), hnr(20, 25)];
        let new = vec![];

        let result = unify_new_and_existing_ranges(current.clone(), &new);
        assert_eq!(
            result, current,
            "Empty new ranges => current unchanged"
        );
    }

    #[test]
    fn test_no_existing_ranges() {
        // If `current` is empty, the result should just be `new_ranges`.
        let current = vec![];
        let new = vec![hnr(10, 15), hnr(30, 35)];

        let result = unify_new_and_existing_ranges(current, &new);
        assert_eq!(
            result, new,
            "No existing ranges => just the new ranges"
        );
    }

    #[test]
    fn test_disjoint_new_ranges() {
        // new ranges do not overlap => appended in sorted order by merge_house_number_range.
        // current=[1..5, 10..15], new=[6..7, 18..20]
        let current = vec![hnr(1, 5), hnr(10, 15)];
        let new = vec![hnr(6, 7), hnr(18, 20)];
        // Merging each: merges disjoint => final => [1..5, 6..7, 10..15, 18..20]
        let result = unify_new_and_existing_ranges(current, &new);

        let expected = vec![hnr(1,5), hnr(6,7), hnr(10,15), hnr(18,20)];
        assert_eq!(
            result, expected,
            "Should yield sorted, disjoint subranges.\nGot: {}\nExpected: {}",
            fmt_ranges(&result),
            fmt_ranges(&expected)
        );
    }

    #[test]
    fn test_merge_overlapping_new_ranges() {
        // current=[10..15], new=[12..18]
        // final => [10..18]
        let current = vec![hnr(10,15)];
        let new = vec![hnr(12,18)];
        let result = unify_new_and_existing_ranges(current, &new);

        let expected = vec![hnr(10,18)];
        assert_eq!(
            result, expected,
            "Overlapping range => unify. Got: {}, Expected: {}",
            fmt_ranges(&result),
            fmt_ranges(&expected),
        );
    }

    #[test]
    fn test_multiple_new_ranges_some_overlap() {
        // current=[1..5, 10..15], new=[5..6, 12..18, 30..40]
        // merge => 
        //   1) with [5..6]: merges partially with [1..5] => [1..6, 10..15]
        //   2) with [12..18]: merges with [10..15] => [1..6, 10..18]
        //   3) with [30..40]: disjoint => [1..6, 10..18, 30..40]
        let current = vec![hnr(1,5), hnr(10,15)];
        let new = vec![hnr(5,6), hnr(12,18), hnr(30,40)];
        let result = unify_new_and_existing_ranges(current, &new);

        let expected = vec![hnr(1,6), hnr(10,18), hnr(30,40)];
        assert_eq!(
            result, expected,
            "Merging partial overlaps & disjoint. Got: {}, Expected: {}",
            fmt_ranges(&result),
            fmt_ranges(&expected),
        );
    }

    #[test]
    fn test_subrange_new_range() {
        // new range is fully contained in an existing range => no expansion.
        let current = vec![hnr(10,20)];
        let new = vec![hnr(12,15)];
        let result = unify_new_and_existing_ranges(current.clone(), &new);

        assert_eq!(result, current,
            "New subrange inside existing => no change");
    }

    #[test]
    fn test_superset_new_range() {
        // new range fully covers an existing range => unify => new bigger range
        let current = vec![hnr(100,110)];
        let new = vec![hnr(90,120)];
        let result = unify_new_and_existing_ranges(current, &new);

        let expected = vec![hnr(90,120)];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_is_transitive_across_new_ranges() {
        // If new has overlapping items among themselves, the final merges them all.
        // current=[10..11], new=[11..14, 14..16] => step by step:
        //  - first new => merges with [10..11] => [10..14]
        //  - second new => merges with [10..14] => [10..16]
        let current = vec![hnr(10,11)];
        let new = vec![hnr(11,14), hnr(14,16)];
        let result = unify_new_and_existing_ranges(current, &new);

        let expected = vec![hnr(10,16)];
        assert_eq!(
            result, expected,
            "Should unify across new overlapping ranges."
        );
    }

    #[test]
    fn test_merge_order_independence() {
        // unify_new_and_existing_ranges processes new ranges in order, but final result
        // should be consistent. We'll confirm the final set doesn't differ if we reorder new.
        let current = vec![hnr(10, 15)];
        let new1 = vec![hnr(1,2), hnr(16,18)];
        let new2 = vec![hnr(16,18), hnr(1,2)];

        let merged1 = unify_new_and_existing_ranges(current.clone(), &new1);
        let merged2 = unify_new_and_existing_ranges(current, &new2);

        assert_eq!(merged1, merged2,
            "Reordering new subranges should not change final result");
    }
}
