// ---------------- [ File: src/merge_new_subranges.rs ]
crate::ix!();

/// Merges newly extracted subranges into the existing list, returning a consolidated list.
/// This example calls an existing helper (like `merge_house_number_range`) for each range.
///
/// # Returns
///
/// * A new `Vec<HouseNumberRange>` containing the merged results.
pub fn merge_new_subranges(
    mut current: Vec<HouseNumberRange>,
    new_ranges: &Vec<HouseNumberRange>,
) -> Vec<HouseNumberRange> {
    trace!(
        "merge_new_subranges: current={} existing ranges, new={} ranges",
        current.len(),
        new_ranges.len()
    );

    for rng in new_ranges {
        current = merge_house_number_range(current, rng);
    }
    current
}

#[cfg(test)]
#[disable]
mod test_merge_new_subranges {
    use super::*;
    use std::fmt;

    /// A small helper to build a `HouseNumberRange` in tests.
    fn hnr(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// A convenience for printing Vec<HouseNumberRange> in asserts.
    /// This is purely cosmetic: it helps debug test failures.
    fn fmt_ranges(ranges: &[HouseNumberRange]) -> String {
        let items: Vec<String> = ranges
            .iter()
            .map(|rng| format!("[{}..={}]", rng.start(), rng.end()))
            .collect();
        items.join(", ")
    }

    impl fmt::Debug for HouseNumberRange {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "[{}..={}]", self.start(), self.end())
        }
    }

    #[test]
    fn test_no_new_ranges() {
        // If `new_ranges` is empty, the function should return `current` unchanged.
        let current = vec![hnr(1, 5), hnr(10, 15)];
        let new = vec![];
        let merged = merge_new_subranges(current.clone(), &new);
        assert_eq!(merged, current, "Merging empty new ranges should leave current unchanged");
    }

    #[test]
    fn test_current_empty() {
        // If `current` is empty, the result should simply be `new_ranges`.
        let current = vec![];
        let new = vec![hnr(5, 10), hnr(20, 25)];
        let merged = merge_new_subranges(current, &new);
        assert_eq!(
            merged, new,
            "Merging into empty current should yield exactly the new ranges"
        );
    }

    #[test]
    fn test_disjoint_ranges() {
        // If new ranges do not overlap with current, they should be simply appended in sorted order.
        // The underlying merge helper typically sorts or merges, but let's confirm final result.
        let current = vec![hnr(1, 2), hnr(10, 11)];
        let new = vec![hnr(3, 4), hnr(20, 21)];
        let merged = merge_new_subranges(current, &new);

        // Because each insertion calls `merge_house_number_range`, the final set should remain sorted.
        let expected = vec![hnr(1,2), hnr(3,4), hnr(10,11), hnr(20,21)];
        assert_eq!(
            merged, expected,
            "Disjoint ranges should appear in sorted order. Got: {}, Expected: {}",
            fmt_ranges(&merged),
            fmt_ranges(&expected)
        );
    }

    #[test]
    fn test_overlap_new_range() {
        // If new ranges overlap with existing, they should unify.
        // e.g. current=[10..15], new=[12..18] => merged=[10..18].
        let current = vec![hnr(10, 15)];
        let new = vec![hnr(12, 18)];
        let merged = merge_new_subranges(current, &new);
        let expected = vec![hnr(10, 18)];
        assert_eq!(
            merged, expected,
            "Should merge overlapping ranges. Got: {}, Expected: {}",
            fmt_ranges(&merged),
            fmt_ranges(&expected)
        );
    }

    #[test]
    fn test_multiple_new_ranges_some_overlap() {
        // Some new ranges do not overlap, others do.
        // current = [1..5, 10..15]
        // new = [5..6, 12..18, 30..40]
        // Overlaps: new[5..6] merges partially with [1..5], new[12..18] merges with [10..15]
        let current = vec![hnr(1,5), hnr(10,15)];
        let new = vec![hnr(5,6), hnr(12,18), hnr(30,40)];
        let merged = merge_new_subranges(current, &new);

        // We'll see [1..6] merges for the first pair; [10..18] for the second,
        // and [30..40] is disjoint, appended after.
        let expected = vec![hnr(1,6), hnr(10,18), hnr(30,40)];
        assert_eq!(
            merged, expected,
            "Merging multiple new ranges with partial overlaps"
        );
    }

    #[test]
    fn test_new_subrange_is_contained_in_existing() {
        // If new range is fully within an existing range, no expansion needed.
        let current = vec![hnr(10,20)];
        let new = vec![hnr(12,15)];
        let merged = merge_new_subranges(current.clone(), &new);
        assert_eq!(
            merged, current,
            "New subrange is inside existing range => no change"
        );
    }

    #[test]
    fn test_new_range_superset_of_existing() {
        // If new range fully covers an existing range, the result should unify into the new range.
        let current = vec![hnr(100,110)];
        let new = vec![hnr(90,120)];
        let merged = merge_new_subranges(current, &new);
        let expected = vec![hnr(90,120)];
        assert_eq!(merged, expected, "Should unify into a single larger range");
    }

    #[test]
    fn test_merge_is_transitive_within_new_ranges() {
        // If we have multiple new ranges that themselves overlap, we want a final unify.
        // current: [1..2], new: [2..5, 5..7]
        // In many usage patterns, we repeatedly call `merge_house_number_range` for each new range.
        // So [1..2] + [2..5] => [1..5], then merging [5..7] => [1..7].
        let current = vec![hnr(1,2)];
        let new = vec![hnr(2,5), hnr(5,7)];
        let merged = merge_new_subranges(current, &new);
        let expected = vec![hnr(1,7)];
        assert_eq!(merged, expected, "All overlapping subranges unify transitively");
    }

    #[test]
    fn test_merge_order_does_not_affect_result() {
        // The function processes `new_ranges` in the given order, each time merging
        // with `current`. We'll confirm that even if we reorder the new ranges,
        // the final result is consistent (assuming `merge_house_number_range` is stable).
        let current = vec![hnr(10, 11)];
        let new1 = vec![hnr(1, 5), hnr(5, 10), hnr(12, 15)];
        let new2 = vec![hnr(12, 15), hnr(5, 10), hnr(1, 5)];

        let merged1 = merge_new_subranges(current.clone(), &new1);
        let merged2 = merge_new_subranges(current.clone(), &new2);

        assert_eq!(
            merged1, merged2,
            "Reordering new subranges does not change final result"
        );
    }
}
