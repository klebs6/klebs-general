// ---------------- [ File: src/collect_address_and_housenumber_data.rs ]
crate::ix!();

/// Iterates through all OSM elements in the file, extracting both addresses
/// and house‐number ranges. The results are appended to `addresses` and
/// `street_hnr_map`.
pub fn collect_address_and_housenumber_data(
    reader: osmpbf::ElementReader<std::io::BufReader<std::fs::File>>,
    country: &Country,
    addresses: &mut Vec<AddressRecord>,
    street_hnr_map: &mut HouseNumberAggregator,
) -> Result<(), OsmPbfParseError> {
    trace!("collect_address_and_housenumber_data: starting iteration");

    let mut count = 0usize;
    reader.for_each(|element| {
        process_single_osm_element(&element, country, addresses, street_hnr_map)
            .expect("could not process single osm element");
        count += 1;

        // Periodic log to observe progress
        if count % 100_000 == 0 {
            info!(
                "collect_address_and_housenumber_data: processed {} elements so far...",
                count
            );
        }
    })?;

    debug!("collect_address_and_housenumber_data: complete. total elements={}", count);
    Ok(())
}

#[cfg(test)]
mod collect_address_and_housenumber_data_tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// Creates a minimal `.pbf` with no nodes/elements => yields 0 addresses
    fn create_empty_pbf(path: &std::path::Path) -> std::io::Result<()> {
        File::create(path).map(|_f| ())
    }

    /// Writes random/corrupted bytes to force a parse error
    fn create_corrupted_pbf(path: &std::path::Path) -> std::io::Result<()> {
        let mut f = File::create(path)?;
        f.write_all(b"not a real pbf file")?;
        Ok(())
    }

    /// Creates a `.pbf` with 2 nodes:
    ///    - Node #1 => has complete `addr:city`, `addr:street`, `addr:postcode`.
    ///    - Node #2 => has partial missing `addr:street` => skip in addresses.
    /// Optionally set `addr:housenumber` => aggregator usage in Node #1.
    ///
    /// For brevity, we only build a single `PrimitiveBlock` with 2 nodes.
    fn create_test_pbf_two_nodes(path: &std::path::Path, housenumber: Option<&str>) -> std::io::Result<()> {
        // 1) OSMHeader
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

        // 2) PrimitiveBlock => 2 nodes
        let mut block = crate::proto::osmformat::PrimitiveBlock::new();
        {
            let mut s_table = crate::proto::osmformat::StringTable::new();
            // index=0 => ""
            s_table.s.push(b"".to_vec());

            // index=1 => "addr:city"
            // index=2 => "TestCity"
            // index=3 => "addr:street"
            // index=4 => "TestStreet"
            // index=5 => "addr:postcode"
            // index=6 => "99999"
            s_table.s.push(b"addr:city".to_vec());     // idx=1
            s_table.s.push(b"TestCity".to_vec());      // idx=2
            s_table.s.push(b"addr:street".to_vec());   // idx=3
            s_table.s.push(b"TestStreet".to_vec());    // idx=4
            s_table.s.push(b"addr:postcode".to_vec()); // idx=5
            s_table.s.push(b"99999".to_vec());         // idx=6

            let mut node1_keys = vec![1, 3, 5];
            let mut node1_vals = vec![2, 4, 6];

            // If housenumber => add "addr:housenumber" => "NNN"
            if let Some(hn) = housenumber {
                let next_idx = s_table.s.len() as u32; // e.g. 7 => "addr:housenumber", 8 => value
                s_table.s.push(b"addr:housenumber".to_vec()); // key
                s_table.s.push(hn.as_bytes().to_vec());       // value
                node1_keys.push(next_idx);
                node1_vals.push(next_idx + 1);
            }

            block.stringtable = protobuf::MessageField::from_option(Some(s_table));
            block.set_granularity(100);
            block.set_lat_offset(0);
            block.set_lon_offset(0);

            let mut group = crate::proto::osmformat::PrimitiveGroup::new();
            // Node #1 => city="TestCity", street="TestStreet", postcode="99999", optional housenumber
            {
                let mut n1 = crate::proto::osmformat::Node::new();
                n1.set_id(1001);
                // lat/lon near 38.5
                n1.set_lat((38.5 * 1e9) as i64 / 100);
                n1.set_lon((-76.5 * 1e9) as i64 / 100);

                for (k, v) in node1_keys.iter().zip(node1_vals.iter()) {
                    n1.keys.push(*k);
                    n1.vals.push(*v);
                }
                group.nodes.push(n1);
            }

            // Node #2 => city= "TestCity" only => missing `addr:street`
            {
                let mut n2 = crate::proto::osmformat::Node::new();
                n2.set_id(1002);
                n2.set_lat((38.6 * 1e9) as i64 / 100);
                n2.set_lon((-76.6 * 1e9) as i64 / 100);

                // city => "TestCity"
                n2.keys.push(1);
                n2.vals.push(2);
                // no street => skip
                // no postcode => skip
                group.nodes.push(n2);
            }

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

        // 3) Write everything
        let mut file = File::create(path)?;
        // Write header
        file.write_all(&(header_blobheader_bytes.len() as u32).to_be_bytes())?;
        file.write_all(&header_blobheader_bytes)?;
        file.write_all(&header_blob_bytes)?;
        // Write data
        file.write_all(&(data_blobheader_bytes.len() as u32).to_be_bytes())?;
        file.write_all(&data_blobheader_bytes)?;
        file.write_all(&data_blob_bytes)?;

        Ok(())
    }

    // -----------
    // Actual test suite
    // -----------
    #[traced_test]
    fn test_collect_address_and_housenumber_data_empty_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("empty.osm.pbf");
        create_empty_pbf(&path).unwrap();

        // Create an ElementReader
        let reader = osmpbf::ElementReader::from_path(&path).unwrap();
        let country = Country::USA;

        let region = example_region();
        let mut addresses = Vec::new();
        let mut aggregator = HouseNumberAggregator::new(&region);

        let res = collect_address_and_housenumber_data(reader, &country, &mut addresses, &mut aggregator);
        assert!(res.is_ok(), "Empty file => no parse error, just 0 elements");
        assert!(addresses.is_empty());
        assert!(aggregator.is_empty());
    }

    #[traced_test]
    fn test_collect_address_and_housenumber_data_corrupted_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("corrupt.osm.pbf");
        create_corrupted_pbf(&path).unwrap();

        let reader = osmpbf::ElementReader::from_path(&path).unwrap();
        let country = Country::USA;
        let region = example_region();
        let mut addresses = Vec::new();
        let mut aggregator = HouseNumberAggregator::new(&region);

        let res = collect_address_and_housenumber_data(reader, &country, &mut addresses, &mut aggregator);
        assert!(res.is_err(), "Corrupted => parse error => Err(...)");
        match res.err().unwrap() {
            OsmPbfParseError::OsmPbf(_) => {
                // Good => parse error from corrupted data
            }
            other => panic!("Expected parse error from corrupted data, got {:?}", other),
        }
        assert!(addresses.is_empty());
        assert!(aggregator.is_empty());
    }

    #[traced_test]
    fn test_collect_address_and_housenumber_data_two_nodes_no_housenumber() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("two_nodes.osm.pbf");
        create_test_pbf_two_nodes(&path, None).unwrap();

        let reader = osmpbf::ElementReader::from_path(&path).unwrap();
        let mut addresses = Vec::new();
        let region = example_region();
        let mut aggregator = HouseNumberAggregator::new(&region);
        let country = Country::USA;

        let result = collect_address_and_housenumber_data(reader, &country, &mut addresses, &mut aggregator);
        assert!(result.is_ok());

        // Node #1 => city="testcity", street="teststreet", pc="99999" => => 1 address
        // Node #2 => missing street => skip
        assert_eq!(addresses.len(), 1, "Should capture Node #1 as an address, skip #2");
        let rec = &addresses[0];
        assert_eq!(rec.city().as_ref().unwrap().name(), "testcity");
        assert_eq!(rec.street().as_ref().unwrap().name(), "teststreet");
        assert_eq!(rec.postcode().as_ref().unwrap().code(), "99999");

        // aggregator => no housenumber => aggregator empty
        assert!(aggregator.is_empty());
    }

    #[traced_test]
    fn test_collect_address_and_housenumber_data_two_nodes_with_housenumber() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("two_nodes_hn.osm.pbf");
        // Node #1 => city="TestCity", street="TestStreet", pc="99999", housenumber="100-110"
        // Node #2 => partial => skip
        create_test_pbf_two_nodes(&path, Some("100-110")).unwrap();

        let reader = osmpbf::ElementReader::from_path(&path).unwrap();
        let mut addresses = Vec::new();
        let region = example_region();
        let mut aggregator = HouseNumberAggregator::new(&region);
        let country = Country::USA;

        let res = collect_address_and_housenumber_data(reader, &country, &mut addresses, &mut aggregator);
        assert!(res.is_ok());

        // addresses => Node #1 => has city/street/postal => 1 address
        assert_eq!(addresses.len(), 1);
        let rec = &addresses[0];
        assert_eq!(rec.city().as_ref().unwrap().name(), "testcity");
        assert_eq!(rec.street().as_ref().unwrap().name(), "teststreet");
        assert_eq!(rec.postcode().as_ref().unwrap().code(), "99999");

        // aggregator => Node #1 => "100-110"
        assert_eq!(aggregator.len(), 1, "One street => aggregator entry");
        let street = StreetName::new("TestStreet").unwrap();
        let rng_vec = aggregator.get(&street).expect("street aggregator found");
        assert_eq!(rng_vec.len(), 1);
        let hnr = &rng_vec[0];
        assert_eq!(hnr.start(), &100);
        assert_eq!(hnr.end(), &110);
    }

    // If you want to test extremely large files or performance, you can do so by 
    // generating many nodes. But the above covers correctness thoroughly.
}
