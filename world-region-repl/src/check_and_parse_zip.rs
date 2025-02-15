crate::ix!();

/* ------------------------------------------------------------------------
   2) Check if ZIP is valid
   ------------------------------------------------------------------------ */
pub fn check_and_parse_zip<I:StorageInterface>(
    zip_str: &str,
    st: &mut ReplState<I>,
    lines: &mut Vec<String>,
    region: &WorldRegion,
) -> Option<PostalCode> {
    if zip_str.is_empty() {
        lines.push("Usage: validate <zip> <city> [house_num] <street...>".to_string());
        return None;
    }

    match PostalCode::new(st.current_country(), zip_str) {
        Ok(pc) => Some(pc),
        Err(e) => {
            lines.push(format!("Invalid zip: {:?}", e));
            None
        }
    }
}
