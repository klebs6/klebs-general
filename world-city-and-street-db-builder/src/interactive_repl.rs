crate::ix!();

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::collections::BTreeSet;
use std::io;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use rustyline::Editor;
use rustyline::completion::{Completer, Candidate, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Helper, Context, Result as RlResult};
use std::borrow::Cow;

#[derive(Debug)]
enum AutocompleteMode {
    City,
    Street,
}

/// We store separate lists for each region’s city/street names. In real code, you'd
/// do a single data structure or maybe a map keyed by (region, city/street). 
/// For brevity, we store them in simple vectors here.
struct RegionData {
    cities:  Vec<String>,
    streets: Vec<String>,
}

/// The REPL can store the data for all regions, plus a “current region” selection, 
/// and a “current mode” selection (city vs. street).
struct ReplState {
    regions:        std::collections::HashMap<WorldRegion, RegionData>,
    current_region: WorldRegion,
    mode:           AutocompleteMode,
    fuzzy_matcher:  SkimMatcherV2,

    // We also keep a `DataAccess` for validating addresses, etc. if needed
    db_access:      DataAccess,
}

/// Our custom Helper/Completer for rustyline.
struct DMVCompleter {
    state: Arc<Mutex<ReplState>>,
}

/// Implement `Helper`:
impl Helper for DMVCompleter {}

/// A minimal `Candidate` struct (or use `Pair` from `rustyline::completion::Pair`)
#[derive(Debug, Clone)]
pub struct MyCandidate {
    display: String,
    replacement: String,
}

impl Candidate for MyCandidate {
    fn display(&self) -> &str {
        &self.display
    }
    fn replacement(&self) -> &str {
        &self.replacement
    }
}

/// Implement `Validator` (no actual validation logic here):
impl Validator for DMVCompleter {
    // If you want advanced validation, override methods like `validate()`, `validate_while_typing()`, etc.
}

/// Implement `Highlighter` (just return strings as-is, or do coloring, etc.):
impl Highlighter for DMVCompleter {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _force: bool
    ) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(
        &self,
        hint: &'h str
    ) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }

    fn highlight<'l>(
        &self,
        line: &'l str,
        _pos: usize
    ) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }
}

/// Implement `Hinter`. The trait requires a type alias for `Hint`.
impl Hinter for DMVCompleter {
    // The `Hint` associated type is typically a `String`, or `Option<String>`, etc.
    type Hint = String;

    /// Return the hint for the current line if any.
    fn hint(
        &self,
        _line: &str,
        _pos: usize,
        _ctx: &Context<'_>
    ) -> Option<Self::Hint> {
        // For no hints, return `None`.
        None
    }
}


impl Completer for DMVCompleter {
    type Candidate = Pair;

    /// The main method: given the current “input line” and cursor position,
    /// return a list of completion candidates. We do fuzzy matching against either
    /// the city list or the street list of the current region.
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        // The substring user typed is from line[..pos].
        let typed_so_far = &line[..pos];

        // Acquire the ReplState
        let state_guard = self.state.lock().unwrap();

        let ReplState {
            ref regions,
            ref current_region,
            ref mode,
            ref fuzzy_matcher,
            ..
        } = *state_guard;

        let region_data = match regions.get(current_region) {
            Some(d) => d,
            None => {
                // If for some reason the region is not loaded, no completions:
                return Ok((pos, vec![]));
            }
        };

        // Decide which list to fuzzy-match:
        let all_items = match mode {
            AutocompleteMode::City => &region_data.cities,
            AutocompleteMode::Street => &region_data.streets,
        };

        // For each item in all_items, compute a fuzzy score with typed_so_far
        // We'll keep only the best-scoring items, or skip if below threshold.
        let mut candidates = Vec::new();
        for s in all_items {
            if let Some(score) = fuzzy_matcher.fuzzy_match(s, typed_so_far) {
                // We have a match => store it. We'll keep the raw string as well, 
                // along with the fuzzy score. 
                candidates.push((s.clone(), score));
            }
        }

        // Sort descending by score, then pick top e.g. 10
        candidates.sort_by_key(|(_, score)| -(*score));
        candidates.truncate(10);

        // Convert to Pair for rustyline
        let pairs: Vec<Pair> = candidates
            .into_iter()
            .map(|(s, _)| Pair {
                display: s.clone(),
                replacement: s, 
            })
            .collect();

        // We return (start, candidates). Typically we want to remove the entire typed string
        // so we replace from 'pos-len(typed_so_far)' up to 'pos'. Because typed_so_far
        // is exactly the substring from line[..pos], we can replace from `pos - typed_so_far.len()`.
        let start_pos = pos - typed_so_far.len();
        Ok((start_pos, pairs))
    }
}

