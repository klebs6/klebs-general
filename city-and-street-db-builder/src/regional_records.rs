crate::ix!();

/// Build multiple regions from PBF files:
pub async fn build_regions(
    regions:    &[USRegion],
    db:         &mut Database,
    target_dir: &Path,
) -> Result<(),UsaCityAndStreetDbBuilderError> {
    for region in regions {

        info!("building region: {:#?}", region);

        let handle   = OpenStreetMapRegionalDataDownloadHandle::from(region.clone());

        info!("download_handle: {:#?}", handle);

        let pbf_file = handle.obtain_pbf(&target_dir).await?;

        info!("pbf_file: {:#?}", pbf_file);

        let regional_records = RegionalRecords::from_osm_pbf_file(*region,pbf_file)?;

        info!("scanned {} regional_records", regional_records.len());

        regional_records.write_to_storage(db)?;
    }
    Ok(())
}

#[derive(Builder,Debug,Getters)]
#[getset(get="pub")]
#[builder(setter(into))]
pub struct RegionalRecords {
    region:  USRegion,
    records: Vec<AddressRecord>,
}

impl RegionalRecords {

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn from_osm_pbf_file(region: USRegion, pbf_file: impl AsRef<Path>) 
        -> Result<Self,OsmPbfParseError> 
    {
        let pbf_path = pbf_file.as_ref();

        validate_pbf_filename(&region, pbf_path)?;
        let records = parse_osm_pbf(pbf_path)?;

        Ok(Self {
            region,
            records,
        })
    }

    /// store region data in rocksdb
    pub fn write_to_storage(&self, db: &mut Database) 
        -> Result<(),DatabaseConstructionError> 
    {
        if db.region_done(&self.region)? {
            tracing::info!("Region {} already built, skipping", self.region);
            return Ok(());
        }

        db.write_indexes(&self.region, &InMemoryIndexes::from(self))?;

        db.mark_region_done(&self.region)?;

        Ok(())
    }
}
