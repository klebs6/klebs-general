crate::ix!();

#[derive(Builder,Debug)]
#[builder(setter(into))]
pub struct CrateActivitySummary {
    date_interval_1d:         NaiveDate,
    date_interval_3d:         NaiveDate,
    date_interval_full_start: NaiveDate,
    date_interval_full_end:   NaiveDate,

    total_downloads:               i64,
    avg_daily_downloads:           f64,
    avg_daily_downloads_per_crate: f64,
    median_daily_downloads:        i64,
    crates_analyzed:               usize,

    top_crates_1d: Vec<(String, i64)>,
    top_crates_3d: Vec<(String, i64)>,
    top_crates_7d: Vec<(String, i64)>,

    /// If true, we'll print each individual crate in a group
    expand_groups: bool,

    /// Minimum group size required to treat them as a “group”
    min_group_size: usize,
}

impl CrateActivitySummary {
    pub fn new(
        summaries: &[CrateUsageSummary],
        interval_downloads_1d: HashMap<String, i64>,
        interval_downloads_3d: HashMap<String, i64>,
        interval_downloads_7d: HashMap<String, i64>,
        one_day_ago: NaiveDate,
        three_days_ago: NaiveDate,
        seven_days_ago: NaiveDate,
        expand_groups: bool,
        min_group_size: usize,
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
        let avg_daily_downloads: f64 =
            summaries.iter().map(|s| s.average_daily_downloads()).sum::<f64>();
        let avg_daily_downloads_per_crate = if summaries.is_empty() {
            0.0
        } else {
            avg_daily_downloads / summaries.len() as f64
        };

        // Median daily downloads
        let mut daily_downloads: Vec<i64> =
            summaries.iter().map(|s| *s.total_downloads()).collect();
        daily_downloads.sort();
        let median_daily_downloads = if daily_downloads.is_empty() {
            0
        } else if daily_downloads.len() % 2 == 0 {
            let mid = daily_downloads.len() / 2;
            (daily_downloads[mid - 1] + daily_downloads[mid]) / 2
        } else {
            daily_downloads[daily_downloads.len() / 2]
        };

        // Convert the HashMaps into sorted vecs
        let mut top_crates_1d: Vec<_> = interval_downloads_1d.into_iter().collect();
        let mut top_crates_3d: Vec<_> = interval_downloads_3d.into_iter().collect();
        let mut top_crates_7d: Vec<_> = interval_downloads_7d.into_iter().collect();

        top_crates_1d.sort_by_key(|&(_, downloads)| std::cmp::Reverse(downloads));
        top_crates_3d.sort_by_key(|&(_, downloads)| std::cmp::Reverse(downloads));
        top_crates_7d.sort_by_key(|&(_, downloads)| std::cmp::Reverse(downloads));

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
            top_crates_7d,

            expand_groups,
            min_group_size,
        }
    }
}

