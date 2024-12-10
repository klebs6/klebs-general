crate::ix!();

#[derive(Builder,Debug)]
#[builder(setter(into))]
pub struct CrateActivitySummary {
    date_interval_1d:              NaiveDate,
    date_interval_3d:              NaiveDate,
    date_interval_full_start:      NaiveDate,
    date_interval_full_end:        NaiveDate,
    total_downloads:               i64,
    avg_daily_downloads:           f64,
    avg_daily_downloads_per_crate: f64,
    median_daily_downloads:        i64,
    crates_analyzed:               usize,
    top_crates_1d:                 Vec<(String, i64)>,
    top_crates_3d:                 Vec<(String, i64)>,
}

impl fmt::Display for CrateActivitySummary {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        writeln!(f, "Crate Activity Summary:")?;
        writeln!(f, "  Full Data Range:             {} to {}", self.date_interval_full_start, self.date_interval_full_end)?;
        writeln!(f, "  Date Interval (Last 1 Day):  {}", self.date_interval_1d)?;
        writeln!(f, "  Date Interval (Last 3 Days): {}", self.date_interval_3d)?;

        writeln!(f, "  Total Downloads:                   {}", self.total_downloads)?;
        writeln!(f, "  Average Daily Downloads:           {:.2}", self.avg_daily_downloads)?;
        writeln!(f, "  Average Daily Downloads per Crate: {:.2}", self.avg_daily_downloads_per_crate)?;
        writeln!(f, "  Median Daily Downloads:            {}", self.median_daily_downloads)?;
        writeln!(f, "  Crates Analyzed:                   {}", self.crates_analyzed)?;

        writeln!(f, "\nTop Crates (Last 1 Day):")?;
        for (crate_name, downloads) in &self.top_crates_1d {
            writeln!(f, "  {:<30} {:>10} downloads", crate_name, downloads)?;
        }

        writeln!(f, "\nTop Crates (Last 3 Days):")?;
        for (crate_name, downloads) in &self.top_crates_3d {
            writeln!(f, "  {:<30} {:>10} downloads", crate_name, downloads)?;
        }

        Ok(())
    }
}

impl CrateActivitySummary {

    pub fn new(
        summaries: &[CrateUsageSummary],
        interval_downloads_1d: HashMap<String, i64>,
        interval_downloads_3d: HashMap<String, i64>,
        one_day_ago: NaiveDate,
        three_days_ago: NaiveDate,
    ) -> Self {

        // Compute the full date range
        let (full_start, full_end) = summaries
            .iter()
            .flat_map(|s| s.version_downloads())
            .map(|d| d.date())
            .minmax()
            .into_option()
            .unwrap_or((&one_day_ago, &one_day_ago));

        // Overall stats
        let total_downloads: i64 = summaries.iter().map(|s| s.total_downloads()).sum();
        let avg_daily_downloads: f64 = summaries.iter().map(|s| s.average_daily_downloads()).sum::<f64>();
        let avg_daily_downloads_per_crate = avg_daily_downloads / summaries.len() as f64;

        // Median daily downloads
        let mut daily_downloads: Vec<i64> = summaries.iter().map(|s| *s.total_downloads()).collect();
        daily_downloads.sort();
        let median_daily_downloads = if daily_downloads.is_empty() {
            0
        } else if daily_downloads.len() % 2 == 0 {
            let mid = daily_downloads.len() / 2;
            (daily_downloads[mid - 1] + daily_downloads[mid]) / 2
        } else {
            daily_downloads[daily_downloads.len() / 2]
        };

        // Top crates for the last 1 and 3 days
        let mut top_crates_1d: Vec<_> = interval_downloads_1d.into_iter().collect();
        let mut top_crates_3d: Vec<_> = interval_downloads_3d.into_iter().collect();

        top_crates_1d.sort_by_key(|&(_, downloads)| std::cmp::Reverse(downloads));
        top_crates_3d.sort_by_key(|&(_, downloads)| std::cmp::Reverse(downloads));

        CrateActivitySummary {
            date_interval_1d:         one_day_ago,
            date_interval_3d:         three_days_ago,
            date_interval_full_end:   *full_end,
            date_interval_full_start: *full_start,
            total_downloads,
            avg_daily_downloads,
            avg_daily_downloads_per_crate,
            median_daily_downloads,
            crates_analyzed: summaries.len(),
            top_crates_1d,
            top_crates_3d,
        }
    }
}
