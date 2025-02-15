// ---------------- [ File: src/merge_house_number_range.rs ]
crate::ix!();

/// Merges `new_range` into the existing set of subranges, **unifying both
/// overlapping and adjacent** subranges.  
///
/// e.g. if existing has `[1..5]` and `new_range` is `[6..7]`, we now unify  
/// them into `[1..7]`.  
///
/// # Note
///
/// If you intended disjoint-but-adjacent subranges to remain separate,
/// do *not* use the “<= last.end() + 1” check below. But the test_adjacent_ranges
/// scenario wants them merged, so we unify adjacency.
pub fn merge_house_number_range(
    mut existing: Vec<HouseNumberRange>,
    new_range: &HouseNumberRange
) -> Vec<HouseNumberRange> 
{
    // Insert new range, then sort by start
    existing.push(new_range.clone());
    existing.sort_by_key(|r| *r.start());

    let mut merged = Vec::with_capacity(existing.len());
    for rng in existing {
        if merged.is_empty() {
            merged.push(rng);
        } else {
            let last = merged.last_mut().unwrap();

            // For adjacency or overlap, check if rng.start() <= last.end() + 1
            // so that [1..10] & [11..15] => unify => [1..15].
            if *rng.start() <= *last.end() + 1 {
                // unify => extend last.end if needed
                if rng.end() > last.end() {
                    last.set_end(*rng.end());
                }
            } else {
                // truly disjoint => push new subrange
                merged.push(rng);
            }
        }
    }
    merged
}

#[cfg(test)]
mod merge_range_tests {
    use super::*;
    use crate::HouseNumberRange;

    #[traced_test]
    fn test_disjoint_ranges() {
        let existing = vec![
            HouseNumberRange::new(1, 10),
        ];
        let new = HouseNumberRange::new(20, 20);
        let merged = merge_house_number_range(existing, &new);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], HouseNumberRange::new(1, 10));
        assert_eq!(merged[1], HouseNumberRange::new(20, 20));
    }

    #[traced_test]
    fn test_overlapping_ranges() {
        // existing=[1..=10], new=[8..=12] => unify => [1..=12]
        let existing = vec![
            HouseNumberRange::new(1, 10),
        ];
        let new = HouseNumberRange::new(8, 12);
        let merged = merge_house_number_range(existing, &new);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], HouseNumberRange::new(1, 12));
    }

    #[traced_test]
    fn test_adjacent_ranges() {
        // existing=[1..=10], new=[11..=15] => unify => [1..=15]
        let existing = vec![
            HouseNumberRange::new(1, 10),
        ];
        let new = HouseNumberRange::new(11, 15);
        let merged = merge_house_number_range(existing, &new);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], HouseNumberRange::new(1, 15));
    }

    #[traced_test]
    fn test_merge_multiple() {
        let existing = vec![
            HouseNumberRange::new(1, 5),
            HouseNumberRange::new(10, 15),
        ];
        let new = HouseNumberRange::new(7, 12);
        // Sort => [1..=5, 7..=12, 10..=15], unify => [1..=5, 7..=15]
        let merged = merge_house_number_range(existing, &new);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], HouseNumberRange::new(1, 5));
        assert_eq!(merged[1], HouseNumberRange::new(7, 15));
    }
}
