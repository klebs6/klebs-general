crate::ix!();

error_tree!{

    pub enum InvalidWorldAddress {
        CityNotFoundForPostalCodeInRegion {
            city:   CityName,
            region: WorldRegion,
            zip:    PostalCode,
        },
        PostalCodeToCityKeyNotFoundForRegion {
            z2c_key: String,
            region:  WorldRegion,
            zip:     PostalCode,
        },
        StreetNotFoundForPostalCodeInRegion {
            street: StreetName,
            region: WorldRegion,
            zip:    PostalCode,
        },
        PostalCodeToStreetKeyNotFoundForRegion {
            s_key:  String,
            region: WorldRegion,
            zip:    PostalCode,
        },
        StreetNotFoundForCityInRegion {
            street: StreetName,
            city:   CityName,
            region: WorldRegion,
        },
        CityToStreetsKeyNotFoundForCityInRegion {
            c_key:  String,
            region: WorldRegion,
            city:   CityName,
        }
    }

    pub enum OsmPbfParseError {
        OsmPbf(osmpbf::Error),
        InvalidInputFile { reason: String },
        WorldAddressBuilderError(WorldAddressBuilderError),
        IoError(io::Error),
    }

    pub enum DatabaseConstructionError {
        OsmPbfParseError(OsmPbfParseError),
        RocksDB(rocksdb::Error),
    }

    pub enum UsaCityAndStreetDbBuilderError {
        PbfDownloadError(PbfDownloadError),
        DatabaseConstructionError(DatabaseConstructionError),
        DbLockError,
        NotAllAddressesValidatedSuccessfully,
    }

    /// Error types for city and street name construction
    pub enum CityNameConstructionError {
        InvalidName { attempted_name: String }
        UninitializedField(derive_builder::UninitializedFieldError),
    }

    pub enum StreetNameConstructionError {
        InvalidName { attempted_name: String }
        UninitializedField(derive_builder::UninitializedFieldError),
    }

    pub enum IncompatibleOsmPbfElement {
        MaybeTodoUnhandledOsmPbfRelationElement,
        MaybeTodoUnhandledOsmPbfDenseNode,
        IncompatibleOsmPbfNode(IncompatibleOsmPbfNode),
        IncompatibleOsmPbfWay(IncompatibleOsmPbfWay),
        IncompatibleOsmPbfRelation(IncompatibleOsmPbfRelation),
        IncompatibleOsmPbfDenseNode(IncompatibleOsmPbfDenseNode),
    }

    pub enum IncompatibleOsmPbfDenseNode {
        Incompatible {
            id: i64,
        },
        CityNameConstructionError(CityNameConstructionError),
        StreetNameConstructionError(StreetNameConstructionError),
        PostalCodeConstructionError(PostalCodeConstructionError),
    }

    pub enum IncompatibleOsmPbfRelation {
        Incompatible {
            id: i64,
        },
        CityNameConstructionError(CityNameConstructionError),
        StreetNameConstructionError(StreetNameConstructionError),
        PostalCodeConstructionError(PostalCodeConstructionError),
    }

    pub enum IncompatibleOsmPbfNode {
        Incompatible {
            id: i64,
        },
        CityNameConstructionError(CityNameConstructionError),
        StreetNameConstructionError(StreetNameConstructionError),
        PostalCodeConstructionError(PostalCodeConstructionError),
    }

    pub enum IncompatibleOsmPbfWay {
        Incompatible {
            id: i64,
        },
        CityNameConstructionError(CityNameConstructionError),
        StreetNameConstructionError(StreetNameConstructionError),
        PostalCodeConstructionError(PostalCodeConstructionError),
    }
}
