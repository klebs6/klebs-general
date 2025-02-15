// ---------------- [ File: src/repl_state.rs ]
crate::ix!();

/// (3) Our main REPL state:
///   - `regions`: which regions exist, keyed by the actual `WorldRegion` object
///   - `current_region`: which region the user is “in” at the moment
///   - `mode`: city/street auto-completion mode
///   - `fuzzy_matcher`: fuzzy logic for auto-complete and suggestion
///   - `db_access`: for deep DB queries
///   - `house_number_ranges`: a mapping from (region, street) to a list of (start, end) ranges
///        so we can do range queries (or store them in DB).
///     For simplicity, we store by the *lowercased street name*, 
///     plus the region’s abbreviation, e.g. `(region_abbr, "north avenue")`.
#[derive(Builder,Getters,Setters)]
#[getset(get="pub",set="pub")]
#[builder(pattern = "owned")]
pub struct ReplState<I:StorageInterface> {
    regions:        HashMap<WorldRegion, RegionData>,
    current_region: WorldRegion,
    mode:           AutocompleteMode,
    fuzzy_matcher:  Arc<SkimMatcherV2>,
    db_access:      Arc<DataAccess<I>>,

    /// Key = (region.abbreviation().to_string(), street_lc), Value = Vec of (start, end)
    house_number_ranges: HashMap<(String, String), Vec<HouseNumberRange>>,
}

impl<I:StorageInterface> ReplState<I> {

    pub fn current_country(&self) -> Country {
        self.current_region.try_into().expect(&format!("expected our region {:?} would be convertible to Country", self.current_region))
    }
}
