// ---------------- [ File: src/interactive_repl.rs ]
crate::ix!();

pub fn run_interactive_repl<I: StorageInterface>(db_arc: Arc<Mutex<I>>)
    -> Result<(), PagerError>
{
    // Force show cursor at start
    let _ = execute!(stdout(), Show);

    let db_guard = db_arc.lock().expect("DB lock poisoned");

    let done_regions = load_done_regions(&*db_guard);
    if done_regions.is_empty() {
        eprintln!("No ‘DONE’ regions found in DB! Possibly nothing to query.");
    }

    let default_region = done_regions
        .first()
        .copied()
        .unwrap_or_else(|| WorldRegion::default());
    let regions_map = build_all_region_data(&*db_guard, &done_regions);

    let mut house_number_ranges: HashMap<(String, String), Vec<HouseNumberRange>> = HashMap::new();
    for region in &done_regions {
        if let Some(rd) = regions_map.get(region) {
            for street_str in rd.streets() {
                let st_obj = match StreetName::new(street_str) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if let Ok(Some(rngs)) = db_guard.load_house_number_ranges(region, &st_obj) {
                    let key = (region.abbreviation().to_string(), street_str.clone());
                    house_number_ranges.insert(key, rngs);
                }
            }
        }
    }
    drop(db_guard);

    // Construct ReplState via its builder, *without* calling .default().
    let builder = ReplStateBuilder::default()
        .regions(regions_map)
        .current_region(default_region)
        .mode(AutocompleteMode::City)
        .fuzzy_matcher(Arc::new(SkimMatcherV2::default()))
        .db_access(Arc::new(DataAccess::with_db(db_arc.clone())))
        .house_number_ranges(house_number_ranges);

    let st = builder
        .build()
        .map_err(|_e| PagerError::Default { 
            msg: "Failed to build ReplState".to_string() 
        })?;

    let shared_state = Arc::new(Mutex::new(st));
    let completer = MyCompleter::new(shared_state.clone());
    let mut rl = Editor::<MyCompleter<I>, DefaultHistory>::new()?;
    rl.set_helper(Some(completer));
    let _ = rl.load_history("interactive_repl_history.txt");

    println!("Welcome to the multi-region REPL. Type 'help' for commands, 'exit' to quit.");
    loop {
        let prompt = {
            let s = shared_state.lock().map_err(|_e| PagerError::Default {
                msg: "Mutex lock poisoned".to_string(),
            })?;
            format!("({}/{:?})> ", s.current_region().abbreviation(), s.mode())
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

                if tokens[0].eq_ignore_ascii_case("region") && tokens.len() >= 2 {
                    let ab = tokens[1];
                    let mut st = shared_state.lock().map_err(|_e| PagerError::Default {
                        msg: "Mutex lock poisoned".to_string(),
                    })?;
                    let maybe_region = st
                        .regions()
                        .keys()
                        .find(|r| r.abbreviation().eq_ignore_ascii_case(ab))
                        .cloned();
                    if let Some(rr) = maybe_region {
                        st.set_current_region(rr.clone());
                        println!("Region changed to: {}", rr.abbreviation());
                    } else {
                        println!("Unknown region abbreviation '{}'.", ab);
                    }
                    continue;
                }

                if tokens[0].eq_ignore_ascii_case("mode") && tokens.len() >= 2 {
                    let mode_str = tokens[1];
                    let mut st = shared_state.lock().map_err(|_e| PagerError::Default {
                        msg: "Mutex lock poisoned".to_string(),
                    })?;
                    match mode_str.to_lowercase().as_str() {
                        "city" => {
                            st.set_mode(AutocompleteMode::City);
                            println!("Now auto-completing Cities.");
                        }
                        "street" => {
                            st.set_mode(AutocompleteMode::Street);
                            println!("Now auto-completing Streets.");
                        }
                        _ => {
                            println!("Unknown mode '{}'. Try 'city' or 'street'.", mode_str);
                        }
                    }
                    continue;
                }

                if tokens[0].eq_ignore_ascii_case("validate") {
                    let mut st = shared_state.lock().map_err(|_e| PagerError::Default {
                        msg: "Mutex lock poisoned".to_string(),
                    })?;
                    if let Err(e) = handle_validate_command(line, &mut *st) {
                        println!("Validate command error: {:?}", e);
                    }
                    continue;
                }

                if tokens[0].eq_ignore_ascii_case("byzip") && tokens.len() >= 2 {
                    let zip = tokens[1];
                    let st = shared_state.lock().map_err(|_e| PagerError::Default {
                        msg: "Mutex lock poisoned".to_string(),
                    })?;
                    handle_byzip_command(zip, &*st);
                    continue;
                }

                if tokens[0].eq_ignore_ascii_case("range") && tokens.len() >= 2 {
                    let mut st = shared_state.lock().map_err(|_e| PagerError::Default {
                        msg: "Mutex lock poisoned".to_string(),
                    })?;
                    handle_range_command(tokens[1..].to_vec(), &mut *st);
                    continue;
                }

                // Otherwise typed city or street
                let st = shared_state.lock().map_err(|_e| PagerError::Default {
                    msg: "Mutex lock poisoned".to_string(),
                })?;

                let typed_entry_lines = handle_typed_entry(&*st, line);
                info!("typed_entry_lines_len: {}", typed_entry_lines.len());
                print_or_page(&typed_entry_lines);
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

    let _ = rl.save_history("interactive_repl_history.txt");
    Ok(())
}
