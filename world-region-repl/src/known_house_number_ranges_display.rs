// ---------------- [ File: src/known_house_number_ranges_display.rs ]
crate::ix!();

pub fn known_house_number_ranges_display(
    ranges:     &[HouseNumberRange], 
) -> Vec<String> {

    let mut result = Vec::new();

    let condensed_lines = compress_house_ranges(&ranges.iter()
        .map(|r| (*r.start(), *r.end()))
        .collect::<Vec<_>>());

    for line in condensed_lines {
        result.push(format!("  {}", line));
    }

    result
}
