crate::ix!();

#[derive(Builder,Debug,Getters)]
#[getset(get="pub")]
#[builder(setter(into))]
pub struct RegionalRecords {
    region:  WorldRegion,
    records: Vec<AddressRecord>,
}

impl RegionalRecords {

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn from_osm_pbf_file(region: WorldRegion, pbf_file: impl AsRef<Path>) 
        -> Result<Self,OsmPbfParseError> 
    {
        let pbf_path = pbf_file.as_ref();
        let country  = region.country();

        validate_pbf_filename(&region, pbf_path)?;
        let records = parse_osm_pbf(pbf_path,&country)?;

        Ok(Self {
            region,
            records,
        })
    }

    /// store region data in rocksdb
    pub fn write_to_storage(&self, db: &mut Database) 
        -> Result<(),DatabaseConstructionError> 
    {
        info!("writing regional records to storage for region: {:#?}", self.region);

        if db.region_done(&self.region)? {
            tracing::info!("Region {} already built, skipping", self.region);
            return Ok(());
        }

        db.write_indexes(&self.region, &InMemoryIndexes::from(self))?;

        db.mark_region_done(&self.region)?;

        Ok(())
    }
}
