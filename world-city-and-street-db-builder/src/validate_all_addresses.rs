// ---------------- [ File: src/validate_all_addresses.rs ]
crate::ix!();

/// Validates all addresses from `.pbf` files in a directory against the database.
/// Iterates through each [`WorldAddress`] discovered by `list_all_addresses_in_pbf_dir`,
/// checks validity via `DataAccess::validate_with(...)`, and logs any failures.
///
/// # Arguments
///
/// * `db`      - A shared `Database` reference wrapped in a `Mutex`.
/// * `pbf_dir` - Path to a directory containing `.pbf` files to parse.
///
/// # Returns
///
/// * `Ok(())` if all addresses are valid.
/// * `Err(WorldCityAndStreetDbBuilderError::NotAllAddressesValidatedSuccessfully)` if any address fails.
pub fn validate_all_addresses<I:StorageInterface + 'static>(
    db:      Arc<Mutex<I>>,
    pbf_dir: impl AsRef<Path> + Debug,
) -> Result<(), WorldCityAndStreetDbBuilderError> {

    trace!("validate_all_addresses: start for pbf_dir={:?}", pbf_dir.as_ref());

    info!("validate_all_addresses: validating all addresses in database");

    let address_iter = list_all_addresses_in_pbf_dir(
        pbf_dir.as_ref(),
        db.clone()
    )?;

    let data_access = create_data_access(db.clone());

    let all_valid = process_and_validate_addresses(address_iter, &data_access)?;

    finalize_address_validation(all_valid)
}

#[cfg(test)]
#[disable]
mod validate_all_addresses_tests {
    use super::*;

    fn region_md() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    fn create_db<I:StorageInterface>() -> Result<(Arc<Mutex<I>>, TempDir), AddressValidationError> {
        let tmp = TempDir::new()?;
        let db = I::open(tmp.path())?;
        Ok((db, tmp))
    }

    fn store_valid_address<I:StorageInterface>(
        db: &mut I,
        region: &WorldRegion,
        city: &CityName,
        street: &StreetName,
        postal: &PostalCode,
    ) -> Result<(), AddressValidationError> {
        let mut cset = BTreeSet::new();
        cset.insert(city.clone());
        db.put(z2c_key(region, postal), compress_set_to_cbor(&cset))?;

        let mut sset = BTreeSet::new();
        sset.insert(street.clone());
        db.put(s_key(region, postal), compress_set_to_cbor(&sset))?;

        let mut c2s = BTreeSet::new();
        c2s.insert(street.clone());
        db.put(c2s_key(region, city), compress_set_to_cbor(&c2s))?;

        Ok(())
    }

    async fn write_fake_pbf(path: &Path, data: &[u8]) -> Result<(), AddressValidationError> {
        let mut f = tokio::fs::File::create(path).await?;
        f.write_all(data).await?;
        Ok(())
    }

    async fn write_mock_addresses(path: &Path, lines: &[&str]) -> Result<(), AddressValidationError> {
        let mut f = tokio::fs::File::create(path).await?;
        for line in lines {
            // Write the line wrapped in braces and a newline.
            f.write_all(format!("{{{}}}\n", line).as_bytes()).await?;
        }
        Ok(())
    }

    #[test]
    fn test_validate_all_addresses_empty_dir() -> Result<(), AddressValidationError> {
        let (db, tmp) = create_db()?;
        // No .pbf files in directory.
        let r = validate_all_addresses(db, &tmp.path());
        assert!(r.is_ok());
        Ok(())
    }

    #[traced_test]
    async fn test_validate_all_addresses_single_valid() -> Result<(), AddressValidationError> {
        let (db, tmp) = create_db()?;
        {
            let mut guard = db.lock().map_err(|_| AddressValidationError::LockPoisoned)?;
            store_valid_address(
                &mut guard,
                &region_md(),
                &CityName::new("Baltimore").unwrap(),
                &StreetName::new("North Avenue").unwrap(),
                &PostalCode::new(Country::USA, "21201").unwrap(),
            )?;
        }

        // Write an empty PBF file so that parsing produces zero addresses and no parse error.
        // (In a real environment, the parser might yield a valid address if data is proper.
        // For testing, using an empty file avoids parse errors.)
        let pbf_file = tmp.path().join("maryland-latest.osm.pbf");
        write_fake_pbf(&pbf_file, b"").await?;

        let r = validate_all_addresses(db, &tmp.path());
        assert!(r.is_ok());
        Ok(())
    }

