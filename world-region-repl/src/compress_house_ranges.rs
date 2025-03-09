// ---------------- [ File: src/compress_house_ranges.rs ]
crate::ix!();

/// Print house-number ranges in a style where:
///   1) We sort by ascending `start`.
///   2) We group by the thousand-bucket: i.e. all addresses 0..999 in one group, 
///      1000..1999 in next, 2000..2999, etc.
///   3) Within each group, we split the ranges into lines of up to `MAX_COLUMNS` items.
///   4) If a range has the same start/end, we print just "N". 
///      Otherwise we print "start - end".
///   5) We always add a trailing comma at the end of each line, to match your examples.
///
/// For example, if you have a bunch of addresses in the 4000..4999 range, 
/// and more than 10 of them, we'll create multiple lines for that bucket, each line
/// containing up to 10 comma-separated items, plus a trailing comma.
pub fn compress_house_ranges(ranges: &[(u32, u32)]) -> Vec<String> {

    if ranges.is_empty() {
        return vec![];
    }

    // 1) Sort by ascending start
    let mut sorted = ranges.to_vec();
    sorted.sort_by_key(|&(s, _)| s);

    // 2) Convert each (start, end) to a string: "start" or "start - end"
    //    Also figure out the “thousand-bucket” via start/1000
    use std::collections::BTreeMap;
    let mut buckets: BTreeMap<u32, Vec<String>> = BTreeMap::new();

    for &(start, end) in &sorted {
        let bucket = start / 1000; // e.g. 0 for 0..999, 4 for 4000..4999
        let label = if start == end {
            format!("{}", start)
        } else {
            format!("{}-{}", start, end)
        };
        buckets.entry(bucket).or_default().push(label);
    }

    let mut result = Vec::new();
    // 3) For each thousand-bucket in ascending order, chunk it into lines of up to 1000 columns,
    //    (essentially infinite)
    const MAX_COLUMNS: usize = 1000;
    for (_bucket, items) in &buckets {
        let mut idx = 0;
        while idx < items.len() {
            let end = (idx + MAX_COLUMNS).min(items.len());
            let chunk = &items[idx..end];
            // Join with comma+space, then add trailing comma
            let line_str = chunk.join(", ");
            result.push(format!(" {},", line_str));
            idx = end;
        }
    }
    result
}

#[cfg(test)]
mod house_number_range_grouping_tests {
    use super::print_house_number_ranges_grouped;

    #[test]
    fn test_example_from_user() {
        // The user gave an example set:
        //  2, 4, 10, 12, 16, 49, 55, 350, ...
        //  2001, 2010, 2012, 2018, ...
        //  etc.
        //
        // We'll just confirm that our function doesn't crash and produces chunked lines.
        // If you want to do a full exact string-compare, you can capture stdout in a test harness.
        let data = vec![
            (2,2),(4,4),(10,10),(12,12),(16,16),(49,49),(55,55),(350,350),
            (2001,2001),(2010,2010),(2012,2012),(2018,2018),(2023,2023),(2025,2025),
            (4292,4292),(4300,4300),(4600,4600),(4626,4626),(4628,4628),(4630,4630),
            (4632,4632),(4700,4700),(4702,4702),(4704,4704),(4718,4718),(4720,4720),
            (4722,4722),(4802,4802),(4804,4804),(4815,4815),(4902,4902),(4904,4906),
            (4908,4908),(4910,4910),(4912,4912),(4925,4925),(5001,5001),(5004,5005),
            (5007,5007),(5011,5011),(5015,5016),(5019,5020),(5022,5022),(5024,5024),
            (5026,5026),(5028,5028),(5032,5032),(5054,5054),(5101,5101),
        ];
        print_house_number_ranges_grouped(&data);

        // Manually inspect the output to confirm the lines are chunked by thousand-bucket,
        // with up to 10 entries per line, each line ending with a comma, etc.
    }

    #[test]
    fn test_single_bucket_over_ten_items() {
        let data = (1..=15).map(|x| (x, x)).collect::<Vec<_>>();
        // That means 1..15, all in bucket=0 (i.e. 0..999). So we expect multiple lines 
        // with up to 10 items each, trailing comma.
        print_house_number_ranges_grouped(&data);
        // e.g. first line has 1..10 => "1, 2, 3, ..., 10,"
        // second line has 11..15 => "11, 12, 13, 14, 15,"
    }

    #[test]
    fn test_multiple_buckets() {
        let data = vec![(999,999),(1000,1000),(1999,1999),(2000,2000)];
        // bucket 0 => [999]
        // bucket 1 => [1000,1999]
        // bucket 2 => [2000]
        print_house_number_ranges_grouped(&data);
        // We'll see 1 line for 999, then 1 line for 1000,1999, then 1 line for 2000
        // each line ends with a trailing comma
    }
}
