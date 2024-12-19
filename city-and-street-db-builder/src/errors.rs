crate::ix!();

error_tree!{

    pub enum InvalidUsaAddress {
        CityNotFoundForZipCodeInRegion {
            city:   CityName,
            region: USRegion,
            zip:    PostalCode,
        },
        ZipToCityKeyNotFoundForRegion {
            z2c_key: String,
            region:  USRegion,
            zip:     PostalCode,
        },
        StreetNotFoundForZipCodeInRegion {
            street: StreetName,
            region: USRegion,
            zip:    PostalCode,
        },
        ZipToStreetKeyNotFoundForRegion {
            s_key:  String,
            region: USRegion,
            zip:    PostalCode,
        },
        StreetNotFoundForCityInRegion {
            street: StreetName,
            city:   CityName,
            region: USRegion,
        },
        CityToStreetsKeyNotFoundForCityInRegion {
            c_key:  String,
            region: USRegion,
            city:   CityName,
        }
    }

    pub enum OsmPbfParseError {
        OsmPbf(osmpbf::Error),
        InvalidInputFile { reason: String },
        UsaAddressBuilderError(UsaAddressBuilderError),
        IoError(io::Error),
    }

    pub enum DatabaseConstructionError {
        OsmPbfParseError(OsmPbfParseError),
        RocksDB(rocksdb::Error),
    }

    pub enum Md5ChecksumVerificationError {
        ChecksumMismatch { 
            expected: String, 
            actual:   String 
        },
        IoError(io::Error),
    }

    pub enum PbfDownloadError {
        IoError(io::Error),
        HttpError(reqwest::Error),
        Md5ChecksumVerificationError(Md5ChecksumVerificationError),
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
