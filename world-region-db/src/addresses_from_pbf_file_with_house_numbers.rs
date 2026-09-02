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
pub fn addresses_from_pbf_file_with_house_numbers<I:StorageInterface + 'static>(
    path:         PathBuf,
    world_region: WorldRegion,
    db:           Arc<Mutex<I>>,
) -> Result<impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>, OsmPbfParseError> {
    trace!("addresses_from_pbf_file_with_house_numbers: Invoked with path={:?}, region={:?}", path, world_region);

    let country = try_resolve_country(world_region)?;
    trace!("addresses_from_pbf_file_with_house_numbers: Resolved country={:?}", country);

    let (tx, rx) = create_address_stream_channel();
    trace!("addresses_from_pbf_file_with_house_numbers: Created sync_channel for address streaming");

    let dbc = db.clone();

    // Move ownership into background thread
    thread::spawn(move || {
        handle_pbf_house_number_extractor_in_thread(
            dbc, 
            path, 
            country, 
            world_region, 
            tx
        );
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

    /// A convenience function that creates a minimal .osm.pbf file containing:
    ///   - A single Node with complete address tags (addr:city/street/postcode).
    ///   - Optionally includes an addr:housenumber to test aggregator storage.
    fn create_minimal_address_pbf(
        path: &std::path::Path,
        city: &str,
        street: &str,
        postcode: &str,
        housenumber: Option<&str>,
        node_id: i64,
    ) -> std::io::Result<()> {
        info!("Creating minimal address PBF at {:?}, city='{}', street='{}', postcode='{}', housenumber='{:?}'",
            path, city, street, postcode, housenumber);

        // 1) Build OSMHeader
        let mut header_block = crate::proto::osmformat::HeaderBlock::new();
        {
            let mut bbox = crate::proto::osmformat::HeaderBBox::new();
            // Example bounding box around Northern California (roughly)
            bbox.set_left(-122_000_000_000);
            bbox.set_right(-121_000_000_000);
            bbox.set_top(38_000_000_000);
            bbox.set_bottom(37_000_000_000);
            header_block.bbox = protobuf::MessageField::from_option(Some(bbox));

            header_block.required_features.push("OsmSchema-V0.6".to_string());
            header_block.required_features.push("DenseNodes".to_string());
            debug!("OSMHeader bounding box set for minimal address PBF");
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
            s_table.s.push(b"addr:city".to_vec());          // idx=1
            s_table.s.push(city.as_bytes().to_vec());       // idx=2
            s_table.s.push(b"addr:street".to_vec());        // idx=3
            s_table.s.push(street.as_bytes().to_vec());     // idx=4
            s_table.s.push(b"addr:postcode".to_vec());      // idx=5
            s_table.s.push(postcode.as_bytes().to_vec());   // idx=6

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
            // lat/lon within bounding box => e.g. lat=37.500, lon=-121.500
            node.set_lat((37.5 * 1e9) as i64 / 100);
            node.set_lon((-121.5 * 1e9) as i64 / 100);

            // Insert keys/vals
            for k in &node_keys {
                node.keys.push(*k);
            }
            for v in &node_vals {
                node.vals.push(*v);
            }

            group.nodes.push(node);
            block.primitivegroup.push(group);
            debug!("Created a single Node with city='{}', street='{}', postcode='{}', housenumber='{:?}'",
                city, street, postcode, housenumber);
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
        file.write_all(&(header_blobheader_bytes.len() as u32).to_be_bytes())?;
        file.write_all(&header_blobheader_bytes)?;
        file.write_all(&header_blob_bytes)?;

        file.write_all(&(data_blobheader_bytes.len() as u32).to_be_bytes())?;
        file.write_all(&data_blobheader_bytes)?;
        file.write_all(&data_blob_bytes)?;

        info!("Minimal address PBF written successfully to {:?}", path);
        Ok(())
    }

    #[traced_test]
    fn test_addresses_from_pbf_file_with_house_numbers_unknown_region() {
        info!("Testing addresses_from_pbf_file_with_house_numbers with an unknown region (Guam)");
        let unknown_region: WorldRegion = USRegion::USTerritory(USTerritory::Guam).into();
        let tmp_db = Database::open(std::env::temp_dir().join("dummy_db_path")).unwrap();
        let result = addresses_from_pbf_file_with_house_numbers(
            "some_file.osm.pbf".into(),
            unknown_region,
            tmp_db,
        );

        assert!(result.is_ok(),
            "In current code, 'Guam' is not considered an error => remove or adapt this test if needed."
        );
    }

    #[traced_test]
    fn test_addresses_from_pbf_file_with_house_numbers_missing_file() {
        info!("Testing addresses_from_pbf_file_with_house_numbers for missing file (California region)");
        let known_region: WorldRegion = USRegion::UnitedState(UnitedState::California).into();
        let db = Database::open(std::env::temp_dir().join("dummy_db_2")).unwrap();
        let db_clone = db.clone();

        // We'll pass a path that doesn't exist.
        let missing_path = std::path::PathBuf::from("/path/does/not/exist/osm.pbf");

        let result_iter = addresses_from_pbf_file_with_house_numbers(
            missing_path,
            known_region,
            db_clone,
        )
        .expect("should succeed in returning an iterator, error is produced in background parse thread");

        let first = result_iter.into_iter().next();
        assert!(first.is_some());
        let first_err = first.unwrap();
        assert!(first_err.is_err());

        match first_err.err().unwrap() {
            OsmPbfParseError::OsmPbf(_) => {
                debug!("Got expected I/O-based parse error from missing file");
            }
            other => panic!("Expected I/O error from missing file, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_addresses_from_pbf_file_with_house_numbers_valid_no_housenumber() {
        info!("Testing addresses_from_pbf_file_with_house_numbers with no housenumber (California)");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::California).into();
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("test_db_3")).unwrap();

        // Create a minimal .pbf => city="Sunnyvale", street="Mathilda Ave", postcode="94085", no housenumber
        let pbf_path = tmp.path().join("test_no_hn.osm.pbf");
        create_minimal_address_pbf(
            &pbf_path,
            "Sunnyvale",
            "Mathilda Ave",
            "94085",
            None,
            1001,
        )
        .unwrap();

        let iter = addresses_from_pbf_file_with_house_numbers(pbf_path, region, db.clone())
            .expect("should produce an iterator");
        let results: Vec<_> = iter.collect();
        assert_eq!(results.len(), 1, "One node => one address");
        let addr_result = results[0].as_ref().unwrap();

        assert_eq!(addr_result.city().name(), "sunnyvale");
        assert_eq!(addr_result.street().name(), "mathilda ave");
        assert_eq!(addr_result.postal_code().code(), "94085");

        let db_guard = db.lock().unwrap();
        let possible_key = format!("HNR:{}:{}", region.abbreviation(), "mathilda ave");
        let hnr = db_guard.get(possible_key).unwrap();
        assert!(hnr.is_none(), "No housenumber => aggregator empty => no DB entry");
    }

    #[traced_test]
    fn test_addresses_from_pbf_file_with_house_numbers_valid_with_housenumber() {
        info!("Testing addresses_from_pbf_file_with_house_numbers including housenumber (California)");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::California).into();
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("test_db_4")).unwrap();

        let pbf_path = tmp.path().join("test_with_hn.osm.pbf");
        // We'll store housenumber="100-110"
        create_minimal_address_pbf(
            &pbf_path,
            "Palo Alto",
            "El Camino Real",
            "94301",
            Some("100-110"),
            2002,
        )
        .unwrap();

        let iter = addresses_from_pbf_file_with_house_numbers(pbf_path, region, db.clone())
            .expect("iterator must be produced");
        let results: Vec<_> = iter.collect();
        assert_eq!(results.len(), 1, "One node => one address result");
        let addr_res = results[0].as_ref().unwrap();

        assert_eq!(addr_res.city().name(), "palo alto");
        assert_eq!(addr_res.street().name(), "el camino real");
        assert_eq!(addr_res.postal_code().code(), "94301");

        let db_guard = db.lock().unwrap();
        let hnr_key = format!("HNR:{}:{}", region.abbreviation(), "el camino real");
        let hnr_val_opt = db_guard.get(hnr_key.as_bytes()).unwrap();
        assert!(
            hnr_val_opt.is_some(),
            "Expect aggregator was stored => found DB entry for housenumber range"
        );

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

    #[traced_test]
    fn test_addresses_from_pbf_file_with_house_numbers_corrupted_pbf() {
        info!("Testing corrupted PBF scenario (California)");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::California).into();
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path().join("test_db_5")).unwrap();

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
            OsmPbfParseError::OsmPbf(_) => {
                debug!("Got parse error from corrupted file, as expected");
            }
            other => panic!("Expected OsmPbf parse error for corruption, got {:?}", other),
        }
        let second = results_iter.next();
        assert!(second.is_none());
    }

    /// Creates a .osm.pbf near Northern California, containing exactly two Node elements:
    ///   - Both have city="Santa Clara", street="Homestead Rd", postcode="95050"
    ///   - Housenumbers: "100-110" and "120-130"
    ///   - lat/lon inside the bounding box so neither node is discarded.
    fn create_santaclara_2nodes_pbf(path: &std::path::Path) -> std::io::Result<()> {
        info!("Creating two-node PBF fixture at {:?}", path);

        // 1) Prepare OSMHeader with bounding box ~ [-122..-121, 38..37]
        let mut header_block = crate::proto::osmformat::HeaderBlock::new();
        {
            let mut bbox = crate::proto::osmformat::HeaderBBox::new();
            bbox.set_left(-122_000_000_000);
            bbox.set_right(-121_000_000_000);
            bbox.set_top(38_000_000_000);
            bbox.set_bottom(37_000_000_000);

            header_block.bbox = protobuf::MessageField::from_option(Some(bbox));
            header_block.required_features.push("OsmSchema-V0.6".to_string());
            header_block.required_features.push("DenseNodes".to_string());
            debug!("OSMHeader bounding box prepared for Santa Clara area");
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

        // 2) Build a PrimitiveBlock with two Nodes
        let mut block = crate::proto::osmformat::PrimitiveBlock::new();
        {
            let mut s_table = crate::proto::osmformat::StringTable::new();
            // index=0 => ""
            // We'll define:
            // 1 => "addr:city"
            // 2 => "Santa Clara"
            // 3 => "addr:street"
            // 4 => "Homestead Rd"
            // 5 => "addr:postcode"
            // 6 => "95050"
            // 7 => "addr:housenumber"
            // 8 => "100-110"
            // 9 => "120-130"

            s_table.s.push(b"".to_vec());
            s_table.s.push(b"addr:city".to_vec());      // idx=1
            s_table.s.push(b"Santa Clara".to_vec());    // idx=2
            s_table.s.push(b"addr:street".to_vec());    // idx=3
            s_table.s.push(b"Homestead Rd".to_vec());   // idx=4
            s_table.s.push(b"addr:postcode".to_vec());  // idx=5
            s_table.s.push(b"95050".to_vec());          // idx=6
            s_table.s.push(b"addr:housenumber".to_vec()); // idx=7
            s_table.s.push(b"100-110".to_vec());        // idx=8
            s_table.s.push(b"120-130".to_vec());        // idx=9

            block.stringtable = protobuf::MessageField::from_option(Some(s_table));
            block.set_granularity(100);
            block.set_lat_offset(0);
            block.set_lon_offset(0);

            let mut group = crate::proto::osmformat::PrimitiveGroup::new();

            // Node #1 => housenumber=100-110
            {
                let mut n1 = crate::proto::osmformat::Node::new();
                n1.set_id(1001);
                // lat/lon: inside bounding box => e.g. lat=37.350, lon=-121.980
                n1.set_lat((37.350 * 1e9) as i64 / 100);
                n1.set_lon((-121.980 * 1e9) as i64 / 100);

                // city => "Santa Clara", street => "Homestead Rd", pc => "95050", housenumber => "100-110"
                n1.keys.push(1); n1.vals.push(2);
                n1.keys.push(3); n1.vals.push(4);
                n1.keys.push(5); n1.vals.push(6);
                n1.keys.push(7); n1.vals.push(8);

                group.nodes.push(n1);
            }

            // Node #2 => housenumber=120-130
            {
                let mut n2 = crate::proto::osmformat::Node::new();
                n2.set_id(1002);
                // lat=37.360, lon=-121.980 => still inside bounding box
                n2.set_lat((37.360 * 1e9) as i64 / 100);
                n2.set_lon((-121.980 * 1e9) as i64 / 100);

                n2.keys.push(1); n2.vals.push(2);
                n2.keys.push(3); n2.vals.push(4);
                n2.keys.push(5); n2.vals.push(6);
                n2.keys.push(7); n2.vals.push(9);

                group.nodes.push(n2);
            }

            block.primitivegroup.push(group);
            debug!("Created two Node elements for Santa Clara with two distinct house number ranges");
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

        // 3) Write to disk
        let mut f = std::fs::File::create(path)?;
        // header
        f.write_all(&(header_blobheader_bytes.len() as u32).to_be_bytes())?;
        f.write_all(&header_blobheader_bytes)?;
        f.write_all(&header_blob_bytes)?;

        // data
        f.write_all(&(data_blobheader_bytes.len() as u32).to_be_bytes())?;
        f.write_all(&data_blobheader_bytes)?;
        f.write_all(&data_blob_bytes)?;

        info!("Two-node PBF fixture for Santa Clara written successfully to {:?}", path);
        Ok(())
    }

    #[traced_test]
    fn test_addresses_from_pbf_file_with_house_numbers_multiple_nodes() {
        info!("Testing addresses_from_pbf_file_with_house_numbers with multiple Node elements (California)");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::California).into();
        let tmp_dir = TempDir::new().unwrap();
        let db = Database::open(tmp_dir.path().join("test_db_6")).unwrap();

        let pbf_path = tmp_dir.path().join("multi_hn.osm.pbf");
        create_santaclara_2nodes_pbf(&pbf_path)
            .expect("Failed to create multi-node .osm.pbf fixture");

        let iter = addresses_from_pbf_file_with_house_numbers(pbf_path, region, db.clone())
            .expect("iterator is produced without immediate error");
        let results: Vec<_> = iter.collect();

        assert_eq!(results.len(), 2, "We have 2 distinct nodes => 2 addresses");
        for (i, item) in results.iter().enumerate() {
            let addr = item.as_ref().unwrap();
            assert_eq!(addr.city().name(), "santa clara");
            assert_eq!(addr.street().name(), "homestead rd");
            assert_eq!(addr.postal_code().code(), "95050");
            match i {
                0 => debug!("Node #1 => housenumber=100-110"),
                1 => debug!("Node #2 => housenumber=120-130"),
                _ => unreachable!("Should only have 2 nodes"),
            }
        }

        // aggregator => merges 2 subranges => [100..110], [120..130]
        let db_guard = db.lock().unwrap();
        let abbr = region.abbreviation();
        let hnr_key = format!("HNR:{}:{}", abbr, "homestead rd");
        let hnr_val_opt = db_guard.get(&hnr_key).unwrap();
        assert!(
            hnr_val_opt.is_some(),
            "Should have aggregator data stored for 2 subranges"
        );

        let raw = hnr_val_opt.unwrap();
        let clist: crate::compressed_list::CompressedList<HouseNumberRange> =
            serde_cbor::from_slice(&raw).unwrap();
        let items = clist.items();
        assert_eq!(items.len(), 2, "We expect 2 subranges in aggregator");
        assert_eq!(items[0].start(), &100);
        assert_eq!(items[0].end(), &110);
        assert_eq!(items[1].start(), &120);
        assert_eq!(items[1].end(), &130);
    }
}
