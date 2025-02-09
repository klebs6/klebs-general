// ---------------- [ File: src/merge_house_number_range.rs ]
crate::ix!();

/// Merges `new_range` into the existing set of disjoint subranges,
/// returning a new Vec that is still disjoint and sorted.
///
/// If overlapping or adjacent (e.g. [10..=15], [16..20]) => unify => [10..=20].
/// Adjust if adjacency shouldn't unify.
pub fn merge_house_number_range(
    mut existing: Vec<HouseNumberRange>,
    new_range: &HouseNumberRange
) -> Vec<HouseNumberRange> 
{
    existing.push(new_range.clone());
    existing.sort_by_key(|r| *r.start());

    let mut merged = Vec::new();
    for rng in existing {
        if merged.is_empty() {
            merged.push(rng);
        } else {
            let last = merged.last_mut().unwrap();
            // Overlap or adjacency
            if *rng.start() <= *last.end() + 1 {
                if rng.end() > last.end() {
                    last.set_end(*rng.end());
                }
            } else {
                merged.push(rng);
            }
        }
    }
    merged
}

#[cfg(test)]
#[disable]
mod merge_range_tests {
    use super::*;
    use crate::HouseNumberRange;

    #[test]
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

    #[test]
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

    #[test]
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

    #[test]
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
