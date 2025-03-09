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
    fn test_meta_key_from_world_region_maryland() {
        // Suppose region.abbreviation() => "US" (or "MD", depending on your code).
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let mk = MetaKeyForRegion::from(region);

        // We verify the stored region is the same:
        assert_eq!(mk.region(), &region, "Stored region should match");

        // We verify the key is "META:REGION_DONE:<abbr>"
        let expected = format!("META:REGION_DONE:{}", region.abbreviation());
        assert_eq!(mk.key(), &expected, "Key should follow the 'META:REGION_DONE:...' format");
    }

    #[traced_test]
    fn test_meta_key_from_world_region_virginia() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let mk = MetaKeyForRegion::from(region);

        assert_eq!(mk.region(), &region);

        let expected = format!("META:REGION_DONE:{}", region.abbreviation());
        assert_eq!(mk.key(), &expected);
    }

    #[traced_test]
    fn test_meta_key_as_ref_u8() {
        // We can confirm that AsRef<[u8]> yields the bytes of mk.key()
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let mk = MetaKeyForRegion::from(region);

        let bytes: &[u8] = mk.as_ref();
        // Compare to mk.key().as_bytes():
        assert_eq!(bytes, mk.key().as_bytes());
    }

    #[traced_test]
    fn test_meta_key_eq() {
        // Test that two MetaKeyForRegion are equal if their region/key are the same
        let region_a: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let region_b: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();

        let mk_a1 = MetaKeyForRegion::from(region_a);
        let mk_a2 = MetaKeyForRegion::from(region_a);
        let mk_b  = MetaKeyForRegion::from(region_b);

        assert_eq!(mk_a1, mk_a2, "Same region => same meta key => equality");
        assert_ne!(mk_a1, mk_b,  "Different region => different meta key => not equal");
    }

    #[traced_test]
    fn test_meta_key_debug_display() {
        // Checking we can Debug-print the struct. 
        // The derived Debug might show "MetaKeyForRegion { region: ..., key: ... }".
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let mk = MetaKeyForRegion::from(region);

        let dbg_str = format!("{:?}", mk);
        // Just confirm we see partial contents:
        assert!(dbg_str.contains("MetaKeyForRegion"), "Should contain struct name");
        assert!(dbg_str.contains("key: \"META:REGION_DONE:"));
    }
}
