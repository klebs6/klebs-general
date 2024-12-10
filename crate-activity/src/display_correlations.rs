crate::ix!();

pub const STRONG_CORRELATION_MAGNITUDE: f64 = 0.7;

pub fn display_correlations(correlations: &[(String, String, f64)]) {
    let mut correlation_map: HashMap<String, Vec<(String, f64)>> = HashMap::new();

    // Organize correlations into a map for efficient grouping
    for (crate_a, crate_b, correlation) in correlations {
        if correlation.abs() >= STRONG_CORRELATION_MAGNITUDE {
            correlation_map
                .entry(crate_a.clone())
                .or_default()
                .push((crate_b.clone(), *correlation));
        }
    }

    println!("----------------[crate-correlation-analysis]----------------");

    // Sort and prepare the display output
    for (crate_name, correlated_crates) in correlation_map.iter_mut() {

        // Sort correlations by their absolute values in descending order
        correlated_crates.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());

        // Display the crate and its correlations
        println!("{}", crate_name);
        for (name, value) in correlated_crates {
            println!("  {:>6.2}  {}", value, name);
        }
        println!("");
    }
}
