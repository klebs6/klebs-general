crate::ix!();

impl From<USRegion> for OpenStreetMapRegionalDataDownloadHandle {

    fn from(region: USRegion) -> Self {
        use UnitedState::*;
        use USTerritory::*;
        use USFederalDistrict::*;

        macro_rules! osm_handle { ($link:expr) => { OpenStreetMapRegionalDataDownloadHandle::new(region,$link)} }

        match region {
            USRegion::UnitedState(Alabama)                  => osm_handle!{ "https://download.geofabrik.de/north-america/us/alabama-latest.osm.pbf"              },
            USRegion::UnitedState(Alaska)                   => osm_handle!{ "https://download.geofabrik.de/north-america/us/alaska-latest.osm.pbf"               },
            USRegion::UnitedState(Arizona)                  => osm_handle!{ "https://download.geofabrik.de/north-america/us/arizona-latest.osm.pbf"              },
            USRegion::UnitedState(Arkansas)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/arkansas-latest.osm.pbf"             },
            USRegion::UnitedState(California)               => osm_handle!{ "https://download.geofabrik.de/north-america/us/california-latest.osm.pbf"           },
            USRegion::UnitedState(Colorado)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/colorado-latest.osm.pbf"             },
            USRegion::UnitedState(Connecticut)              => osm_handle!{ "https://download.geofabrik.de/north-america/us/connecticut-latest.osm.pbf"          },
            USRegion::UnitedState(Delaware)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/delaware-latest.osm.pbf"             },
            USRegion::UnitedState(Florida)                  => osm_handle!{ "https://download.geofabrik.de/north-america/us/florida-latest.osm.pbf"              },
            USRegion::UnitedState(Georgia)                  => osm_handle!{ "https://download.geofabrik.de/north-america/us/georgia-latest.osm.pbf"              },
            USRegion::UnitedState(Hawaii)                   => osm_handle!{ "https://download.geofabrik.de/north-america/us/hawaii-latest.osm.pbf"               },
            USRegion::UnitedState(Idaho)                    => osm_handle!{ "https://download.geofabrik.de/north-america/us/idaho-latest.osm.pbf"                },
            USRegion::UnitedState(Illinois)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/illinois-latest.osm.pbf"             },
            USRegion::UnitedState(Indiana)                  => osm_handle!{ "https://download.geofabrik.de/north-america/us/indiana-latest.osm.pbf"              },
            USRegion::UnitedState(Iowa)                     => osm_handle!{ "https://download.geofabrik.de/north-america/us/iowa-latest.osm.pbf"                 },
            USRegion::UnitedState(Kansas)                   => osm_handle!{ "https://download.geofabrik.de/north-america/us/kansas-latest.osm.pbf"               },
            USRegion::UnitedState(Kentucky)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/kentucky-latest.osm.pbf"             },
            USRegion::UnitedState(Louisiana)                => osm_handle!{ "https://download.geofabrik.de/north-america/us/louisiana-latest.osm.pbf"            },
            USRegion::UnitedState(Maine)                    => osm_handle!{ "https://download.geofabrik.de/north-america/us/maine-latest.osm.pbf"                },
            USRegion::UnitedState(Maryland)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/maryland-latest.osm.pbf"             },
            USRegion::UnitedState(Massachusetts)            => osm_handle!{ "https://download.geofabrik.de/north-america/us/massachusetts-latest.osm.pbf"        },
            USRegion::UnitedState(Michigan)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/michigan-latest.osm.pbf"             },
            USRegion::UnitedState(Minnesota)                => osm_handle!{ "https://download.geofabrik.de/north-america/us/minnesota-latest.osm.pbf"            },
            USRegion::UnitedState(Mississippi)              => osm_handle!{ "https://download.geofabrik.de/north-america/us/mississippi-latest.osm.pbf"          },
            USRegion::UnitedState(Missouri)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/missouri-latest.osm.pbf"             },
            USRegion::UnitedState(Montana)                  => osm_handle!{ "https://download.geofabrik.de/north-america/us/montana-latest.osm.pbf"              },
            USRegion::UnitedState(Nebraska)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/nebraska-latest.osm.pbf"             },
            USRegion::UnitedState(Nevada)                   => osm_handle!{ "https://download.geofabrik.de/north-america/us/nevada-latest.osm.pbf"               },
            USRegion::UnitedState(NewHampshire)             => osm_handle!{ "https://download.geofabrik.de/north-america/us/new-hampshire-latest.osm.pbf"        },
            USRegion::UnitedState(NewJersey)                => osm_handle!{ "https://download.geofabrik.de/north-america/us/new-jersey-latest.osm.pbf"           },
            USRegion::UnitedState(NewMexico)                => osm_handle!{ "https://download.geofabrik.de/north-america/us/new-mexico-latest.osm.pbf"           },
            USRegion::UnitedState(NewYork)                  => osm_handle!{ "https://download.geofabrik.de/north-america/us/new-york-latest.osm.pbf"             },
            USRegion::UnitedState(NorthCarolina)            => osm_handle!{ "https://download.geofabrik.de/north-america/us/north-carolina-latest.osm.pbf"       },
            USRegion::UnitedState(NorthDakota)              => osm_handle!{ "https://download.geofabrik.de/north-america/us/north-dakota-latest.osm.pbf"         },
            USRegion::UnitedState(Ohio)                     => osm_handle!{ "https://download.geofabrik.de/north-america/us/ohio-latest.osm.pbf"                 },
            USRegion::UnitedState(Oklahoma)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/oklahoma-latest.osm.pbf"             },
            USRegion::UnitedState(Oregon)                   => osm_handle!{ "https://download.geofabrik.de/north-america/us/oregon-latest.osm.pbf"               },
            USRegion::UnitedState(Pennsylvania)             => osm_handle!{ "https://download.geofabrik.de/north-america/us/pennsylvania-latest.osm.pbf"         },
            USRegion::UnitedState(RhodeIsland)              => osm_handle!{ "https://download.geofabrik.de/north-america/us/rhode-island-latest.osm.pbf"         },
            USRegion::UnitedState(SouthCarolina)            => osm_handle!{ "https://download.geofabrik.de/north-america/us/south-carolina-latest.osm.pbf"       },
            USRegion::UnitedState(SouthDakota)              => osm_handle!{ "https://download.geofabrik.de/north-america/us/south-dakota-latest.osm.pbf"         },
            USRegion::UnitedState(Tennessee)                => osm_handle!{ "https://download.geofabrik.de/north-america/us/tennessee-latest.osm.pbf"            },
            USRegion::UnitedState(Texas)                    => osm_handle!{ "https://download.geofabrik.de/north-america/us/texas-latest.osm.pbf"                },
            USRegion::UnitedState(Utah)                     => osm_handle!{ "https://download.geofabrik.de/north-america/us/utah-latest.osm.pbf"                 },
            USRegion::UnitedState(Vermont)                  => osm_handle!{ "https://download.geofabrik.de/north-america/us/vermont-latest.osm.pbf"              },
            USRegion::UnitedState(Virginia)                 => osm_handle!{ "https://download.geofabrik.de/north-america/us/virginia-latest.osm.pbf"             },
            USRegion::UnitedState(Washington)               => osm_handle!{ "https://download.geofabrik.de/north-america/us/washington-latest.osm.pbf"           },
            USRegion::UnitedState(WestVirginia)             => osm_handle!{ "https://download.geofabrik.de/north-america/us/west-virginia-latest.osm.pbf"        },
            USRegion::UnitedState(Wisconsin)                => osm_handle!{ "https://download.geofabrik.de/north-america/us/wisconsin-latest.osm.pbf"            },
            USRegion::UnitedState(Wyoming)                  => osm_handle!{ "https://download.geofabrik.de/north-america/us/wyoming-latest.osm.pbf"              },
            USRegion::USTerritory(VirginIslands)            => osm_handle!{ "https://download.geofabrik.de/north-america/us/us-virgin-islands-latest.osm.pbf"    },
            USRegion::USTerritory(PuertoRico)               => osm_handle!{ "https://download.geofabrik.de/north-america/us/puerto-rico-latest.osm.pbf"          },
            USRegion::USFederalDistrict(DistrictOfColumbia) => osm_handle!{ "https://download.geofabrik.de/north-america/us/district-of-columbia-latest.osm.pbf" },

            _ => unimplemented!(),
        }
    }
}

/// Tests for download and MD5 verification (mocked, since we won't do real network ops here)
#[cfg(test)]
mod download_tests {
    use super::*;
    use tokio::runtime::Runtime;
    use std::fs::File as StdFile;
    use std::io::Write;

    #[test]
    fn verify_md5_checksum_mismatch() {
        let region = USRegion::UnitedState(UnitedState::Maryland);
        let handle = OpenStreetMapRegionalDataDownloadHandle::from(region);

        let rt = Runtime::new().unwrap();
        let tmp_path = std::env::temp_dir().join("md5_test.osm.pbf");
        {
            let mut f = StdFile::create(&tmp_path).unwrap();
            f.write_all(b"some random data").unwrap();
        }

        let res = rt.block_on(handle.verify_md5_checksum(&tmp_path));
        assert!(res.is_err());
        match res.err().unwrap() {
            Md5ChecksumVerificationError::ChecksumMismatch { .. } => {},
            _ => panic!("Expected ChecksumMismatch"),
        }
    }
}
