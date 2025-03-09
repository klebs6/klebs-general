// ---------------- [ File: src/check_city_validity.rs ]
crate::ix!();

/* ------------------------------------------------------------------------
   4) Check if city is recognized for that ZIP
   ------------------------------------------------------------------------ */
pub fn check_city_validity<'a,I:StorageInterface>(
    lines:      &mut Vec<String>,
    st:         &ReplState<I>,
    region:     &WorldRegion,
    pc_obj:     &PostalCode,
    city_parts: impl IntoIterator<Item = String>,

) -> Option<CityName> {

    // e.g. parse city object
    let city_str = city_parts.into_iter().collect::<Vec<String>>().join(" ");
    let city_str_lc = city_str.to_lowercase();
    let city_obj = match CityName::new(&city_str_lc) {
        Ok(cx) => cx,
        Err(e) => {
            lines.push(format!("Invalid city '{}': {:?}", city_str, e));
            return None;
        }
    };

    // confirm city is recognized
    let z2c_str = z2c_key(region, pc_obj);
    if let Some(cs) = st.db_access().get_city_set(&z2c_str) {
        if !cs.iter().any(|cx| cx.name() == city_obj.name()) {
            lines.push(format!(
                "City '{}' not recognized in zip {}",
                city_obj.name(),
                pc_obj.code()
            ));
            return None;
        }
    } else {
        lines.push(format!(
            "ZIP '{}' not recognized in region {}.",
            pc_obj.code(),
            region.abbreviation()
        ));
        return None;
    }

    Some(city_obj)
}
