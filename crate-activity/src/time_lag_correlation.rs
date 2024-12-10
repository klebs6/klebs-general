crate::ix!();

pub fn compute_time_lag_correlations(
    summaries: &[CrateUsageSummary],
    max_lag: i32
) -> Vec<(String, String, i32, f64)> {
    let mut results = Vec::new();
    let crate_data: HashMap<String, Vec<i64>> = summaries.iter().map(|s| {
        (s.crate_name().clone(), s.version_downloads().iter().map(|d| *d.downloads()).collect())
    }).collect();

    let crate_names: Vec<_> = crate_data.keys().cloned().collect();

    for i in 0..crate_names.len() {
        for j in (i+1)..crate_names.len() {
            let name_a = &crate_names[i];
            let name_b = &crate_names[j];
            let series_a = &crate_data[name_a];
            let series_b = &crate_data[name_b];

            let mut best_corr: f64 = 0.0;
            let mut best_lag = 0;
            let mut found_any = false;

            for lag in -max_lag..=max_lag {
                if let Some((xs, ys)) = align_for_lag(series_a, series_b, lag) {
                    let corr: f64 = pearson_correlation_i64(&xs, &ys);

                    if !found_any {
                        best_corr = corr;
                        best_lag = lag;
                        found_any = true;
                    } else {
                        let current_abs = corr.abs();
                        let best_abs = best_corr.abs();
                        if current_abs > best_abs {
                            // Strictly better correlation
                            best_corr = corr;
                            best_lag = lag;
                        } else if (current_abs - best_abs).abs() < 1e-12 {
                            // Tie in absolute correlation
                            let current_distance = lag.abs();
                            let best_distance = best_lag.abs();
                            if current_distance < best_distance {
                                // Closer to zero wins
                                best_corr = corr;
                                best_lag = lag;
                            } else if current_distance == best_distance {
                                // Still tied: pick negative lag if available
                                if lag < best_lag {
                                    best_corr = corr;
                                    best_lag = lag;
                                }
                            }
                        }
                    }
                }
            }

            if !found_any {
                // No valid alignment
                results.push((name_a.clone(), name_b.clone(), 0, 0.0));
            } else {
                results.push((name_a.clone(), name_b.clone(), best_lag, best_corr));
            }
        }
    }

    results
}


/// Align two time series for a given lag.
/// If lag > 0, we shift B forward: compare A[t] with B[t-lag]
/// If lag < 0, we shift A forward: compare A[t-lag] with B[t]
fn align_for_lag(
    a: &[i64],
    b: &[i64],
    lag: i32
) -> Option<(Vec<i64>, Vec<i64>)> {
    let n = a.len();
    if n == 0 || b.len() != n {
        return None;
    }

    if lag == 0 {
        // No lead/lag
        return Some((a.to_vec(), b.to_vec()));
    } else if lag > 0 {
        // lag > 0 means A leads B by lag days.
        // A(t) = B(t+lag)
        // Align A[lag..] with B[..n-lag]
        let shift = lag as usize;
        if shift >= n {
            return None;
        }
        let a_slice = &a[shift..];
        let b_slice = &b[..(n - shift)];
        if a_slice.len() == b_slice.len() && !a_slice.is_empty() {
            Some((a_slice.to_vec(), b_slice.to_vec()))
        } else {
            None
        }
    } else {
        // lag < 0 means B leads A by |lag| days.
        // B(t) = A(t+|lag|)
        // Align A[..n-|lag|] with B[|lag|..]
        let shift = (-lag) as usize;
        if shift >= n {
            return None;
        }
        let a_slice = &a[..(n - shift)];
        let b_slice = &b[shift..];
        if a_slice.len() == b_slice.len() && !a_slice.is_empty() {
            Some((a_slice.to_vec(), b_slice.to_vec()))
        } else {
            None
        }
    }
}

fn pearson_correlation_i64(x: &[i64], y: &[i64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }
    pearson_correlation(x, y)
}

pub fn display_time_lag_correlations(results: &[(String, String, i32, f64)]) {
    println!("----------------[time-lag-correlations]----------------");
    // Sort by absolute correlation descending
    let mut sorted = results.to_vec();
    sorted.sort_by(|a,b| b.3.abs().partial_cmp(&a.3.abs()).unwrap());
    for (a, b, lag, corr) in sorted.iter().take(10) {
        println!("{} - {}: best lag={} correlation={:.3}", a, b, lag, corr);
    }
    println!("");
}


#[cfg(test)]
mod time_lag_correlations_tests {
    use super::*;

    fn make_summary(crate_name: &str, downloads: &[i64]) -> CrateUsageSummary {
        // Create VersionDownload mocks
        let mut version_downloads = Vec::new();
        for (i, &d) in downloads.iter().enumerate() {
            let date = chrono::NaiveDate::from_ymd_opt(2024,1,(i+1) as u32).unwrap();
            version_downloads.push(VersionDownloadBuilder::default()
                .version(1_i64)
                .downloads(d)
                .date(date)
                .build().unwrap());
        }

        let total_downloads: i64 = downloads.iter().sum();
        let avg_daily = total_downloads as f64 / downloads.len() as f64;
        let peak = *downloads.iter().max().unwrap();

        CrateUsageSummaryBuilder::default()
            .crate_name(crate_name.to_string())
            .total_downloads(total_downloads)
            .average_daily_downloads(avg_daily)
            .peak_daily_downloads(peak)
            .download_trend(Some(DownloadTrend::Stable))
            .version_downloads(version_downloads)
            .build()
            .unwrap()
    }

