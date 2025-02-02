// ---------------- [ File: src/run_repl.rs ]
crate::ix!();

/// Runs a simple REPL that allows the user to:
///   - type `cities` to see all city names (for demonstration)
///   - type `streets` to see all street names
///   - type `autocomplete city <prefix>` to filter city names
///   - type `autocomplete street <prefix>` to filter street names
///   - type `validate <region_abbr> <postal_code> <city> <street>` to test if an address is valid
///   - type `exit` to quit
///
/// The code is intentionally simplistic to illustrate how you might
/// utilize the database for queries or lookups.
pub fn run_repl(db: Arc<Mutex<Database>>) -> Result<(), Box<dyn std::error::Error>> {
    // Build a DataAccess
    let da = DataAccess::with_db(db.clone());

    // Let’s pick a single region to demonstrate. In a real tool,
    // you might let the user switch regions or pass an argument.
    let default_region = WorldRegion::from(USRegion::UnitedState(
        UnitedState::Maryland,
    ));

    // Preload city+street lists for the region. If you want DC/VA, you can do similarly
    let all_cities = {
        let guard = db.lock().map_err(|_| "Lock poisoned")?;
        load_all_cities_for_region(&guard, &default_region)
    };

    let all_streets = {
        let guard = db.lock().map_err(|_| "Lock poisoned")?;
        load_all_streets_for_region(&guard, &default_region)
    };

    println!("Entering simple REPL. Type 'help' for commands, or 'exit' to quit.");
    let stdin = std::io::stdin();

    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                // EOF
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.eq_ignore_ascii_case("exit") {
                    println!("Goodbye.");
                    break;
                }
                if line.eq_ignore_ascii_case("help") {
                    println!("Commands:");
                    println!("  cities               => List all known city names for region (MD in this example)");
                    println!("  streets              => List all known street names for region (MD in this example)");
                    println!("  autocomplete city <prefix>");
                    println!("  autocomplete street <prefix>");
                    println!("  validate <region_abbr> <postal_code> <city> <street>");
                    println!("  exit");
                    continue;
                }
                if line.eq_ignore_ascii_case("cities") {
                    println!("All city names in region {}:", default_region.abbreviation());
                    for c in &all_cities {
                        println!("  {}", c);
                    }
                    continue;
                }
                if line.eq_ignore_ascii_case("streets") {
                    println!("All street names in region {}:", default_region.abbreviation());
                    for s in &all_streets {
                        println!("  {}", s);
                    }
                    continue;
                }

                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.is_empty() {
                    continue;
                }

                // parse commands
                if tokens[0].eq_ignore_ascii_case("autocomplete") && tokens.len() >= 3 {
                    // e.g. "autocomplete city Bal"
                    let mode = tokens[1];
                    let prefix = tokens[2..].join(" ");
                    match mode.to_lowercase().as_str() {
                        "city" => {
                            let filtered: Vec<_> = all_cities
                                .iter()
                                .filter(|c| c.starts_with(&prefix.to_lowercase()))
                                .collect();
                            if filtered.is_empty() {
                                println!("No city match for prefix '{}'", prefix);
                            } else {
                                println!("Cities matching '{}':", prefix);
                                for c in filtered {
                                    println!("  {}", c);
                                }
                            }
                        }
                        "street" => {
                            let filtered: Vec<_> = all_streets
                                .iter()
                                .filter(|s| s.starts_with(&prefix.to_lowercase()))
                                .collect();
                            if filtered.is_empty() {
                                println!("No street match for prefix '{}'", prefix);
                            } else {
                                println!("Streets matching '{}':", prefix);
                                for s in filtered {
                                    println!("  {}", s);
                                }
                            }
                        }
                        other => {
                            println!("Unrecognized autocomplete mode: '{}'", other);
                        }
                    }
                    continue;
                }

                if tokens[0].eq_ignore_ascii_case("validate") && tokens.len() >= 5 {
                    // e.g. "validate US 21201 baltimore north avenue"
                    let region_abbr = tokens[1];
                    let postal = tokens[2];
                    let city = tokens[3];
                    let street = tokens[4..].join(" ");
                    println!("Validating address => region={}, postal={}, city='{}', street='{}'",
                             region_abbr, postal, city, street);

                    // We have to figure out which `WorldRegion` that abbreviation means. 
                    // For demonstration, assume "US" => MD region if you want or parse properly.
                    // (Real code might do a real map from "US" => "Maryland" or do a `dmv_regions()` search.)
                    let region_guessed = default_region; // forcing MD in example
                    if !region_guessed.abbreviation().eq_ignore_ascii_case(region_abbr) {
                        println!("Warning: we only loaded data for region {}, not for {}. Validation may fail!",
                                 region_guessed.abbreviation(), region_abbr);
                    }

                    let pc_obj = match PostalCode::new(Country::USA, postal) {
                        Ok(pc) => pc,
                        Err(e) => {
                            println!("Invalid postal code: {:?}", e);
                            continue;
                        }
                    };
                    let city_obj = match CityName::new(city) {
                        Ok(c) => c,
                        Err(e) => {
                            println!("Invalid city name: {:?}", e);
                            continue;
                        }
                    };
                    let street_obj = match StreetName::new(&street) {
                        Ok(s) => s,
                        Err(e) => {
                            println!("Invalid street name: {:?}", e);
                            continue;
                        }
                    };

                    let wa = match WorldAddressBuilder::default()
                        .region(region_guessed)
                        .postal_code(pc_obj)
                        .city(city_obj)
                        .street(street_obj)
                        .build()
                    {
                        Ok(a) => a,
                        Err(e) => {
                            println!("Could not build world address: {:?}", e);
                            continue;
                        }
                    };

                    match wa.validate_with(&da) {
                        Ok(_) => {
                            println!("Address is valid in region!");
                        }
                        Err(InvalidWorldAddress::CityNotFoundForPostalCodeInRegion { city, postal_code, region }) => {
                            println!("Invalid: city '{}' not found in postal code {} for region {:?}", city.name(), postal_code.code(), region);
                        }
                        Err(InvalidWorldAddress::StreetNotFoundForPostalCodeInRegion { street, postal_code, region }) => {
                            println!("Invalid: street '{}' not found in postal code {} for region {:?}", street.name(), postal_code.code(), region);
                        }
                        Err(e) => {
                            println!("Address is invalid: {:?}", e);
                        }
                    }

                    continue;
                }

                println!("Unknown command or bad syntax. Type 'help' for usage.");
            }
            Err(e) => {
                println!("Error reading input: {}", e);
            }
        }
    }
    Ok(())
}


