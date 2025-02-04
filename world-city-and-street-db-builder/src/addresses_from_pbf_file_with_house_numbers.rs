// ---------------- [ File: src/addresses_from_pbf_file_with_house_numbers.rs ]
crate::ix!();

/// The top-level function orchestrates:
/// 1) Converting a [`WorldRegion`] into a [`Country`].
/// 2) Creating a streaming channel.
/// 3) Spawning a background thread to:
///    - Open and parse the OSM PBF file.
///    - Accumulate house number ranges in memory.
///    - Send intermediate address results over the channel.
///    - Store aggregated house number ranges into the database.
/// 4) Returning the consumer side of that channel as an [`Iterator`].
///
/// # Arguments
///
/// * `path`         - Path to an OSM PBF file on disk.
/// * `world_region` - Geographic region used for country inference.
/// * `db`           - Shared mutable database handle.
///
/// # Returns
///
/// * `Ok(impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>)` on success.
/// * `Err(OsmPbfParseError)` if the country conversion fails immediately.
pub fn addresses_from_pbf_file_with_house_numbers(
    path: PathBuf,
    world_region: WorldRegion,
    db: Arc<Mutex<Database>>,
) -> Result<impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>, OsmPbfParseError> {
    trace!("addresses_from_pbf_file_with_house_numbers: Invoked with path={:?}, region={:?}", path, world_region);

    let country = try_resolve_country(world_region)?;
    trace!("addresses_from_pbf_file_with_house_numbers: Resolved country={:?}", country);

    let (tx, rx) = create_address_stream_channel();
    trace!("addresses_from_pbf_file_with_house_numbers: Created sync_channel for address streaming");

    // Move ownership into background thread
    thread::spawn(move || {
        handle_pbf_house_number_extractor_in_thread(path, country, world_region, db, tx);
    });

    // Provide the consumer an iterator of results
    Ok(rx.into_iter())
}