impl fmt::Display for CrateActivitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::collections::HashMap;
        use std::fmt::Write as _;

        // Helper: extract the group prefix from a crate name
        // e.g. "surgefx-allpass" => "surgefx", "workspacer-3p" => "workspacer"
        fn extract_prefix(crate_name: &str) -> String {
            if let Some(idx) = crate_name.find('-') {
                crate_name[..idx].to_string()
            } else {
                crate_name.to_string()
            }
        }

        // We'll store stats in a struct for convenience
        #[derive(Clone)]
        struct GroupStats {
            group_label:   String,
            max_downloads: i64,
            sum_downloads: i64,
            avg_downloads: f64,
            n_crates:      usize,
            members:       Vec<(String, i64)>,
        }

        #[tracing::instrument(level = "debug", skip(crates))]
        fn group_crates_compact(
            crates: &[(String, i64)],
            min_group_size: usize,
        ) -> (Vec<GroupStats>, Vec<(String, i64)>) {
            let mut group_map: HashMap<String, Vec<(String, i64)>> = HashMap::new();

            // Collect members by prefix
            for (crate_name, downloads) in crates {
                let prefix = extract_prefix(crate_name);
                group_map
                    .entry(prefix)
                    .or_default()
                    .push((crate_name.clone(), *downloads));
            }

            let mut groups = Vec::new();
            let mut single_items = Vec::new();

            for (prefix, members) in group_map {
                if members.len() >= min_group_size {
                    // We form a group
                    let sum_downloads: i64 = members.iter().map(|m| m.1).sum();
                    let max_downloads: i64 = members.iter().map(|m| m.1).max().unwrap_or(0);
                    let n_crates = members.len();
                    let avg_downloads = if n_crates > 0 {
                        sum_downloads as f64 / n_crates as f64
                    } else {
                        0.0
                    };
                    // e.g. "surgefx-" or "workspacer-"
                    let group_label = format!("{}-*", prefix);

                    // Sort the group's members by descending downloads
                    let mut sorted_members = members.clone();
                    sorted_members.sort_by(|a, b| {
                        b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
                    });

                    groups.push(GroupStats {
                        group_label,
                        max_downloads,
                        sum_downloads,
                        avg_downloads,
                        n_crates,
                        members: sorted_members,
                    });
                } else {
                    // If group < min_group_size, treat each crate individually
                    for (crate_name, downloads) in members {
                        single_items.push((crate_name, downloads));
                    }
                }
            }

            // Sort groups by descending max_downloads, then by descending sum, then alpha
            groups.sort_by(|a, b| {
                b.max_downloads
                    .cmp(&a.max_downloads)
                    .then_with(|| b.sum_downloads.cmp(&a.sum_downloads))
                    .then_with(|| a.group_label.cmp(&b.group_label))
            });

            // Then sort single items (descending)
            single_items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

            (groups, single_items)
        }

        #[tracing::instrument(level = "debug", skip(f, crates))]
        fn display_grouped_crates_compact(
            f: &mut fmt::Formatter<'_>,
            heading: &str,
            crates: &[(String, i64)],
            min_group_size: usize,
            expand_groups: bool,
        ) -> fmt::Result {
            writeln!(f, "\n{}", heading)?;

            let (groups, single_items) = group_crates_compact(crates, min_group_size);

            // The total download count we show at the top line
            // is the sum of each group's max_downloads plus the sum of each single item.
            // Because the user wants to avoid "double-counting" the same prefix family.
            let mut total_for_display = 0i64;
            for g in &groups {
                total_for_display += g.max_downloads;
            }
            let single_total: i64 = single_items.iter().map(|x| x.1).sum();
            total_for_display += single_total;

            let group_coverage: usize = groups.iter().map(|g| g.n_crates).sum();
            let overall_count = group_coverage + single_items.len();

            writeln!(
                f,
                "  {} distinct prefix group(s) covering {} crates, total {} downloads (by max+singles)",
                groups.len(),
                overall_count,
                total_for_display
            )?;

            // Display each group
            for g in &groups {
                // example line:
                //   "workspacer-*   max=87  avg=27.76  sum=1721  n_crates=xx"
                writeln!(
                    f,
                    "  {:<24} max={:<5} avg={:>6.2} sum={:<6} n_crates={}",
                    g.group_label,
                    g.max_downloads,
                    g.avg_downloads,
                    g.sum_downloads,
                    g.n_crates
                )?;
                // If expand_groups is true, show each member
                if expand_groups {
                    for (crate_name, dl) in &g.members {
                        writeln!(f, "    {:<24} {:>5} downloads", crate_name, dl)?;
                    }
                }
            }

            // Finally, display single items (which didn't meet min_group_size)
            for (crate_name, downloads) in &single_items {
                writeln!(f, "  {:<24} {} downloads", crate_name, downloads)?;
            }

            Ok(())
        }

        // 1) Print the main summary lines
        writeln!(f, "Crate Activity Summary:")?;
        writeln!(f, "  Full Data Range:             {} to {}", 
                 self.date_interval_full_start, self.date_interval_full_end)?;
        writeln!(f, "  Date Interval (Last 1 Day):  {}", self.date_interval_1d)?;
        writeln!(f, "  Date Interval (Last 3 Days): {}", self.date_interval_3d)?;

        writeln!(f, "  Total Downloads:                   {}", self.total_downloads)?;
        writeln!(f, "  Average Daily Downloads:           {:.2}", self.avg_daily_downloads)?;
        writeln!(f, "  Average Daily Downloads per Crate: {:.2}", self.avg_daily_downloads_per_crate)?;
        writeln!(f, "  Median Daily Downloads:            {}", self.median_daily_downloads)?;
        writeln!(f, "  Crates Analyzed:                   {}", self.crates_analyzed)?;

        // 2) Group + display for each interval
        display_grouped_crates_compact(
            f,
            "Top Crates (Last 1 Day):",
            &self.top_crates_1d,
            self.min_group_size,
            self.expand_groups,
        )?;
        display_grouped_crates_compact(
            f,
            "Top Crates (Last 3 Days):",
            &self.top_crates_3d,
            self.min_group_size,
            self.expand_groups,
        )?;
        display_grouped_crates_compact(
            f,
            "Top Crates (Last 7 Days):",
            &self.top_crates_7d,
            self.min_group_size,
            self.expand_groups,
        )?;

        Ok(())
    }
}
