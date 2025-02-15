crate::ix!();

/* ------------------------------------------------------------------------
   5) If user typed no street => partial usage => done
   ------------------------------------------------------------------------ */
pub fn handle_no_street(
    lines:             &mut Vec<String>,
    zip_str:           &str,
    city_name:         &str,
    house_number_part: &Option<String>,
) {
    lines.push(format!(
        "ZIP='{}' + city='{}' => validated. No street typed.",
        zip_str,
        city_name
    ));
    if let Some(num_s) = house_number_part {
        lines.push(format!(
            "(house number={} typed but no street => ignoring or partial?).",
            num_s
        ));
    }
}
