crate::ix!();

pub fn align_and_normalize_data(
    version_downloads: &[VersionDownload],
    full_date_range:   &[NaiveDate],

) -> Vec<i64> {

    let downloads_map: HashMap<NaiveDate, i64> = version_downloads
        .iter()
        .map(|d| (*d.date(), *d.downloads())) // Dereference both the date and the downloads
        .collect();

    full_date_range
        .iter()
        .map(|date| *downloads_map.get(date).unwrap_or(&0)) // Fill missing dates with 0
        .collect()
}

pub fn debug_alignment(
    crate_a: &str,
    crate_b: &str,
    aligned_a: &[i64],
    aligned_b: &[i64],
) {
    println!("Aligned data for {} and {}:", crate_a, crate_b);
    println!("  Aligned A: {:?}", aligned_a);
    println!("  Aligned B: {:?}", aligned_b);
}

#[cfg(test)]
mod alignment_and_normalization_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_empty_input() {
        let version_downloads = [];
        let full_date_range = [];
        let result = align_and_normalize_data(&version_downloads, &full_date_range);
        assert_eq!(result, Vec::<i64>::new(), "Empty input should produce empty output.");
    }

    #[test]
    fn test_full_date_range_matches_data() {
        let version_downloads = [
            VersionDownloadBuilder::default()
                .version(1_i64)
                .downloads(100_i64)
                .date(NaiveDate::from_ymd_opt(2024, 12, 1).unwrap())
                .build()
                .unwrap(),
            VersionDownloadBuilder::default()
                .version(1_i64)
                .downloads(200_i64)
                .date(NaiveDate::from_ymd_opt(2024, 12, 2).unwrap())
                .build()
                .unwrap(),
        ];
        let full_date_range = vec![
            NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
        ];
        let result = align_and_normalize_data(&version_downloads, &full_date_range);
        assert_eq!(result, vec![100, 200], "All dates should align correctly.");
    }

    #[test]
    fn test_full_date_range_extends_beyond_data() {
        let version_downloads = [
            VersionDownloadBuilder::default()
                .version(1_i64)
                .downloads(200_i64)
                .date(NaiveDate::from_ymd_opt(2024, 12, 2).unwrap())
                .build()
                .unwrap(),
        ];
        let full_date_range = vec![
            NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 3).unwrap(),
        ];
        let result = align_and_normalize_data(&version_downloads, &full_date_range);
        assert_eq!(
            result,
            vec![0, 200, 0],
            "Missing dates should be filled with 0."
        );
    }

    #[test]
    fn test_full_date_range_subset_of_data() {
        let version_downloads = [
            VersionDownloadBuilder::default()
                .version(1_i64)
                .downloads(100_i64)
                .date(NaiveDate::from_ymd_opt(2024, 12, 1).unwrap())
                .build()
                .unwrap(),
            VersionDownloadBuilder::default()
                .version(1_i64)
                .downloads(200_i64)
                .date(NaiveDate::from_ymd_opt(2024, 12, 2).unwrap())
                .build()
                .unwrap(),
            VersionDownloadBuilder::default()
                .version(1_i64)
                .downloads(300_i64)
                .date(NaiveDate::from_ymd_opt(2024, 12, 3).unwrap())
                .build()
                .unwrap(),
        ];
        let full_date_range = vec![
            NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 3).unwrap(),
        ];
        let result = align_and_normalize_data(&version_downloads, &full_date_range);
        assert_eq!(
            result,
            vec![200, 300],
            "Only values within the full_date_range should be included."
        );
    }

    #[test]
    fn test_duplicate_dates_in_input() {
        let version_downloads = [
            VersionDownloadBuilder::default()
                .version(1_i64)
                .downloads(100_i64)
                .date(NaiveDate::from_ymd_opt(2024, 12, 2).unwrap())
                .build()
                .unwrap(),
            VersionDownloadBuilder::default()
                .version(1_i64)
                .downloads(200_i64)
                .date(NaiveDate::from_ymd_opt(2024, 12, 2).unwrap())
                .build()
                .unwrap(), // Duplicate date
        ];
        let full_date_range = vec![
            NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
        ];
        let result = align_and_normalize_data(&version_downloads, &full_date_range);
        assert_eq!(
            result,
            vec![200],
            "The most recent value for a duplicate date should be used."
        );
    }
}
