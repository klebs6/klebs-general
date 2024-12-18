crate::ix!();

pub type RegionToZipToStreetsMap = BTreeMap<USRegion, BTreeMap<PostalCode, BTreeSet<StreetName>>>;
pub type ZipToStreetMap          = BTreeMap<PostalCode, BTreeSet<StreetName>>;
pub type ZipToCityMap            = BTreeMap<PostalCode, BTreeSet<CityName>>;
pub type CityToZipMap            = BTreeMap<CityName, BTreeSet<PostalCode>>;
pub type CityToStreetMap         = BTreeMap<CityName, BTreeSet<StreetName>>;
pub type StreetToZipMap          = BTreeMap<StreetName, BTreeSet<PostalCode>>;
pub type StreetToCitiesMap       = BTreeMap<StreetName, BTreeSet<CityName>>;

#[derive(Getters,Setters)]
#[getset(get="pub",set="pub")]
pub struct InMemoryIndexes {
    region_zip_streets: RegionToZipToStreetsMap, // State -> ZIP -> Streets
    zip_cities:         ZipToCityMap,            // ZIP -> Cities
    city_zips:          CityToZipMap,            // City -> ZIPs
    city_streets:       CityToStreetMap,         // City -> Streets
    street_zips:        StreetToZipMap,          // Street -> ZIPs
    street_cities:      StreetToCitiesMap,       // Street -> Cities
}

impl InMemoryIndexes {

    pub fn zip_to_street_map_for_region(&self, region: &USRegion) -> Option<&ZipToStreetMap> {
        self.region_zip_streets.get(region)
    }
}

impl From<&RegionalRecords> for InMemoryIndexes {

    /// Build indexes given a set of address records and a region name.
    fn from(regional_records: &RegionalRecords) -> InMemoryIndexes 
    {
        let region  = regional_records.region();
        let records = regional_records.records();

        tracing::info!("building indices with {} records",records.len());

        let mut region_zip_streets = BTreeMap::new();
        let mut zip_cities        = BTreeMap::new();
        let mut city_zips         = BTreeMap::new();
        let mut city_streets      = BTreeMap::new();
        let mut street_zips       = BTreeMap::new();
        let mut street_cities     = BTreeMap::new();

        region_zip_streets.insert(region.clone(), BTreeMap::new());

        // For simplicity, assume UnitedStates
        let country = Country::USA;

        for rec in records {

            if rec.is_empty() {
                continue;
            }

            // Construct typed objects if possible
            let city_obj   = rec.city();
            let street_obj = rec.street();
            let zip_obj    = rec.postcode();

            // State->ZIP->Street
            if let (Some(zipc), Some(st)) = (zip_obj.clone(), street_obj.clone()) {
                region_zip_streets
                    .get_mut(&region).unwrap()
                    .entry(zipc)
                    .or_insert_with(BTreeSet::new)
                    .insert(st);
            }

            // ZIP->Cities
            if let (Some(zipc), Some(ct)) = (zip_obj.clone(), city_obj.clone()) {
                zip_cities
                    .entry(zipc)
                    .or_insert_with(BTreeSet::new)
                    .insert(ct);
            }

            // City->ZIP
            if let (Some(ct), Some(zipc)) = (city_obj.clone(), zip_obj.clone()) {
                city_zips
                    .entry(ct)
                    .or_insert_with(BTreeSet::new)
                    .insert(zipc);
            }

            // City->Streets
            if let (Some(ct), Some(st)) = (city_obj.clone(), street_obj.clone()) {
                city_streets
                    .entry(ct)
                    .or_insert_with(BTreeSet::new)
                    .insert(st);
            }

            // Street->ZIPs
            if let (Some(st), Some(zipc)) = (street_obj.clone(), zip_obj.clone()) {
                street_zips
                    .entry(st)
                    .or_insert_with(BTreeSet::new)
                    .insert(zipc);
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
            region_zip_streets,
            zip_cities,
            city_zips,
            city_streets,
            street_zips,
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
        let region  = USRegion::UnitedState(UnitedState::Maryland);
        let rr      = RegionalRecords::mock_for_region(&region);
        let indexes = InMemoryIndexes::from(&rr);
        // Check that indexes contain the expected data:
        // From mock(0): Baltimore, North Avenue, 21201, etc.
        let zip_map = indexes.zip_to_street_map_for_region(&region).unwrap();
        let streets_for_21201 = zip_map.get(&PostalCode::new(Country::USA,"21201").unwrap()).unwrap();
        assert!(streets_for_21201.contains(&StreetName::new("North Avenue").unwrap()));
    }
}
