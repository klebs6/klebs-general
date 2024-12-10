crate::ix!();

pub fn intersect_date_ranges(
    dates_a: &[NaiveDate],
    dates_b: &[NaiveDate],
) -> Vec<NaiveDate> {
    let set_a: HashSet<_> = dates_a.iter().cloned().collect();
    let set_b: HashSet<_> = dates_b.iter().cloned().collect();
    let mut intersection: Vec<NaiveDate> = set_a.intersection(&set_b).cloned().collect();
    intersection.sort(); // Ensure the output is sorted
    intersection
}

#[cfg(test)]
mod intersect_date_ranges_tests {
    use super::*;
    use chrono::NaiveDate;
    use std::collections::HashSet;

    #[test]
    fn test_empty_inputs() {
        let dates_a: Vec<NaiveDate> = vec![];
        let dates_b: Vec<NaiveDate> = vec![];
        let result = intersect_date_ranges(&dates_a, &dates_b);
        assert!(result.is_empty(), "Intersection of empty inputs should be empty.");
    }

    #[test]
    fn test_non_overlapping_date_ranges() {
        let dates_a = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        ];
        let dates_b = vec![
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(),
        ];
        let result = intersect_date_ranges(&dates_a, &dates_b);
        assert!(result.is_empty(), "Non-overlapping ranges should yield an empty intersection.");
    }

    #[test]
    fn test_identical_date_ranges() {
        let dates_a = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        ];
        let dates_b = dates_a.clone();
        let result = intersect_date_ranges(&dates_a, &dates_b);
        assert_eq!(result.len(), dates_a.len(), "Intersection of identical ranges should match their length.");
        assert_eq!(result, dates_a, "Intersection of identical ranges should yield the same dates.");
    }

    #[test]
    fn test_partially_overlapping_date_ranges() {
        let dates_a = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        ];
        let dates_b = vec![
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(),
        ];
        let result = intersect_date_ranges(&dates_a, &dates_b);
        let expected = vec![
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        ];
        assert_eq!(result, expected, "Intersection should yield only overlapping dates.");
    }

    #[test]
    fn test_subset_date_ranges() {
        let dates_a = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        ];
        let dates_b = vec![
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        ];
        let result = intersect_date_ranges(&dates_a, &dates_b);
        assert_eq!(result, dates_b, "Intersection should match the smaller range if it's a subset.");
    }

    #[test]
    fn test_disjoint_ranges_with_duplicates() {
        let dates_a = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), // Duplicate
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        ];
        let dates_b = vec![
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), // Duplicate
        ];
        let result = intersect_date_ranges(&dates_a, &dates_b);
        let expected = vec![NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()];
        assert_eq!(result, expected, "Intersection should ignore duplicates and yield correct results.");
    }
}
