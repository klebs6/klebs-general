crate::ix!();

pub fn write_house_number_ranges_into_storage(
    house_number_ranges: &HashMap<StreetName,Vec<HouseNumberRange>>, 
    region:              &WorldRegion,
    db:                  &mut Database
) -> Result<(),DatabaseConstructionError> 
{
    for (street, new_ranges) in &self.house_number_ranges {
        // load existing
        let existing_opt = load_house_number_ranges(db, &self.region, &street)?;
        let mut current = existing_opt.unwrap_or_default();

        // unify
        for rng in new_ranges {
            current = merge_house_number_range(current, rng.clone());
        }

        // store
        store_house_number_ranges(db, &self.region, &street, &current)?;
    }

    Ok(())
}
