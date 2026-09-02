// ---------------- [ File: src/meta_key.rs ]
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

#[cfg(test)]
mod meta_key_tests {
    use super::*;

    #[traced_test]
    fn test_meta_key_from_world_region_california() {
        info!("Testing MetaKeyForRegion with California region");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::California).into();
        let mk = MetaKeyForRegion::from(region);

        // We verify the stored region is the same:
        assert_eq!(mk.region(), &region, "Stored region should match");

        // We verify the key is "META:REGION_DONE:<abbr>"
        let expected = format!("META:REGION_DONE:{}", region.abbreviation());
        assert_eq!(mk.key(), &expected, "Key should follow 'META:REGION_DONE:...' format");
    }

    #[traced_test]
    fn test_meta_key_from_world_region_texas() {
        info!("Testing MetaKeyForRegion with Texas region");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Texas).into();
        let mk = MetaKeyForRegion::from(region);

        assert_eq!(mk.region(), &region, "Stored region should match Texas");

        let expected = format!("META:REGION_DONE:{}", region.abbreviation());
        assert_eq!(mk.key(), &expected, "Key should follow 'META:REGION_DONE:...' format");
    }

    #[traced_test]
    fn test_meta_key_as_ref_u8_california() {
        info!("Testing AsRef<[u8]> for MetaKeyForRegion (California)");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::California).into();
        let mk = MetaKeyForRegion::from(region);

        let bytes: &[u8] = mk.as_ref();
        // Compare to mk.key().as_bytes():
        assert_eq!(bytes, mk.key().as_bytes());
    }

    #[traced_test]
    fn test_meta_key_eq() {
        info!("Testing equality of MetaKeyForRegion (California vs Texas)");
        let region_a: WorldRegion = USRegion::UnitedState(UnitedState::California).into();
        let region_b: WorldRegion = USRegion::UnitedState(UnitedState::Texas).into();

        let mk_a1 = MetaKeyForRegion::from(region_a);
        let mk_a2 = MetaKeyForRegion::from(region_a);
        let mk_b  = MetaKeyForRegion::from(region_b);

        assert_eq!(mk_a1, mk_a2, "Same region => same meta key => equality");
        assert_ne!(mk_a1, mk_b,  "Different regions => different meta key => not equal");
    }

    #[traced_test]
    fn test_meta_key_debug_display() {
        info!("Testing Debug print of MetaKeyForRegion (California)");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::California).into();
        let mk = MetaKeyForRegion::from(region);

        let dbg_str = format!("{:?}", mk);
        debug!("Debug string for MetaKeyForRegion: {}", dbg_str);

        // Just confirm we see partial contents:
        assert!(dbg_str.contains("MetaKeyForRegion"), "Should contain struct name");
        assert!(dbg_str.contains("META:REGION_DONE:"), "Should contain the key prefix");
    }
}
