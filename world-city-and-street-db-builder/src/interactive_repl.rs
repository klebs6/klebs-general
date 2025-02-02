// ---------------- [ File: src/interactive_repl.rs ]
crate::ix!();

/// (8) Start the REPL
pub fn run_interactive_repl(db_arc: Arc<Mutex<Database>>) -> Result<(), Box<dyn std::error::Error>> {
    // Lock DB once, build DataAccess, find all done regions, build region data.
    let db_guard = db_arc.lock().unwrap();
    let done_regions = load_done_regions(&db_guard);
    if done_regions.is_empty() {
        eprintln!("No ‘DONE’ regions found in DB! Possibly nothing to query.");
    }

    // If you want to pick a default region, do so. 
    // For example, pick the first region in done_regions or make a dummy if empty.
    let default_region = match done_regions.first() {
        Some(r) => *r,
        None => WorldRegion::default(),
    };

    let regions_map = build_all_region_data(&db_guard, &done_regions);

    // House-number ranges: either read from DB or a stub
    let ranges_stub = load_house_number_ranges_stub();

    // Create initial ReplState
    let st = ReplStateBuilder::default()
        .regions(regions_map)
        .current_region(default_region)
        .mode(AutocompleteMode::City)
        .fuzzy_matcher(Arc::new(SkimMatcherV2::default()))
        .db_access(DataAccess::with_db(db_arc.clone()))
        .house_number_ranges(ranges_stub)
        .build()
        .unwrap();

    drop(db_guard); // release the DB lock

    let shared_state = Arc::new(Mutex::new(st));
    let completer = MyCompleter {
        state: shared_state.clone(),
    };

    let mut rl = Editor::<MyCompleter, DefaultHistory>::new()?;
    rl.set_helper(Some(completer));

    // Optionally load command history from file
    let _ = rl.load_history("interactive_repl_history.txt");

    println!("Welcome to the multi-region REPL. Type 'help' for commands, 'exit' to quit.");

    loop {
        let prompt = {
            let s = shared_state.lock().unwrap();
            let abbr = s.current_region().abbreviation();
            format!("({}/{:?})> ", abbr, s.mode())
        };
        let line_res = rl.readline(&prompt);

        match line_res {
            Ok(line) => {
                let line = line.trim();
                rl.add_history_entry(line);

                if line.eq_ignore_ascii_case("exit") {
                    println!("Goodbye.");
                    break;
                }
                if line.eq_ignore_ascii_case("help") {
                    print_help();
                    continue;
                }

                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.is_empty() {
                    continue;
                }

                // region <ABBR>
                if tokens[0].eq_ignore_ascii_case("region") && tokens.len() >= 2 {
                    let abbr = tokens[1];
                    let mut st = shared_state.lock().unwrap();
                    // Attempt to find a region whose abbreviation matches:
                    let maybe_region = st.regions().keys().find(|r| {
                        r.abbreviation().eq_ignore_ascii_case(abbr)
                    }).clone();
                    if let Some(r) = maybe_region {
                        let r = r.clone();
                        st.set_current_region(r);
                        println!("Region changed to: {}", r.abbreviation());
                    } else {
                        println!("Unknown region abbreviation '{}'. Available:", abbr);
                        for r in st.regions().keys() {
                            println!("  {}", r.abbreviation());
                        }
                    }
                    continue;
                }

                // mode <city|street>
                if tokens[0].eq_ignore_ascii_case("mode") && tokens.len() >= 2 {
                    let mode_str = tokens[1];
                    let mut st = shared_state.lock().unwrap();
                    match mode_str.to_lowercase().as_str() {
                        "city" => {
                            st.set_mode(AutocompleteMode::City);
                            println!("Now auto-completing Cities.");
                        }
                        "street" => {
                            st.set_mode(AutocompleteMode::Street);
                            println!("Now auto-completing Streets.");
                        }
                        _ => println!("Unknown mode '{}'. Try 'city' or 'street'.", mode_str),
                    }
                    continue;
                }

                // validate <zip> <city> [<house_number>] <street...>
                if tokens[0].eq_ignore_ascii_case("validate") && tokens.len() >= 4 {
                    let mut st = shared_state.lock().unwrap();
                    handle_validate_command(line, &mut *st);
                    continue;
                }

                // byzip <zip> => list city/street sets
                if tokens[0].eq_ignore_ascii_case("byzip") && tokens.len() >= 2 {
                    let zip = tokens[1];
                    let st = shared_state.lock().unwrap();
                    handle_byzip_command(zip, &*st);
                    continue;
                }

                // range <street> [house_number]
                if tokens[0].eq_ignore_ascii_case("range") && tokens.len() >= 2 {
                    let mut st = shared_state.lock().unwrap();
                    handle_range_command(tokens[1..].to_vec(), &mut *st);
                    continue;
                }

                // Otherwise, interpret line as a city or street in the current region
                let st = shared_state.lock().unwrap();
                handle_typed_entry(&*st, line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D / EOF");
                break;
            }
            Err(e) => {
                println!("Readline error: {}", e);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history("interactive_repl_history.txt");
    Ok(())
}

/// Print help
fn print_help() {
    println!("Commands:");
    println!("  help                  => this help");
    println!("  exit                  => quit the REPL");
    println!("  region <abbr>         => switch to that region (must be in the 'DONE' set)");
    println!("  mode <city|street>    => toggle fuzzy completion mode");
    println!("  validate <zip> <city> [<house_num>] <street...>");
    println!("  byzip <zip>           => list city/street sets for that zip");
    println!("  range <street> [num]  => show house number ranges (and optionally check a house_num)");
    println!("Typing a recognized city or street shows DB info. \n");
}

/// (A) Attempt to interpret the line as either a city or a street in the current region.
/// If it matches exactly (case-insensitive) a known city or street, we show associated
/// info from the DB. If it doesn’t match, we print “(Doesn’t match known city/street)”
/// plus a fuzzy suggestion.
fn handle_typed_entry(st: &ReplState, typed: &str) {
    let typed_lc = typed.to_lowercase();

    // Get the RegionData for the current region
    let reg_data = match st.regions().get(st.current_region()) {
        Some(d) => d,
        None => {
            println!("No RegionData for region {:?}", st.current_region().abbreviation());
            return;
        }
    };

    // 1) Check if typed matches a known city
    if reg_data.cities().iter().any(|c| c.eq_ignore_ascii_case(&typed_lc)) {
        // Found a city => show associated postal codes, streets
        let city_obj = CityName::new(&typed_lc).unwrap_or_else(|_| {
            println!("Weird city parse error, but continuing…");
            CityName::new("unknown").unwrap()
        });
        println!("'{}' is recognized as a city in region {}.", typed, st.current_region().abbreviation());

        // Show postal codes
        let c2z_key = c2z_key(st.current_region(), &city_obj);
        if let Some(postals) = st.db_access().get_postal_code_set(&c2z_key) {
            println!("Postal codes for city '{}':", typed_lc);
            for pc in postals {
                println!("  {}", pc.code());
            }
        }

        // Show streets
        let c2s_key = c2s_key(st.current_region(), &city_obj);
        if let Some(streets) = st.db_access().get_street_set(&c2s_key) {
            println!("Streets in city '{}':", typed_lc);
            for s in streets {
                println!("  {}", s.name());
            }
        }
        return;
    }

    // 2) Check if typed matches a known street
    if reg_data.streets().iter().any(|s| s.eq_ignore_ascii_case(&typed_lc)) {
        println!("'{}' is recognized as a street in region {}.", typed, st.current_region().abbreviation());

        let street_obj = StreetName::new(&typed_lc).unwrap_or_else(|_| {
            println!("Strange parse error for street. Continuing…");
            StreetName::new("unknown").unwrap()
        });

        // Which cities is it in?
        let s2c_key = s2c_key(st.current_region(), &street_obj);
        if let Some(cities) = st.db_access().get_city_set(&s2c_key) {
            println!("Cities containing '{}':", typed_lc);
            for c in cities {
                println!("  {}", c.name());
            }
        }

        // Which ZIP(s)?
        let s2z_key = s2z_key(st.current_region(), &street_obj);
        if let Some(zips) = st.db_access().get_postal_code_set(&s2z_key) {
            println!("ZIP codes containing '{}':", typed_lc);
            for z in zips {
                println!("  {}", z.code());
            }
        }

        // House-number ranges?
        let abbr = st.current_region().abbreviation().to_string();
        let key = (abbr, typed_lc.clone());
        if let Some(ranges) = st.house_number_ranges().get(&key) {
            println!("House number ranges for '{}':", typed_lc);
            for (start,end) in ranges {
                println!("  {} - {}", start, end);
            }
        } else {
            println!("No known house-number range data for '{}'.", typed_lc);
        }
        return;
    }

    // If neither matched:
    println!("You typed: '{}', which doesn't match any known city or street in region {}.", typed, st.current_region().abbreviation());

    // Provide fuzzy suggestions for whichever mode? Or for both? Here we can do both:
    let all_cities = reg_data.cities();
    let all_streets = reg_data.streets();

    // e.g. if city is the likely intended mode, we can do fuzzy across cities. If street, do fuzzy across streets.
    let city_suggestions = best_fuzzy_matches(st.fuzzy_matcher(), typed, all_cities, 3);
    let street_suggestions = best_fuzzy_matches(st.fuzzy_matcher(), typed, all_streets, 3);

    if !city_suggestions.is_empty() {
        println!("Did you mean (city)?");
        for c in &city_suggestions {
            println!("  {}", c);
        }
    }
    if !street_suggestions.is_empty() {
        println!("Did you mean (street)?");
        for s in &street_suggestions {
            println!("  {}", s);
        }
    }
}

/// Utility: pick top `limit` fuzzy matches.
fn best_fuzzy_matches(
    matcher: &SkimMatcherV2,
    input: &str,
    candidates: &[String],
    limit: usize,
) -> Vec<String> {
    let mut scored = Vec::new();
    for c in candidates {
        if let Some(score) = matcher.fuzzy_match(c, input) {
            scored.push((c.clone(), score));
        }
    }
    // sort descending
    scored.sort_by_key(|(_c,score)| -(*score));
    scored.truncate(limit);
    scored.into_iter().map(|(c,_score)| c).collect()
}

/// (B) “byzip <zip>” => we show city set & street set from DB
fn handle_byzip_command(zip: &str, st: &ReplState) {
    let region = st.current_region();
    // city set
    if let Ok(pc_obj) = PostalCode::new(Country::USA, zip) {
        let z2c_key = z2c_key(&region, &pc_obj);
        if let Some(cities) = st.db_access().get_city_set(&z2c_key) {
            println!("Cities in ZIP '{}':", zip);
            for c in &cities {
                println!("  {}", c.name());
            }
        } else {
            println!("No city data for zip '{}'.", zip);
        }

        let s_key = s_key(&region, &pc_obj);
        if let Some(sts) = st.db_access().get_street_set(&s_key) {
            println!("Streets in ZIP '{}':", zip);
            for s in &sts {
                println!("  {}", s.name());
            }
        } else {
            println!("No street data for zip '{}'.", zip);
        }
    } else {
        println!("Invalid postal code '{}'.", zip);
    }
}

/// (C) “range <street> [house_number]”
fn handle_range_command(args: Vec<&str>, st: &mut ReplState) {
    // parse street from all but maybe the last if the last is a number
    // or the second argument is all but the first if the first is numeric. 
    // We'll do a simpler approach: if the last token is an integer, treat that as house_number; 
    // everything else is the street. 
    if args.is_empty() {
        println!("Usage: range <street> [house_number]");
        return;
    }
    let last_arg = args[args.len()-1];
    let maybe_num = last_arg.parse::<u32>();
    let (street_tokens, maybe_house_num) = match maybe_num {
        Ok(n) => (&args[..args.len()-1], Some(n)),
        Err(_) => (&args[..], None),
    };
    let street_lc = street_tokens.join(" ").to_lowercase();

    // find the region abbreviation
    let abbr = st.current_region().abbreviation().to_string();
    let key = (abbr, street_lc.clone());
    if let Some(ranges) = st.house_number_ranges().get(&key) {
        println!("Ranges for street '{}':", street_lc);
        for (start,end) in ranges {
            println!("  {} - {}", start, end);
        }
        if let Some(house_num) = maybe_house_num {
            // check if in any range
            let in_range = ranges.iter().any(|(lo, hi)| house_num >= *lo && house_num <= *hi);
            if in_range {
                println!("House number {} is within a known range for '{}'.", house_num, street_lc);
            } else {
                println!("House number {} is NOT within any known range for '{}'.", house_num, street_lc);
            }
        }
    } else {
        println!("No known range data for street '{}'.", street_lc);
    }
}

/// (D) “validate <zip> <city> [house_number] <street...>”
fn handle_validate_command(line: &str, st: &mut ReplState) {
    // minimal parse
    let tokens: Vec<&str> = line.split_whitespace().collect();
    //  0=validate
    //  1=postal
    //  2=city
    // Possibly 3=house_number
    // Then the rest is street
    if tokens.len() < 4 {
        println!("Usage: validate <zip> <city> [house_number] <street...>");
        return;
    }
    let postal = tokens[1];
    let city = tokens[2];

    // Next: see if tokens[3] is an integer => house_number?
    let mut house_num: Option<u32> = None;
    let mut street_tokens: Vec<&str> = vec![];
    if tokens.len() >= 4 {
        // parse token[3]
        let test = tokens[3].parse::<u32>();
        match test {
            Ok(n) => {
                // so tokens[4..] is the street
                house_num = Some(n);
                if tokens.len() >= 5 {
                    street_tokens = tokens[4..].to_vec();
                } else {
                    // no street
                    println!("Usage: validate <zip> <city> [house_number] <street...>");
                    return;
                }
            }
            Err(_) => {
                // tokens[3..] is the street
                street_tokens = tokens[3..].to_vec();
            }
        }
    }
    if street_tokens.is_empty() {
        println!("Usage: validate <zip> <city> [house_number] <street...>");
        return;
    }
    let street = street_tokens.join(" ");

    // Build the address
    let region = st.current_region();
    let pc_obj = match PostalCode::new(Country::USA, postal) {
        Ok(pc) => pc,
        Err(e) => {
            println!("Invalid postal code: {:?}", e);
            return;
        }
    };
    let city_obj = match CityName::new(city) {
        Ok(cy) => cy,
        Err(e) => {
            println!("Invalid city name: {:?}", e);
            return;
        }
    };
    let street_obj = match StreetName::new(&street) {
        Ok(stx) => stx,
        Err(e) => {
            println!("Invalid street name: {:?}", e);
            return;
        }
    };

    let wa = match WorldAddressBuilder::default()
        .region(*region)
        .postal_code(pc_obj)
        .city(city_obj)
        .street(street_obj)
        .build()
    {
        Ok(a) => a,
        Err(e) => {
            println!("Could not build WorldAddress: {:?}", e);
            return;
        }
    };

    // Now call .validate_with
    match wa.validate_with(st.db_access()) {
        Ok(_) => {
            // If we have house_num, check range
            if let Some(hn) = house_num {
                let ab = region.abbreviation().to_string();
                let st_lc = wa.street().name().to_string();
                let key = (ab, st_lc);
                if let Some(ranges) = st.house_number_ranges().get(&key) {
                    let in_range = ranges.iter().any(|(lo,hi)| hn >= *lo && hn <= *hi);
                    if in_range {
                        println!("Address is valid (including house number).");
                    } else {
                        println!("Address found, but house number {} is not in any known range!", hn);
                    }
                } else {
                    println!("Address found, but we have no range data for that street => can't confirm house number.");
                }
            } else {
                println!("Address is valid!");
            }
        }
        Err(e) => {
            match e {
                InvalidWorldAddress::CityNotFoundForPostalCodeInRegion { city, postal_code, .. } => {
                    println!("Invalid: city '{}' not in postal {}. Fuzzy suggestions:", city.name(), postal_code.code());
                    // Fuzzy city suggestions
                    let region_data = st.regions()[&region].clone();
                    let sugs = best_fuzzy_matches(st.fuzzy_matcher(), &city.name(), region_data.cities(), 5);
                    for s in sugs {
                        println!("  {}", s);
                    }
                }
                InvalidWorldAddress::StreetNotFoundForPostalCodeInRegion { street, postal_code, .. } => {
                    println!("Invalid: street '{}' not in postal {}. Fuzzy suggestions:", street.name(), postal_code.code());
                    let region_data = st.regions()[&region].clone();
                    let sugs = best_fuzzy_matches(st.fuzzy_matcher(), &street.name(), region_data.streets(), 5);
                    for s in sugs {
                        println!("  {}", s);
                    }
                }
                InvalidWorldAddress::StreetNotFoundForCityInRegion { street, city, .. } => {
                    println!("Invalid: street '{}' not found in city '{}'. Suggestions:", street.name(), city.name());
                    let region_data = st.regions()[&region].clone();
                    let sugs = best_fuzzy_matches(st.fuzzy_matcher(), &street.name(), region_data.streets(), 5);
                    for s in sugs {
                        println!("  {}", s);
                    }
                }
                other => {
                    println!("Validation error: {:?}", other);
                }
            }
        }
    }
}