/// Load all city names and street names from the DB for a given region,
/// by scanning prefix keys. This is the same general approach as in
/// the simpler example, but we return them as a Vec<String>.
fn load_region_data(db: &Database, region: &WorldRegion) -> RegionData {
    // We can reuse the earlier prefix scanners, e.g.:
    let c2z_prefix = format!("C2Z:{}:", region.abbreviation());
    let s2c_prefix = format!("S2C:{}:", region.abbreviation());

    let mut city_vec = Vec::new();
    let mut street_vec = Vec::new();

    // Gather city names from c2z keys: "C2Z:US:baltimore" => "baltimore"
    for item in db.db().prefix_iterator(c2z_prefix.as_bytes()) {
        if let Ok((key_bytes, _val_bytes)) = item {
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            let parts: Vec<&str> = key_str.splitn(3, ':').collect();
            if parts.len() == 3 {
                city_vec.push(parts[2].to_string());
            }
        }
    }

    // Gather street names from s2c keys: "S2C:US:north avenue" => "north avenue"
    for item in db.db().prefix_iterator(s2c_prefix.as_bytes()) {
        if let Ok((key_bytes, _val_bytes)) = item {
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            let parts: Vec<&str> = key_str.splitn(3, ':').collect();
            if parts.len() == 3 {
                street_vec.push(parts[2].to_string());
            }
        }
    }

    // Sort them for consistent iteration (optional):
    city_vec.sort();
    street_vec.sort();

    RegionData {
        cities: city_vec,
        streets: street_vec,
    }
}

/// The main REPL logic: we set up `rustyline` with our custom `DMVCompleter`,
/// then let the user type lines. As they type, fuzzy completions are shown.
/// We also parse a few special commands (region <abbr>, mode city, mode street, exit).
pub fn run_interactive_repl(db_arc: Arc<Mutex<Database>>) -> Result<(), Box<dyn std::error::Error>> {
    // Prepare the ReplState:
    let da = DataAccess::with_db(db_arc.clone());

    let md = WorldRegion::from(USRegion::UnitedState(UnitedState::Maryland));
    let va = WorldRegion::from(USRegion::UnitedState(UnitedState::Virginia));
    let dc = WorldRegion::from(USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia));

    // Preload city/street data for each
    let mut regions_map = std::collections::HashMap::new();
    {
        let guard = db_arc.lock().unwrap();
        regions_map.insert(md,  load_region_data(&guard, &md));
        regions_map.insert(va,  load_region_data(&guard, &va));
        regions_map.insert(dc,  load_region_data(&guard, &dc));
    }

    // We'll default to MD for now:
    let initial_state = ReplState {
        regions: regions_map,
        current_region: md,
        mode: AutocompleteMode::City,
        fuzzy_matcher: SkimMatcherV2::default(),
        db_access: da,
    };

    let repl_state = Arc::new(Mutex::new(initial_state));

    // Create the custom completer:
    let completer = DMVCompleter {
        state: repl_state.clone(),
    };

    // Build a rustyline Editor with our custom helper.
    let mut rl = Editor::<DMVCompleter, DefaultHistory>::new()?;
    rl.set_helper(Some(completer));

    // Optionally, load history from a file:
    let _ = rl.load_history("dmv_repl_history.txt");

    println!("Interactive DMV REPL with fuzzy autocomplete. Type 'exit' to quit.");
    println!("Commands: region <md|va|dc>, mode <city|street>, or any arbitrary input to test completion.");
    loop {
        let prompt = {
            let st = repl_state.lock().unwrap();
            format!("({:?}/{:?})> ", st.current_region.abbreviation(), st.mode)
        };
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                let line = line.trim().to_string();
                // Save in history
                rl.add_history_entry(line.as_str());

                // Check special commands:
                if line.eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                // Parse e.g. "region md"
                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.len() >= 2 && tokens[0].eq_ignore_ascii_case("region") {
                    let abbr = tokens[1].to_lowercase();
                    let mut st = repl_state.lock().unwrap();
                    let new_region = match abbr.as_str() {
                        "md" | "maryland" => md,
                        "va" | "virginia" => va,
                        "dc" => dc,
                        _ => {
                            println!("Unknown region: {}", abbr);
                            continue;
                        }
                    };
                    st.current_region = new_region;
                    println!("Region changed to: {:?}", new_region.abbreviation());
                    continue;
                }

                // "mode city" or "mode street"
                if tokens.len() >= 2 && tokens[0].eq_ignore_ascii_case("mode") {
                    let kind = tokens[1].to_lowercase();
                    let mut st = repl_state.lock().unwrap();
                    match kind.as_str() {
                        "city" => {
                            st.mode = AutocompleteMode::City;
                            println!("Autocomplete mode set to City.");
                        }
                        "street" => {
                            st.mode = AutocompleteMode::Street;
                            println!("Autocomplete mode set to Street.");
                        }
                        other => {
                            println!("Unknown mode: {}", other);
                        }
                    }
                    continue;
                }

                // Otherwise, do something with the line. We can just echo it or do validation if desired:
                println!("You typed: {}", line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("EOF (CTRL-D)");
                break;
            },
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }

    // Save history on exit
    let _ = rl.save_history("dmv_repl_history.txt");
    Ok(())
}