#[cfg(test)]
mod addresses_from_pbf_file_with_house_numbers_tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    use tokio::runtime::Runtime;

    /// A convenience function that creates a minimal `.osm.pbf` file containing:
    ///   - A single Node with complete address tags (addr:city/street/postcode).
    ///   - Optionally includes an `addr:housenumber` to test aggregator storage.
    fn create_minimal_address_pbf(
        path: &std::path::Path,
        city: &str,
        street: &str,
        postcode: &str,
        housenumber: Option<&str>,
        node_id: i64,
    ) -> std::io::Result<()> {
        // 1) Build OSMHeader
        let mut header_block = crate::proto::osmformat::HeaderBlock::new();
        {
            let mut bbox = crate::proto::osmformat::HeaderBBox::new();
            bbox.set_left(-77_000_000_000);
            bbox.set_right(-76_000_000_000);
            bbox.set_top(39_000_000_000);
            bbox.set_bottom(38_000_000_000);
            header_block.bbox = protobuf::MessageField::from_option(Some(bbox));
            header_block.required_features.push("OsmSchema-V0.6".to_string());
            header_block.required_features.push("DenseNodes".to_string());
        }
        let header_bytes = header_block.write_to_bytes()?;

        let mut header_blob = crate::proto::fileformat::Blob::new();
        header_blob.set_raw(header_bytes.clone());
        header_blob.set_raw_size(header_bytes.len() as i32);
        let header_blob_bytes = header_blob.write_to_bytes()?;

        let mut header_blobheader = crate::proto::fileformat::BlobHeader::new();
        header_blobheader.set_type("OSMHeader".to_string());
        header_blobheader.set_datasize(header_blob_bytes.len() as i32);
        let header_blobheader_bytes = header_blobheader.write_to_bytes()?;

        // 2) PrimitiveBlock with a single Node
        let mut block = crate::proto::osmformat::PrimitiveBlock::new();
        {
            let mut s_table = crate::proto::osmformat::StringTable::new();
            // index=0 => ""
            // index=1 => "addr:city" => city_val=2
            // index=3 => "addr:street" => street_val=4
            // index=5 => "addr:postcode" => pc_val=6
            // optionally index=7 => "addr:housenumber" => hval=8
            s_table.s.push(b"".to_vec());
            s_table.s.push(b"addr:city".to_vec());
            s_table.s.push(city.as_bytes().to_vec());
            s_table.s.push(b"addr:street".to_vec());
            s_table.s.push(street.as_bytes().to_vec());
            s_table.s.push(b"addr:postcode".to_vec());
            s_table.s.push(postcode.as_bytes().to_vec());

            let mut node_keys = vec![1, 3, 5];
            let mut node_vals = vec![2, 4, 6];
            if let Some(hn) = housenumber {
                s_table.s.push(b"addr:housenumber".to_vec()); // idx=7
                s_table.s.push(hn.as_bytes().to_vec());       // idx=8
                node_keys.push(7);
                node_vals.push(8);
            }

            block.stringtable = protobuf::MessageField::from_option(Some(s_table));
            block.set_granularity(100);
            block.set_lat_offset(0);
            block.set_lon_offset(0);

            let mut group = crate::proto::osmformat::PrimitiveGroup::new();
            let mut node = crate::proto::osmformat::Node::new();
            node.set_id(node_id);
            // lat/lon near 39.0/-76.0
            node.set_lat((39.0 * 1e9) as i64 / 100);
            node.set_lon((-76.0 * 1e9) as i64 / 100);

            // Insert keys/vals
            for k in &node_keys {
                node.keys.push(*k);
            }
            for v in &node_vals {
                node.vals.push(*v);
            }

            group.nodes.push(node);
            block.primitivegroup.push(group);
        }
        let block_bytes = block.write_to_bytes()?;

        let mut data_blob = crate::proto::fileformat::Blob::new();
        data_blob.set_raw(block_bytes.clone());
        data_blob.set_raw_size(block_bytes.len() as i32);
        let data_blob_bytes = data_blob.write_to_bytes()?;

        let mut data_blobheader = crate::proto::fileformat::BlobHeader::new();
        data_blobheader.set_type("OSMData".to_string());
        data_blobheader.set_datasize(data_blob_bytes.len() as i32);
        let data_blobheader_bytes = data_blobheader.write_to_bytes()?;

        // 3) Write it all to disk
        let mut file = std::fs::File::create(path)?;
        // write header
        file.write_all(&(header_blobheader_bytes.len() as u32).to_be_bytes())?;
        file.write_all(&header_blobheader_bytes)?;
        file.write_all(&header_blob_bytes)?;
        // write data
        file.write_all(&(data_blobheader_bytes.len() as u32).to_be_bytes())?;
        file.write_all(&data_blobheader_bytes)?;
        file.write_all(&data_blob_bytes)?;

        Ok(())
    }

    // ----------------------------------------------------------------------
    // 1) Test unknown region => immediate error from try_resolve_country
    // ----------------------------------------------------------------------
    #[test]
    fn test_addresses_from_pbf_file_with_house_numbers_unknown_region() {
        // Suppose we define some region that is not recognized by `try_resolve_country(...)`
        // We'll do a partial approach: e.g. if your code doesn't know how to parse "Guam"
        // or something. We'll do it.
        let unknown_region: WorldRegion = USRegion::USTerritory(USTerritory::Guam).into();
        let tmp_db = Database::open(std::env::temp_dir().join("dummy_db_path")).unwrap();
        let result = addresses_from_pbf_file_with_house_numbers(
            "some_file.osm.pbf".into(),
            unknown_region,
            tmp_db,
        );
        // We expect an immediate Err(...) from try_resolve_country
        assert!(result.is_err());
        match result.err().unwrap() {
            OsmPbfParseError::WorldRegionConversionError(_) => {
                // Good, as expected
            }
            other => panic!("Expected WorldRegionConversionError, got {:?}", other),
        }
    }

    // ----------------------------------------------------------------------
    // 2) Test missing .pbf file => the background thread tries to open => fails => yields an error
    // ----------------------------------------------------------------------
    #[test]
    fn test_addresses_from_pbf_file_with_house_numbers_missing_file() {
        let known_region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let db = Database::open(std::env::temp_dir().join("dummy_db_2")).unwrap();
        let db_clone = db.clone();

        // We'll pass a path that doesn't exist.
        let missing_path = std::path::PathBuf::from("/path/does/not/exist/osm.pbf");

        let result_iter = addresses_from_pbf_file_with_house_numbers(
            missing_path,
            known_region,
            db_clone,
        )
        .expect("should succeed in returning an iterator, even if file is missing => the error is produced in the thread");

        // Now we consume the iterator. The first item should be Err(...) from the open failure.
        let first = result_iter.into_iter().next();
        assert!(first.is_some());
        let first_err = first.unwrap();
        assert!(first_err.is_err());
        match first_err.err().unwrap() {
            OsmPbfParseError::OsmPbf(osmpbf::Error::Io { .. }) => {
                // Good: we got an I/O error from the missing file
            }
            other => panic!("Expected I/O error from missing file, got {:?}", other),
        }
    }

    // ----------------------------------------------------------------------
    // 3) Test valid minimal pbf => we read exactly one address => no housenumber => aggregator is unused
    // ----------------------------------------------------------------------
    #[test]
    fn test_addresses_from_pbf_file_with_house_numbers_valid_no_housenumber() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("test_db_3")).unwrap();

        // Create a minimal .pbf => city="Baltimore", street="North Ave", postcode="21201" => no housenumber
        let pbf_path = tmp.path().join("test_no_hn.osm.pbf");
        create_minimal_address_pbf(
            &pbf_path,
            "Baltimore",
            "North Ave",
            "21201",
            None,
            1001,
        )
        .unwrap();

        let iter = addresses_from_pbf_file_with_house_numbers(pbf_path, region, db.clone())
            .expect("should produce an iterator");
        let results: Vec<_> = iter.collect();
        assert_eq!(results.len(), 1, "One node => one address");
        let addr_result = results[0].as_ref().unwrap();
        assert_eq!(addr_result.city().unwrap().name(), "baltimore");
        assert_eq!(addr_result.street().unwrap().name(), "north ave");
        assert_eq!(addr_result.postcode().unwrap().code(), "21201");

        // aggregator was never used => no house number => let's see if aggregator got stored in DB or not
        // The aggregator is presumably empty => store is a no-op. We'll confirm there's no HNR: key in the DB
        let db_guard = db.lock().unwrap();
        let possible_key = format!("HNR:{}:{}", region.abbreviation(), "north ave");
        let hnr = db_guard.get(possible_key).unwrap();
        assert!(hnr.is_none(), "No housenumber => aggregator empty => no DB entry");
    }

    // ----------------------------------------------------------------------
    // 4) Test valid minimal pbf => includes housenumber => aggregator is used => stored in DB
    // ----------------------------------------------------------------------
    #[test]
    fn test_addresses_from_pbf_file_with_house_numbers_valid_with_housenumber() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("test_db_4")).unwrap();

        let pbf_path = tmp.path().join("test_with_hn.osm.pbf");
        // We'll store housenumber="100-110"
        create_minimal_address_pbf(
            &pbf_path,
            "Calverton",
            "Catlett Road",
            "20138-9997",
            Some("100-110"),
            2002,
        )
        .unwrap();

        let iter = addresses_from_pbf_file_with_house_numbers(pbf_path, region, db.clone())
            .expect("iterator must be produced");
        let results: Vec<_> = iter.collect();
        assert_eq!(results.len(), 1, "One node => one address result");
        let addr_res = results[0].as_ref().unwrap();
        assert_eq!(addr_res.city().unwrap().name(), "calverton");
        assert_eq!(addr_res.street().unwrap().name(), "catlett road");
        assert_eq!(addr_res.postcode().unwrap().code(), "20138-9997");

        // aggregator => must have house number range => [100..=110]
        // it should be stored in the DB as "HNR:VA:catlett road" (or if abbreviation is "US", adjust accordingly).
        let db_guard = db.lock().unwrap();
        let hnr_key = format!("HNR:{}:{}", region.abbreviation(), "catlett road");
        let hnr_val_opt = db_guard.get(hnr_key.as_bytes()).unwrap();
        assert!(hnr_val_opt.is_some(), "We expect aggregator was stored => found DB entry for housenumber range");

        // If we decode that cbor, we should see a CompressedList<HouseNumberRange> with 1 item => [100..110]
        // We'll do a quick check:
        let hnr_bytes = hnr_val_opt.unwrap();
        let clist_res: serde_cbor::Result<crate::compressed_list::CompressedList<HouseNumberRange>> =
            serde_cbor::from_slice(&hnr_bytes);
        assert!(clist_res.is_ok());
        let clist = clist_res.unwrap();
        let items = clist.items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].start(), &100);
        assert_eq!(items[0].end(), &110);
    }

    // ----------------------------------------------------------------------
    // 5) Test a corrupted `.pbf` => iterator yields an error mid-parse
    // ----------------------------------------------------------------------
    #[test]
    fn test_addresses_from_pbf_file_with_house_numbers_corrupted_pbf() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("test_db_5")).unwrap();

        // We'll create a file with random bytes => not a valid .pbf
        let pbf_path = tmp.path().join("corrupt.osm.pbf");
        {
            let mut file = std::fs::File::create(&pbf_path).unwrap();
            file.write_all(b"This is definitely not a real PBF").unwrap();
        }

        let iter = addresses_from_pbf_file_with_house_numbers(pbf_path, region, db)
            .expect("iterator is created");
        let mut results_iter = iter.into_iter();

        let first = results_iter.next();
        assert!(first.is_some());
        let first_err = first.unwrap();
        assert!(first_err.is_err());
        match first_err.err().unwrap() {
            OsmPbfParseError::OsmPbf(osmpbf::Error::Pbf(_)) => {
                // good, parse error from corrupted data
            }
            other => panic!("Expected OsmPbf(PbfError) for corruption, got {:?}", other),
        }
        // No more items
        let second = results_iter.next();
        assert!(second.is_none());
    }

    // ----------------------------------------------------------------------
    // 6) Test multiple addresses from the same pbf => aggregator merges
    // ----------------------------------------------------------------------
    // We'll do a pbf with 2 nodes:
    //   #1 => city=Rockville, street=Rockville Pike, postcode=20850, housenumber=100-110
    //   #2 => city=Rockville, street=Rockville Pike, postcode=20850, housenumber=120-130
    // aggregator => merges into 2 subranges => [100..110], [120..130]. Both stored in DB.
    #[test]
    fn test_addresses_from_pbf_file_with_house_numbers_multiple_nodes() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("test_db_6")).unwrap();

        let pbf_path = tmp.path().join("multi_hn.osm.pbf");
        {
            // We'll manually create a small PBF with 2 nodes by reusing the "create_minimal_address_pbf" approach
            // or we do it ourselves with 2 nodes. For brevity, let's do 2 calls => then combine them. 
            // That might be complicated. We'll do a single custom approach:
            let mut header_block = crate::proto::osmformat::HeaderBlock::new();
            {
                let mut bbox = crate::proto::osmformat::HeaderBBox::new();
                bbox.set_left(-77_000_000_000);
                bbox.set_right(-76_000_000_000);
                bbox.set_top(39_000_000_000);
                bbox.set_bottom(38_000_000_000);
                header_block.bbox = protobuf::MessageField::from_option(Some(bbox));
                header_block.required_features.push("OsmSchema-V0.6".to_string());
                header_block.required_features.push("DenseNodes".to_string());
            }
            let header_bytes = header_block.write_to_bytes().unwrap();

            let mut h_blob = crate::proto::fileformat::Blob::new();
            h_blob.set_raw(header_bytes.clone());
            h_blob.set_raw_size(header_bytes.len() as i32);
            let h_blob_bytes = h_blob.write_to_bytes().unwrap();

            let mut h_bhdr = crate::proto::fileformat::BlobHeader::new();
            h_bhdr.set_type("OSMHeader".to_string());
            h_bhdr.set_datasize(h_blob_bytes.len() as i32);
            let h_bhdr_bytes = h_bhdr.write_to_bytes().unwrap();

            let mut block = crate::proto::osmformat::PrimitiveBlock::new();
            let mut s_table = crate::proto::osmformat::StringTable::new();
            // index=0 => ""
            s_table.s.push(b"".to_vec());
            // Next, push all relevant address keys/vals
            // city=rockville => "addr:city","Rockville"
            s_table.s.push(b"addr:city".to_vec());    // idx=1
            s_table.s.push(b"Rockville".to_vec());    // idx=2
            // street=rockville pike => "addr:street","Rockville Pike"
            s_table.s.push(b"addr:street".to_vec());  // idx=3
            s_table.s.push(b"Rockville Pike".to_vec()); // idx=4
            // pc=20850 => "addr:postcode","20850"
            s_table.s.push(b"addr:postcode".to_vec()); // idx=5
            s_table.s.push(b"20850".as_bytes().to_vec()); // idx=6

            // housenumber => we store two => but let's do them per Node:
            // We'll do indices for node #1 => "addr:housenumber","100-110"
            s_table.s.push(b"addr:housenumber".to_vec()); // idx=7
            s_table.s.push(b"100-110".as_bytes().to_vec()); // idx=8
            // for node #2 => "120-130"
            s_table.s.push(b"120-130".as_bytes().to_vec()); // idx=9

            block.stringtable = protobuf::MessageField::from_option(Some(s_table));
            block.set_granularity(100);
            block.set_lat_offset(0);
            block.set_lon_offset(0);

            let mut group = crate::proto::osmformat::PrimitiveGroup::new();

            // Node #1 => id=1001 => keys: city/street/postcode/housenumber(100-110)
            {
                let mut n1 = crate::proto::osmformat::Node::new();
                n1.set_id(1001);
                // lat/lon near 39.0/-77.0
                n1.set_lat((39.0 * 1e9) as i64 / 100);
                n1.set_lon((-77.0 * 1e9) as i64 / 100);

                // keys => [1,3,5,7], vals => [2,4,6,8]
                n1.keys.push(1); n1.vals.push(2);  // city => rockville
                n1.keys.push(3); n1.vals.push(4);  // street => rockville pike
                n1.keys.push(5); n1.vals.push(6);  // pc => 20850
                n1.keys.push(7); n1.vals.push(8);  // hn => 100-110
                group.nodes.push(n1);
            }
            // Node #2 => id=1002 => same city/street/postcode => housenumber=120-130
            {
                let mut n2 = crate::proto::osmformat::Node::new();
                n2.set_id(1002);
                n2.set_lat((39.05 * 1e9) as i64 / 100);
                n2.set_lon((-77.05 * 1e9) as i64 / 100);

                // keys => [1,3,5,7], but housenumber => index=9
                n2.keys.push(1); n2.vals.push(2);  // city => rockville
                n2.keys.push(3); n2.vals.push(4);  // street => rockville pike
                n2.keys.push(5); n2.vals.push(6);  // pc => 20850
                n2.keys.push(7); n2.vals.push(9);  // hn => 120-130
                group.nodes.push(n2);
            }

            block.primitivegroup.push(group);
            let block_bytes = block.write_to_bytes().unwrap();

            let mut data_blob = crate::proto::fileformat::Blob::new();
            data_blob.set_raw(block_bytes.clone());
            data_blob.set_raw_size(block_bytes.len() as i32);
            let data_blob_bytes = data_blob.write_to_bytes().unwrap();

            let mut data_blobheader = crate::proto::fileformat::BlobHeader::new();
            data_blobheader.set_type("OSMData".to_string());
            data_blobheader.set_datasize(data_blob_bytes.len() as i32);
            let data_blobheader_bytes = data_blobheader.write_to_bytes().unwrap();

            let mut f = std::fs::File::create(&pbf_path).unwrap();
            // header
            f.write_all(&(h_bhdr_bytes.len() as u32).to_be_bytes()).unwrap();
            f.write_all(&h_bhdr_bytes).unwrap();
            f.write_all(&h_blob_bytes).unwrap();
            // data
            f.write_all(&(data_blobheader_bytes.len() as u32).to_be_bytes()).unwrap();
            f.write_all(&data_blobheader_bytes).unwrap();
            f.write_all(&data_blob_bytes).unwrap();
        }

        let iter = addresses_from_pbf_file_with_house_numbers(pbf_path, region, db.clone())
            .expect("iterator is produced");
        let results: Vec<_> = iter.collect();
        assert_eq!(results.len(), 2, "We have 2 nodes => 2 addresses");
        for (i, item) in results.iter().enumerate() {
            let addr = item.as_ref().unwrap();
            assert_eq!(addr.city().unwrap().name(), "rockville");
            assert_eq!(addr.street().unwrap().name(), "rockville pike");
            assert_eq!(addr.postcode().unwrap().code(), "20850");
            match i {
                0 => {
                    // first => 100-110
                }
                1 => {
                    // second => 120-130
                }
                _ => {}
            }
        }

        // aggregator => merges into 2 subranges => [100..110], [120..130]. 
        // Let's see if it's stored in DB => key= HNR:MD:rockville pike or HNR:US:..., depends on abbreviation
        let db_guard = db.lock().unwrap();
        let abbr = region.abbreviation();
        let key_str = format!("HNR:{}:{}", abbr, "rockville pike");
        let hnr_opt = db_guard.get(&key_str).unwrap();
        assert!(hnr_opt.is_some(), "Should have aggregator data stored");
        // decode:
        let hnr_bytes = hnr_opt.unwrap();
        let clist: crate::compressed_list::CompressedList<HouseNumberRange> =
            serde_cbor::from_slice(&hnr_bytes).unwrap();
        let items = clist.items();
        assert_eq!(items.len(), 2, "We expect 2 subranges in aggregator");
        assert_eq!(items[0].start(), &100);
        assert_eq!(items[0].end(), &110);
        assert_eq!(items[1].start(), &120);
        assert_eq!(items[1].end(), &130);
    }
}
