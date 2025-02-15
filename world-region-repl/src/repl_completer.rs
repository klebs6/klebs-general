// ---------------- [ File: src/repl_completer.rs ]
crate::ix!();

/// (4) The custom Rustyline “Completer” that also implements Helper, Validator, Highlighter, Hinter.
pub struct MyCompleter<I:StorageInterface> {
    state: Arc<Mutex<ReplState<I>>>,
}

impl<I:StorageInterface> MyCompleter<I> {

    pub fn new(state: Arc<Mutex<ReplState<I>>>) -> Self {
        Self { state }
    }
}

impl<I:StorageInterface> Helper      for MyCompleter<I> {}
impl<I:StorageInterface> Validator   for MyCompleter<I> {}
impl<I:StorageInterface> Highlighter for MyCompleter<I> {}

impl<I:StorageInterface> Hinter      for MyCompleter<I> {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl<I:StorageInterface> Completer for MyCompleter<I> {

    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> RlResult<(usize, Vec<Pair>)> {
        let typed_so_far = &line[..pos]; 
        let lower = typed_so_far.trim_start().to_lowercase();
        let tokens: Vec<&str> = lower.split_whitespace().collect();

        let guard = self.state.lock().map_err(|_e| {
            // Return an empty completion set on lock error
            rustyline::error::ReadlineError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "REPL State lock error",
            ))
        })?;

        let st = &*guard;

        // 1) If the user typed "validate...", we rely on the “longest_city” logic
        if tokens.first().map(|s| *s) == Some("validate") {
            let ends_with_space = typed_so_far.chars().last().map(|ch| ch.is_whitespace()).unwrap_or(false);
            match complete_validate_line_longest_city(typed_so_far, ends_with_space, st) {
                Ok(sugs) if !sugs.is_empty() => {
                    let pairs = sugs.into_iter()
                        .map(|s| Pair { display: s.clone(), replacement: s })
                        .collect();
                    // Identify how much the user typed since last space:
                    let replace_start = typed_so_far.rfind(' ').map(|idx| idx + 1).unwrap_or(0);
                    let offset = pos - replace_start;
                    return Ok((pos - offset, pairs));
                }
                _ => {
                    return Ok((pos, vec![]));
                }
            }
        }

        // 2) If the user typed "region ..."
        if tokens.first().map(|s| *s) == Some("region") {
            // If there is only "region" typed or we are at the second token, gather region abbr
            let maybe_prefix = tokens.get(1).copied().unwrap_or("");
            let all_regions = st.regions().keys();
            let mut matches = Vec::new();
            for r in all_regions {
                let ab = r.abbreviation().to_lowercase();
                // do a simple starts_with or fuzzy:
                if ab.starts_with(maybe_prefix) {
                    matches.push(ab);
                }
            }
            matches.sort();
            matches.dedup();

            let pairs = matches.into_iter()
                .map(|m| Pair { display: m.clone(), replacement: m })
                .collect::<Vec<_>>();

            // The position from which we do the replacement:
            let replace_start = typed_so_far.rfind(' ').map(|idx| idx + 1).unwrap_or(0);
            let offset = pos - replace_start;
            return Ok((pos - offset, pairs));
        }

        // 3) If the user typed "mode ..."
        if tokens.first().map(|s| *s) == Some("mode") {
            // We only have two possible completions: "city" or "street"
            let maybe_prefix = tokens.get(1).copied().unwrap_or("");
            let possibilities = ["city", "street"];
            let mut matches = Vec::new();
            for p in possibilities.iter() {
                if p.starts_with(maybe_prefix) {
                    matches.push((*p).to_string());
                }
            }
            let pairs = matches.into_iter()
                .map(|m| Pair { display: m.clone(), replacement: m })
                .collect::<Vec<_>>();

            let replace_start = typed_so_far.rfind(' ').map(|idx| idx + 1).unwrap_or(0);
            let offset = pos - replace_start;
            return Ok((pos - offset, pairs));
        }

        // 4) Otherwise fall back to city/street completions (like your existing code)
        let region_data = match st.regions().get(st.current_region()) {
            Some(rd) => rd,
            None => return Ok((pos, vec![])),
        };
        let all_items: &Vec<String> = match st.mode() {
            AutocompleteMode::City => region_data.cities(),
            AutocompleteMode::Street => region_data.streets(),
        };

        let mut candidates = Vec::new();
        for s in all_items {
            if let Some(score) = st.fuzzy_matcher().fuzzy_match(s, &lower) {
                candidates.push((s.to_string(), score));
            }
        }
        // sort descending by score
        candidates.sort_by_key(|(_, sc)| -sc);
        candidates.truncate(10);

        let out_pairs = candidates
            .into_iter()
            .map(|(txt, _)| Pair {
                display: txt.clone(),
                replacement: txt,
            })
            .collect();

        // Where do we begin replacing?
        let start_pos = pos - lower.len();
        Ok((start_pos, out_pairs))
    }
}