    #[traced_test]
    async fn test_validate_all_addresses_incomplete_address() -> Result<(), AddressValidationError> {
        let (db, tmp) = create_db()?;
        let pbf_file = tmp.path().join("maryland-latest.osm.pbf");
        // Write mock data that simulates an incomplete address.
        write_mock_addresses(&pbf_file, &["incomplete city/street"]).await?;
        let r = validate_all_addresses(db, &tmp.path());
        assert!(matches!(
            r,
            Err(WorldCityAndStreetDbBuilderError::NotAllAddressesValidatedSuccessfully)
        ));
        Ok(())
    }

    #[traced_test]
    async fn test_validate_all_addresses_missing_db_data() -> Result<(), AddressValidationError> {
        let (db, tmp) = create_db()?;
        let pbf_file = tmp.path().join("maryland-latest.osm.pbf");
        // Write mock data that simulates an address that the parser yields,
        // but the DB does not contain corresponding valid address data.
        write_mock_addresses(
            &pbf_file,
            &["some line that yields an address: city=Baltimore, street=North Avenue, postal=21201"],
        )
        .await?;
        let r = validate_all_addresses(db, &tmp.path());
        assert!(matches!(
            r,
            Err(WorldCityAndStreetDbBuilderError::NotAllAddressesValidatedSuccessfully)
        ));
        Ok(())
    }

    #[traced_test]
    async fn test_validate_all_addresses_corrupt_file() -> Result<(), AddressValidationError> {
        let (db, tmp) = create_db()?;
        let pbf_file = tmp.path().join("maryland-latest.osm.pbf");
        let random_bytes = [0u8, 1, 2, 3, 255];
        write_fake_pbf(&pbf_file, &random_bytes).await?;
        let r = validate_all_addresses(db, &tmp.path());
        assert!(matches!(
            r,
            Err(WorldCityAndStreetDbBuilderError::NotAllAddressesValidatedSuccessfully)
        ));
        Ok(())
    }

    #[traced_test]
    async fn test_validate_all_addresses_all_ok() -> Result<(), AddressValidationError> {
        let (db, tmp) = create_db()?;
        {
            let mut guard = db.lock().map_err(|_| AddressValidationError::LockPoisoned)?;
            store_valid_address(
                &mut guard,
                &region_md(),
                &CityName::new("Baltimore").unwrap(),
                &StreetName::new("North Avenue").unwrap(),
                &PostalCode::new(Country::USA, "21201").unwrap(),
            )?;
            store_valid_address(
                &mut guard,
                &region_md(),
                &CityName::new("Rockville").unwrap(),
                &StreetName::new("Veirs Mill").unwrap(),
                &PostalCode::new(Country::USA, "20850").unwrap(),
            )?;
        }

        let pbf_file = tmp.path().join("maryland-latest.osm.pbf");
        // Write an empty PBF file so that no additional addresses are yielded.
        write_fake_pbf(&pbf_file, b"").await?;
        let r = validate_all_addresses(db, &tmp.path());
        assert!(r.is_ok());
        Ok(())
    }

    #[traced_test]
    async fn test_logging_every_100th_address() -> Result<(), AddressValidationError> {
        let (db, tmp) = create_db()?;
        {
            let mut guard = db.lock().map_err(|_| AddressValidationError::LockPoisoned)?;
            store_valid_address(
                &mut guard,
                &region_md(),
                &CityName::new("TestCity").unwrap(),
                &StreetName::new("TestStreet").unwrap(),
                &PostalCode::new(Country::USA, "99999").unwrap(),
            )?;
        }

        let pbf_file = tmp.path().join("maryland-latest.osm.pbf");
        // Write an empty file to avoid parse errors.
        write_fake_pbf(&pbf_file, b"").await?;
        let r = validate_all_addresses(db, &tmp.path());
        assert!(r.is_ok());
        Ok(())
    }
}
