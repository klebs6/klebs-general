// ---------------- [ File: src/handle_validate_command.rs ]
crate::ix!();

pub fn handle_validate_command<I:StorageInterface>(line: &str, st: &mut ReplState<I>) -> Result<(), ValidateCommandError> {
    let mut lines = Vec::new();

    // 1) Parse tokens for `validate` command
    let parse = match parse_validate_args(line, &mut lines, st) {
        Some(p) => p,
        None => {
            // A None means we already displayed usage or an error
            print_or_page(&lines);
            return Ok(());
        }
    };

    // 2) Check if ZIP is valid
    let region = st.current_region().clone();
    let zip_str = parse.zip_part();
    let pc_obj = match check_and_parse_zip(zip_str, st, &mut lines, &region) {
        Some(pc) => pc,
        None => {
            print_or_page(&lines);
            return Ok(());
        }
    };

    // 3) If user did not type a city => partial usage => show city set
    if parse.city_parts().is_empty() {
        show_cities_for_zip(&mut lines, st, &region, &pc_obj);
        print_or_page(&lines);
        return Ok(());
    }

    // 4) Check if city is recognized for that ZIP
    let city_obj = match check_city_validity(&mut lines, st, &region, &pc_obj, parse.city_parts().clone()) {
        Some(city) => city,
        None => {
            print_or_page(&lines);
            return Ok(());
        }
    };

    // 5) If user typed no street => partial usage => done
    if parse.street_parts().is_empty() {
        handle_no_street(&mut lines, zip_str, city_obj.name(), parse.house_number_part());
        print_or_page(&lines);
        return Ok(());
    }

    // 6) Parse the street, build final address, validate
    match check_and_validate_street(
        &mut lines,
        st,
        &region,
        parse.street_parts().clone(),
        city_obj,
        &pc_obj,
        parse.house_number_part()
    ) {
        Ok(()) => {
            // all done
        }
        Err(_) => {
            // error info is already in lines
        }
    }

    // 7) Print or page everything
    print_or_page(&lines);
    Ok(())
}
