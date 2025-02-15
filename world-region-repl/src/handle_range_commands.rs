// ---------------- [ File: src/handle_range_commands.rs ]
crate::ix!();

pub fn handle_range_command<I:StorageInterface>(args: Vec<&str>, st: &mut ReplState<I>) {
    let mut lines = Vec::new();

    if args.is_empty() {
        lines.push("Usage: range <street> [house_number]".to_string());
        print_or_page(&lines);
        return;
    }

    let last_arg = args[args.len() - 1];
    let maybe_num = last_arg.parse::<u32>().ok();
    let street_tokens = if maybe_num.is_some() {
        &args[..args.len() - 1]
    } else {
        &args[..]
    };
    let street_lc = street_tokens.join(" ").to_lowercase();

    let street_obj = match StreetName::new(&street_lc) {
        Ok(s) => s,
        Err(e) => {
            lines.push(format!("Invalid street name: {:?}", e));
            print_or_page(&lines);
            return;
        }
    };

    let region = st.current_region();

    match st.db_access().load_house_number_ranges(region, &street_obj) {
        Ok(maybe_ranges) => {
            match maybe_ranges {
                Some(ranges) => {
                    lines.push(format!(
                            "Known house number sub-ranges for '{}' in {}:",
                            street_obj.name(),
                            region.abbreviation()
                    ));

                    // Convert the HouseNumberRanges to lines for printing:
                    let display = known_house_number_ranges_display(&ranges);
                    lines.extend(display);

                    // If user typed a house number, check it:
                    if let Some(hn) = maybe_num {
                        let in_any = ranges.iter().any(|r| r.contains(hn));
                        if in_any {
                            lines.push(format!(
                                    "House number {} IS within a known sub-range.",
                                    hn
                            ));
                        } else {
                            lines.push(format!(
                                    "House number {} is NOT within any known sub-range.",
                                    hn
                            ));
                        }
                    }
                }
                None => {
                    lines.push(format!(
                            "No sub-range data stored for street '{}'.",
                            street_obj.name()
                    ));
                }
            }
        }
        Err(e) => {
            lines.push(format!("Error loading house number ranges: {:?}", e));
        }
    }

    // Finally: page or print
    print_or_page(&lines);
}
