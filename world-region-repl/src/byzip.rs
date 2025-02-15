// ---------------- [ File: src/byzip.rs ]
crate::ix!();

pub fn handle_byzip_command<I:StorageInterface>(zip: &str, st: &ReplState<I>) {
    let mut lines = Vec::new();

    let region = st.current_region();
    if let Ok(pc_obj) = PostalCode::new(st.current_country(), zip) {
        let z2c_key = z2c_key(&region, &pc_obj);
        if let Some(cities) = st.db_access().get_city_set(&z2c_key) {
            lines.push(format!("Cities in ZIP '{}':", zip));
            for c in &cities {
                lines.push(format!("  {}", c.name()));
            }
        } else {
            lines.push(format!("No city data for zip '{}'.", zip));
        }

        let s_key = s_key(&region, &pc_obj);
        if let Some(sts) = st.db_access().get_street_set(&s_key) {
            lines.push(format!("Streets in ZIP '{}':", zip));
            for s in &sts {
                lines.push(format!("  {}", s.name()));
            }
        } else {
            lines.push(format!("No street data for zip '{}'.", zip));
        }
    } else {
        lines.push(format!("Invalid postal code '{}'.", zip));
    }

    print_or_page(&lines);
}
