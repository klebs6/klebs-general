crate::ix!();

#[derive(Getters,Setters,Debug,Clone,PartialEq,Eq)]
#[getset(get="pub",set="pub")]
pub struct MetaKeyForRegion {
    region: WorldRegion,
    key:    String,
}

impl AsRef<[u8]> for MetaKeyForRegion {

    fn as_ref(&self) -> &[u8] { 
        self.key.as_bytes()
    }
}

impl From<WorldRegion> for MetaKeyForRegion {

    fn from(region: WorldRegion) -> Self {
        let region_name = region.abbreviation();
        Self {
            region,
            key: format!("META:REGION_DONE:{}", region_name),
        }
    }
}
