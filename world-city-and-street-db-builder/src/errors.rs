// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum DataAccessError {
        #[cmp_neq]
        Io(io::Error),
        LockPoisoned,
        DatabaseConstructionError(DatabaseConstructionError),
        PostalCodeError(PostalCodeConstructionError),
    }

    pub enum AddressValidationError {
        #[cmp_neq]
        IoError(io::Error),
        DatabaseConstructionError(DatabaseConstructionError),
        LockPoisoned,
    }

    #[derive(PartialEq)]
    pub enum ListAllAddressesError {
        #[cmp_neq]
        IoError(io::Error),
        OsmPbfParseError(OsmPbfParseError),
    }

    #[derive(PartialEq)]
    pub enum InvalidWorldAddress {
        CityNotFoundForPostalCodeInRegion {
            city:        CityName,
            region:      WorldRegion,
            postal_code: PostalCode,
        },
        PostalCodeToCityKeyNotFoundForRegion {
            z2c_key:     String,
            region:      WorldRegion,
            postal_code: PostalCode,
        },
        StreetNotFoundForPostalCodeInRegion {
            street:      StreetName,
            region:      WorldRegion,
            postal_code: PostalCode,
        },
        PostalCodeToStreetKeyNotFoundForRegion {
            s_key:       String,
            region:      WorldRegion,
            postal_code: PostalCode,
        },
        StreetNotFoundForCityInRegion {
            street:      StreetName,
            city:        CityName,
            region:      WorldRegion,
        },
        CityToStreetsKeyNotFoundForCityInRegion {
            c_key:       String,
            region:      WorldRegion,
            city:        CityName,
        }
    }

    #[derive(PartialEq)]
    pub enum OsmPbfParseError {

        #[cmp_neq]
        WorldRegionConversionError(WorldRegionConversionError),

        #[cmp_neq]
        OsmPbf(osmpbf::Error),

        InvalidInputFile { reason: String },

        #[cmp_neq]
        WorldAddressBuilderError(WorldAddressBuilderError),

        #[cmp_neq]
        IoError(io::Error),

        HouseNumberRangeSerdeError {
            msg: String,
        },
    }

    #[derive(PartialEq)]
    pub enum DatabaseConstructionError {
        DataAccessError,
        OsmPbfParseError(OsmPbfParseError),
        RocksDB(rocksdb::Error),
    }

    #[derive(PartialEq)]
    pub enum WorldCityAndStreetDbBuilderError {

        SimulatedDownloadFailure,

        #[cmp_neq]
        DownloadError(DownloadError),

        DatabaseConstructionError(DatabaseConstructionError),
        DbLockError,
        NotAllAddressesValidatedSuccessfully,
    }

    /// Error types for city and street name construction
    #[derive(PartialEq)]
    pub enum CityNameConstructionError {
        InvalidName { attempted_name: String }

        #[cmp_neq]
        UninitializedField(derive_builder::UninitializedFieldError),
    }

    #[derive(PartialEq)]
    pub enum StreetNameConstructionError {
        InvalidName { attempted_name: String }

        #[cmp_neq]
        UninitializedField(derive_builder::UninitializedFieldError),
    }

    #[derive(PartialEq)]
    pub enum IncompatibleOsmPbfElement {
        MaybeTodoUnhandledOsmPbfRelationElement,
        MaybeTodoUnhandledOsmPbfDenseNode,
        IncompatibleOsmPbfNode(IncompatibleOsmPbfNode),
        IncompatibleOsmPbfWay(IncompatibleOsmPbfWay),
        IncompatibleOsmPbfRelation(IncompatibleOsmPbfRelation),
        IncompatibleOsmPbfDenseNode(IncompatibleOsmPbfDenseNode),
    }

    #[derive(PartialEq)]
    pub enum IncompatibleOsmPbfDenseNode {
        Incompatible {
            id: i64,
        },
        CityNameConstructionError(CityNameConstructionError),
        StreetNameConstructionError(StreetNameConstructionError),
        PostalCodeConstructionError(PostalCodeConstructionError),
    }

    #[derive(PartialEq)]
    pub enum IncompatibleOsmPbfRelation {
        Incompatible {
            id: i64,
        },
        CityNameConstructionError(CityNameConstructionError),
        StreetNameConstructionError(StreetNameConstructionError),
        PostalCodeConstructionError(PostalCodeConstructionError),
    }

    #[derive(PartialEq)]
    pub enum IncompatibleOsmPbfNode {
        Incompatible {
            id: i64,
        },
        CityNameConstructionError(CityNameConstructionError),
        StreetNameConstructionError(StreetNameConstructionError),
        PostalCodeConstructionError(PostalCodeConstructionError),

        #[cmp_neq]
        AddressRecordBuilderError {
            id: i64,
            source: AddressRecordBuilderError,
        }
    }

    #[derive(PartialEq)]
    pub enum IncompatibleOsmPbfWay {
        Incompatible {
            id: i64,
        },
        CityNameConstructionError(CityNameConstructionError),
        StreetNameConstructionError(StreetNameConstructionError),
        PostalCodeConstructionError(PostalCodeConstructionError),
    }
}