    #[test]
    fn test_no_data() {
        let summaries: Vec<CrateUsageSummary> = Vec::new();
        let results = compute_time_lag_correlations(&summaries, 7);
        assert!(results.is_empty(), "No crates means no results.");
    }

    #[test]
    fn test_single_crate() {
        // Single crate => no pairs
        let s = make_summary("crateA", &[10,20,30]);
        let summaries = vec![s];
        let results = compute_time_lag_correlations(&summaries, 7);
        assert!(results.is_empty(), "One crate means no pairs.");
    }

    #[test]
    fn test_two_crates_no_lag() {
        // Perfect correlation without lag
        let a = make_summary("A", &[1,2,3,4,5]);
        let b = make_summary("B", &[1,2,3,4,5]);
        let summaries = vec![a,b];
        let results = compute_time_lag_correlations(&summaries, 2);
        assert_eq!(results.len(),1);
        let (ca, cb, lag, corr) = &results[0];
        assert!(corr.abs() > 0.999, "Perfect correlation");
        assert_eq!(*lag, 0, "Best lag is zero.");
        let mut crates = vec![ca.as_str(), cb.as_str()];
        crates.sort();
        assert_eq!(crates, vec!["A","B"]);
    }

    #[test]
    fn test_no_correlation_any_lag() {
        // A: 1,1,1,1,1
        // B: 10,9,5,2,0 no pattern related to A
        let a = make_summary("A", &[1,1,1,1,1]);
        let b = make_summary("B", &[10,9,5,2,0]);
        let summaries = vec![a,b];
        let results = compute_time_lag_correlations(&summaries, 3);
        let (_,_, lag, corr) = &results[0];
        // No correlation at all, best corr might remain near 0 and best_lag=0 by default.
        assert!(*corr < 0.5, "No significant correlation");
        assert_eq!(*lag, 0, "No correlation means no particular lag stands out.");
    }

    #[test]
    fn test_tie_same_abs_correlation_prefer_closer_to_zero() {
        // Construct data where lag=0 and lag=2 both give same perfect correlation.
        let a = dummy_summary("A", &[1,2,3,4,5]);
        let b = dummy_summary("B", &[1,2,3,4,5]); // Identical
        let results = compute_time_lag_correlations(&[a,b], 2);
        assert_eq!(results.len(),1);
        let (_,_, lag, corr) = &results[0];
        // lag=0 and lag=2 might both be perfect correlation
        // According to rules, tie means choose closer to zero => lag=0
        assert!((corr.abs()-1.0).abs()<1e-12);
        assert_eq!(*lag, 0, "Tie broken by choosing lag closer to zero");
    }

    fn dummy_summary(name: &str, values: &[i64]) -> CrateUsageSummary {
        let mut version_downloads = Vec::new();
        for (i, &d) in values.iter().enumerate() {
            let date = chrono::NaiveDate::from_ymd_opt(2024,1,(i+1) as u32).unwrap();
            version_downloads.push(VersionDownloadBuilder::default()
                .version(1_i64)
                .downloads(d)
                .date(date)
                .build().unwrap());
        }

        let total_downloads: i64 = values.iter().sum();
        let avg_daily = total_downloads as f64 / values.len() as f64;
        let peak = *values.iter().max().unwrap();

        CrateUsageSummaryBuilder::default()
            .crate_name(name.to_string())
            .total_downloads(total_downloads)
            .average_daily_downloads(avg_daily)
            .peak_daily_downloads(peak)
            .download_trend(Some(DownloadTrend::Stable))
            .version_downloads(version_downloads)
            .build()
            .unwrap()
    }

    #[test]
    fn test_single_best_lag() {
        let a = dummy_summary("A", &[10,20,30,40,50]);
        let b = dummy_summary("B", &[20,30,40,50,100]);

        let results = compute_time_lag_correlations(&[a,b], 2);
        assert_eq!(results.len(), 1);
        let (_,_, lag, corr) = &results[0];
        assert!((corr.abs()-1.0).abs() < 1e-12, "Expect perfect correlation at best lag");
        assert_eq!(*lag, 1, "Should choose lag=1 for maximum correlation");
    }

    #[test]
    fn test_tie_equal_distances_pick_negative() {
        // A=[10,20,30], B=[20,30,41]
        // ±1 lag yield perfect correlation of 1.0, lag=0 <1.0 correlation.
        // Tie at ±1 chooses negative lag.

        let a = dummy_summary("A", &[10,20,30]);
        let b = dummy_summary("B", &[20,30,41]); 
        let results = compute_time_lag_correlations(&[a,b], 1);
        let (_,_, lag, corr) = &results[0];
        assert!((corr.abs()-1.0).abs() < 1e-12, "Expect perfect correlation at ±1");
        assert_eq!(*lag, -1, "Tie between ±1 broken by choosing negative lag");
    }
}
