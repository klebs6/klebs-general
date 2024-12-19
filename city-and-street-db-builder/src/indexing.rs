crate::ix!();

pub type RegionToPostalCodeToStreetsMap = BTreeMap<WorldRegion, BTreeMap<PostalCode, BTreeSet<StreetName>>>;
pub type PostalCodeToStreetMap          = BTreeMap<PostalCode, BTreeSet<StreetName>>;
pub type PostalCodeToCityMap            = BTreeMap<PostalCode, BTreeSet<CityName>>;
pub type CityToPostalCodeMap            = BTreeMap<CityName, BTreeSet<PostalCode>>;
pub type CityToStreetMap                = BTreeMap<CityName, BTreeSet<StreetName>>;
pub type StreetToPostalCodeMap          = BTreeMap<StreetName, BTreeSet<PostalCode>>;
pub type StreetToCitiesMap              = BTreeMap<StreetName, BTreeSet<CityName>>;

#[derive(Getters,Setters)]
#[getset(get="pub",set="pub")]
pub struct InMemoryIndexes {
    region_postal_code_streets: RegionToPostalCodeToStreetsMap, // State -> PostalCode -> Streets
    postal_code_cities:         PostalCodeToCityMap,            // PostalCode -> Cities
    city_postal_codes:          CityToPostalCodeMap,            // City -> PostalCodes
    city_streets:               CityToStreetMap,                // City -> Streets
    street_postal_codes:        StreetToPostalCodeMap,          // Street -> PostalCodes
    street_cities:              StreetToCitiesMap,              // Street -> Cities
}

impl InMemoryIndexes {

    pub fn postal_code_to_street_map_for_region(&self, region: &WorldRegion) -> Option<&PostalCodeToStreetMap> {
        self.region_postal_code_streets.get(region)
    }
}

impl From<&RegionalRecords> for InMemoryIndexes {

    /// Build indexes given a set of address records and a region name.
    fn from(regional_records: &RegionalRecords) -> InMemoryIndexes 
    {
        let region  = regional_records.region();
        let records = regional_records.records();

        tracing::info!("building indices with {} records",records.len());

        let mut region_postal_code_streets = BTreeMap::new();
        let mut postal_code_cities         = BTreeMap::new();
        let mut city_postal_codes          = BTreeMap::new();
        let mut city_streets               = BTreeMap::new();
        let mut street_postal_codes        = BTreeMap::new();
        let mut street_cities              = BTreeMap::new();

        region_postal_code_streets.insert(region.clone(), BTreeMap::new());

        let country = regional_records.country();

        for rec in records {

            if rec.is_empty() {
                continue;
            }

            // Construct typed objects if possible
            let city_obj        = rec.city();
            let street_obj      = rec.street();
            let postal_code_obj = rec.postcode();

            // State->PostalCode->Street
            if let (Some(postal_code), Some(st)) = (postal_code_obj.clone(), street_obj.clone()) {
                region_postal_code_streets
                    .get_mut(&region).unwrap()
                    .entry(postal_code)
                    .or_insert_with(BTreeSet::new)
                    .insert(st);
            }

            // PostalCode->Cities
            if let (Some(postal_code), Some(ct)) = (postal_code_obj.clone(), city_obj.clone()) {
                postal_code_cities
                    .entry(postal_code)
                    .or_insert_with(BTreeSet::new)
                    .insert(ct);
            }

            // City->PostalCode
            if let (Some(ct), Some(postal_code)) = (city_obj.clone(), postal_code_obj.clone()) {
                city_postal_codes
                    .entry(ct)
                    .or_insert_with(BTreeSet::new)
                    .insert(postal_code);
            }

            // City->Streets
            if let (Some(ct), Some(st)) = (city_obj.clone(), street_obj.clone()) {
                city_streets
                    .entry(ct)
                    .or_insert_with(BTreeSet::new)
                    .insert(st);
            }

            // Street->PostalCodes
            if let (Some(st), Some(postal_code)) = (street_obj.clone(), postal_code_obj.clone()) {
                street_postal_codes
                    .entry(st)
                    .or_insert_with(BTreeSet::new)
                    .insert(postal_code);
            }

            // Street->Cities
            if let (Some(st), Some(ct)) = (street_obj.clone(), city_obj.clone()) {
                street_cities
                    .entry(st)
                    .or_insert_with(BTreeSet::new)
                    .insert(ct);
            }
        }

        InMemoryIndexes {
            region_postal_code_streets,
            postal_code_cities,
            city_postal_codes,
            city_streets,
            street_postal_codes,
            street_cities,
        }
    }
}

/// Tests for InMemoryIndexes
#[cfg(test)]
mod in_memory_indexes_tests {
    use super::*;

    #[test]
    fn build_inmemory_indexes_from_mock_records() {

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let rr                  = RegionalRecords::mock_for_region(&region);
        let indexes             = InMemoryIndexes::from(&rr);

        // Check that indexes contain the expected data:
        // From mock(0): Baltimore, North Avenue, 21201, etc.
        let postal_code_map   = indexes.postal_code_to_street_map_for_region(&region).unwrap();
        let streets_for_21201 = postal_code_map.get(&PostalCode::new(Country::USA,"21201").unwrap()).unwrap();

        assert!(streets_for_21201.contains(&StreetName::new("North Avenue").unwrap()));
    }
}
