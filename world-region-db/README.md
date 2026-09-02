# world-region-db

A decentralizable geographic database built on top of WorldRegion and WorldAddress primitives, using Open Street Map PBF data as a source.

`world-region-db` is a Rust library for managing, storing, and validating
city-street-postal code relationships across various world regions. It uses
RocksDB as its storage backend and supports house-number range aggregation to
handle geospatial data found in OSM (OpenStreetMap) PBF files.

## Features

1. **Parsing OSM PBF Files**  
   - Uses [`osmpbf`] to parse OSM elements (nodes, ways, relations, dense nodes).
   - Converts them into [`AddressRecord`] structures, capturing `addr:city`,
     `addr:street`, `addr:postcode`, and optionally `addr:housenumber`.

2. **House-Number Ranges**  
   - Aggregates sequential house-number subranges (e.g. `100-110`) for each street.
   - Merges new subranges with existing ranges in the database, storing them under
     keys like `HNR:<region_abbr>:<street_name>`.

3. **City/Street/Postal-Code Indexing**  
   - Builds in-memory indexes from [`RegionalRecords`], bridging:
     - `C2Z` (City→PostalCode),
     - `Z2C` (PostalCode→City),
     - `C2S` (City→Street),
     - `S2C` (Street→City),
     - `S2Z` (Street→PostalCode),
     - `S:<region_abbr>:<postal>` etc.
   - Can optionally write these to RocksDB.

4. **Validation**  
   - Implements `validate_with(...)` for [`WorldAddress`], verifying city-street-postcode
     consistency via your stored data:
     - Checks city presence under `Z2C:<region>:<postal>`
     - Checks street presence under `S:<region>:<postal>`
     - Ensures the city→street membership is consistent.

5. **Chained Iteration**  
   - Gathers multiple `.osm.pbf` files and produces a single iterator of
     [`WorldAddress`] items, streaming them from multiple threads or files.

6. **Mock Data**  
   - Provides region-specific mock addresses for tests: Florida, Texas, Tennessee, etc.
     - Example: *Maryland* records unify around Baltimore, Bethesda, Rockville.
     - Example: *Virginia* records unify around Calverton or Virginia Beach.

7. **Extensions & Hooks**  
   - Provides many extension traits (like `WriteStreetsToRegionAndCity`, `LoadHouseNumberRanges`)
     so custom DB types can be used.

## Getting Started

1. **Add the dependency**:

```toml
   [dependencies]
   world-region-db = "*"
```

## Open a Database:

```rust
let db_path = "/path/to/rocksdb";
let db = Database::open(db_path).expect("Failed to open DB");
let data_access = DataAccess::with_db(db);
```

## Parse & Store:

```rust
let region = USRegion::UnitedState(UnitedState::Maryland).into();
let pbf_dir = std::path::PathBuf::from("./pbf_files");
// Download & parse
download_and_parse_region(&region, &pbf_dir, &mut *db.lock().unwrap(), true).await
    .expect("Failed to parse region");
```

## Validate Addresses:

```rust
let example_address = WorldAddressBuilder::default()
    .region(region)
    .postal_code(PostalCode::new(Country::USA, "21201").unwrap())
    .city(CityName::new("Baltimore").unwrap())
    .street(StreetName::new("Howard Street").unwrap())
    .build()
    .unwrap();

if let Err(invalid) = example_address.validate_with(&data_access) {
    eprintln!("Address is invalid: {:?}", invalid);
} else {
    println!("Address validated successfully!");
}
```

## List & Process:

```rust
let all_iter = list_all_addresses_in_pbf_dir(pbf_dir, db).unwrap();
let result = process_and_validate_addresses(all_iter, &data_access)
    .expect("Validation process failed");
println!("All addresses valid? => {}", result);
```

## Directory Layout

address_record.rs: Core struct for storing optional (city, street, postcode).

regional_records.rs: Holds an entire region’s addresses plus house-number aggregator.

house_number_aggregator.rs: Manages merging & storing multiple [start..end] subranges.

indexing.rs: Creates InMemoryIndexes bridging city/postcode/street relationships.

validate_*.rs: Multiple modules checking city→postal_code→street consistency.

## Tests
Unit Tests: In-module #[cfg(test)] blocks that verify indexing, aggregator merging, etc.

## Integration: tests/ directory for higher-level flows (e.g. parse a .pbf, store data,
then validate addresses).

## Contributing
Fork & clone the repo.

Use a dev RocksDB or a memory-based mock for quick local tests.

Submit PRs with descriptive commit messages and thorough test coverage.

## License
This project is licensed under the MIT license. Contributions are welcomed!

Enjoy building robust region/city/street/postal_code integrations with world-region-db!
