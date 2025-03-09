// ---------------- [ File: src/show_cities_for_zip.rs ]
crate::ix!();

/* ------------------------------------------------------------------------
   3) If user typed no city => partial usage => show city set
   ------------------------------------------------------------------------ */
pub fn show_cities_for_zip<I:StorageInterface>(
    lines:  &mut Vec<String>,
    st:     &ReplState<I>,
    region: &WorldRegion,
    pc_obj: &PostalCode,
) {
    let z2c_str = z2c_key(region, pc_obj);
    if let Some(cityset) = st.db_access().get_city_set(&z2c_str) {
        if cityset.is_empty() {
            lines.push(format!(
                "ZIP '{}' recognized in DB, but no city data for region {}?",
                pc_obj.code(),
                region.abbreviation()
            ));
        } else {
            lines.push(format!(
                "ZIP '{}' recognized. Known cities:",
                pc_obj.code()
            ));
            for c in &cityset {
                lines.push(format!("  {}", c.name()));
            }
        }
    } else {
        lines.push(format!(
            "ZIP '{}' not recognized in region {}.",
            pc_obj.code(),
            region.abbreviation()
        ));
    }
}
