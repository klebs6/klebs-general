crate::ix!();

/* ------------------------------------------------------------------------
   6) Parse the street, build the final address, and validate
   ------------------------------------------------------------------------ */
pub fn check_and_validate_street<I:StorageInterface>(
    lines:             &mut Vec<String>,
    st:                &mut ReplState<I>,
    region:            &WorldRegion,
    street_parts:      impl IntoIterator<Item = String>,
    city_obj:          CityName,
    pc_obj:            &PostalCode,
    house_number_part: &Option<String>,

) -> Result<(), ()> {

    // a) parse street
    let street_str = street_parts.into_iter().collect::<Vec<String>>().join(" ");
    let street_obj = match StreetName::new(&street_str) {
        Ok(s) => s,
        Err(e) => {
            lines.push(format!("Invalid street '{}': {:?}", street_str, e));
            return Err(());
        }
    };

    // b) build address
    let wa = match WorldAddressBuilder::default()
        .region(*region)
        .postal_code(pc_obj.clone())
        .city(city_obj.clone())
        .street(street_obj.clone())
        .build()
    {
        Ok(w) => w,
        Err(e) => {
            lines.push(format!(
                "Could not build final address from zip='{}', city='{}', street='{}': {:?}",
                pc_obj.code(),
                city_obj.name(),
                street_obj.name(),
                e
            ));
            return Err(());
        }
    };

    // c) validate address in DB
    match wa.validate_with(&**st.db_access()) {
        Ok(_) => {
            // If user typed house number => check range
            if let Some(num_s) = house_number_part {
                if let Ok(num) = num_s.parse::<u32>() {
                    let ab = region.abbreviation().to_string();
                    let st_lc = street_obj.name().to_string();
                    if let Some(ranges) = st.house_number_ranges().get(&(ab, st_lc.clone())) {
                        let in_range = ranges.iter().any(|range| {
                            num >= *range.start() && num <= *range.end()
                        });
                        if in_range {
                            lines.push(format!(
                                "Address is valid (including house_num={}).",
                                num
                            ));
                        } else {
                            lines.push(format!(
                                "Address found but house_num={} not in known range for '{}'.",
                                num, st_lc
                            ));
                        }
                    } else {
                        lines.push(format!(
                            "Address is valid, but no known house range => can't confirm house_num={}.",
                            num
                        ));
                    }
                } else {
                    lines.push(
                        "Address is valid (zip+city+street) but houseNum parse error => ignoring."
                            .to_string(),
                    );
                }
            } else {
                lines.push("Address is fully valid (zip+city+street)!".to_string());
            }
        }
        Err(e) => {
            lines.push(format!("Invalid address: {:?}", e));
        }
    }

    Ok(())
}
