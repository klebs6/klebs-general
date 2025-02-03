// ---------------- [ File: src/addresses_from_pbf_file_with_house_numbers.rs ]
crate::ix!();

pub fn addresses_from_pbf_file_with_house_numbers(
    path: PathBuf,
    world_region: WorldRegion,
    db: Arc<Mutex<Database>>,
) -> Result<impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>, OsmPbfParseError>
{
    let country = Country::try_from(world_region)?;
    let (tx, rx) = std::sync::mpsc::sync_channel(1000);

    std::thread::spawn(move || {
        let mut aggregator: HashMap<StreetName, Vec<HouseNumberRange>> = HashMap::new();

        // Open the PBF
        let reader = match osmpbf::ElementReader::from_path(&path) {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(Err(OsmPbfParseError::OsmPbf(e)));
                return;
            }
        };

        // For each element
        let result = reader.for_each(|element| {
            // 1) Attempt to parse city/street/postcode
            match AddressRecord::try_from((&element, &country)) {
                Ok(rec) => {
                    // If city, street, postcode exist, build a WorldAddress
                    if let (Some(city), Some(street), Some(postcode)) =
                        (rec.city(), rec.street(), rec.postcode())
                    {
                        let addr = match WorldAddressBuilder::default()
                            .region(world_region)
                            .city(city.clone())
                            .street(street.clone())
                            .postal_code(postcode.clone())
                            .build()
                        {
                            Ok(a) => a,
                            Err(_) => {
                                // skip
                                return;
                            }
                        };

                        // Send the address downstream
                        if tx.send(Ok(addr)).is_err() {
                            // The consumer/receiver was dropped => stop
                            return;
                        }

                        // 2) Also see if there is a house-number range
                        //    using `extract_house_number_range_from_element(...)`
                        match extract_house_number_range_from_element(&element) {
                            Ok(Some(new_range)) => {
                                aggregator
                                    .entry(street.clone())
                                    .or_default()
                                    .push(new_range);
                            }
                            _ => { /* no housenumber or parse error => skip */ }
                        }
                    }
                }
                Err(_) => {
                    // If not a valid address, skip it
                }
            }
        });

        // If osmpbf returned an error mid-iteration:
        if let Err(e) = result {
            let _ = tx.send(Err(OsmPbfParseError::OsmPbf(e)));
        }

        // Merge + store aggregator
        if let Ok(mut db_guard) = db.lock() {
            for (street, subranges) in aggregator {
                let existing_opt = match load_house_number_ranges(&db_guard, &world_region, &street) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!("Could not load existing ranges for street {:?}: {:?}", street, e);
                        None
                    }
                };
                let mut current = existing_opt.unwrap_or_default();
                for rng in subranges {
                    current = merge_house_number_range(current, rng);
                }
                if let Err(e) = store_house_number_ranges(&mut db_guard, &world_region, &street, &current) {
                    tracing::warn!("Could not store updated subranges for street {:?}: {:?}", street, e);
                }
            }
        } else {
            tracing::warn!("Could not lock DB at the end of addresses_from_pbf_file_with_house_numbers()");
        }
    });

    Ok(rx.into_iter())
}
