crate::ix!();

/// All US States (excluding D.C. since it's not a state).
/// 
/// We use strum attributes to handle various forms (full name, abbreviation, and spaceless name).
/// `ascii_case_insensitive` ensures case-insensitive matching.
/// `serialize_all = "title_case"` ensures variant names are considered in Title Case by default.
#[derive(OsmPbfDownloader,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumString,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum UnitedState {

    #[geofabrik(us="alabama-latest.osm.pbf"                )] 
    #[strum(serialize = "Alabama", serialize = "AL"        )] 
    Alabama,

    #[geofabrik(us="alaska-latest.osm.pbf"                 )] 
    #[strum(serialize = "Alaska",         serialize = "AK" )] 
    Alaska,

    #[geofabrik(us="arizona-latest.osm.pbf"                )] 
    #[strum(serialize = "Arizona",        serialize = "AZ" )] 
    Arizona,

    #[geofabrik(us="arkansas-latest.osm.pbf"               )] 
    #[strum(serialize = "Arkansas",       serialize = "AR" )] 
    Arkansas,

    #[geofabrik(us="california-latest.osm.pbf"             )] 
    #[strum(serialize = "California",     serialize = "CA" )] 
    California,

    #[geofabrik(us="colorado-latest.osm.pbf"               )] 
    #[strum(serialize = "Colorado",       serialize = "CO" )] 
    Colorado,

    #[geofabrik(us="connecticut-latest.osm.pbf"            )] 
    #[strum(serialize = "Connecticut",    serialize = "CT" )] 
    Connecticut,

    #[geofabrik(us="delaware-latest.osm.pbf"               )] 
    #[strum(serialize = "Delaware",       serialize = "DE" )] 
    Delaware,

    #[geofabrik(us="florida-latest.osm.pbf"                )] 
    #[strum(serialize = "Florida",        serialize = "FL" )] 
    Florida,

    #[geofabrik(us="georgia-latest.osm.pbf"                )] 
    #[strum(serialize = "Georgia",        serialize = "GA" )] 
    Georgia,

    #[geofabrik(us="hawaii-latest.osm.pbf"                 )] 
    #[strum(serialize = "Hawaii",         serialize = "HI" )] 
    Hawaii,

    #[geofabrik(us="idaho-latest.osm.pbf"                  )] 
    #[strum(serialize = "Idaho",          serialize = "ID" )] 
    Idaho,

    #[geofabrik(us="illinois-latest.osm.pbf"               )] 
    #[strum(serialize = "Illinois",       serialize = "IL" )] 
    Illinois,

    #[geofabrik(us="indiana-latest.osm.pbf"                )] 
    #[strum(serialize = "Indiana",        serialize = "IN" )] 
    Indiana,

    #[geofabrik(us="iowa-latest.osm.pbf"                   )] 
    #[strum(serialize = "Iowa",           serialize = "IA" )] 
    Iowa,

    #[geofabrik(us="kansas-latest.osm.pbf"                 )] 
    #[strum(serialize = "Kansas",         serialize = "KS" )] 
    Kansas,

    #[geofabrik(us="kentucky-latest.osm.pbf"               )] 
    #[strum(serialize = "Kentucky",       serialize = "KY" )] 
    Kentucky,

    #[geofabrik(us="louisiana-latest.osm.pbf"              )] 
    #[strum(serialize = "Louisiana",      serialize = "LA" )] 
    Louisiana,

    #[geofabrik(us="maine-latest.osm.pbf"                  )] 
    #[strum(serialize = "Maine",          serialize = "ME" )] 
    Maine,

    #[geofabrik(us="maryland-latest.osm.pbf"               )] 
    #[strum(serialize = "Maryland",       serialize = "MD" )] 
    Maryland,

    #[geofabrik(us="massachusetts-latest.osm.pbf"          )] 
    #[strum(serialize = "Massachusetts",  serialize = "MA" )] 
    Massachusetts,

    #[geofabrik(us="michigan-latest.osm.pbf"               )] 
    #[strum(serialize = "Michigan",       serialize = "MI" )] 
    Michigan,

    #[geofabrik(us="minnesota-latest.osm.pbf"              )] 
    #[strum(serialize = "Minnesota",      serialize = "MN" )] 
    Minnesota,

    #[geofabrik(us="mississippi-latest.osm.pbf"            )] 
    #[strum(serialize = "Mississippi",    serialize = "MS" )] 
    Mississippi,

    #[geofabrik(us="missouri-latest.osm.pbf"               )] 
    #[strum(serialize = "Missouri",       serialize = "MO" )] 
    Missouri,

    #[geofabrik(us="montana-latest.osm.pbf"                )] 
    #[strum(serialize = "Montana",        serialize = "MT" )] 
    Montana,

    #[geofabrik(us="nebraska-latest.osm.pbf"               )] 
    #[strum(serialize = "Nebraska",       serialize = "NE" )] 
    Nebraska,

    #[geofabrik(us="nevada-latest.osm.pbf"                 )] 
    #[strum(serialize = "Nevada",         serialize = "NV" )] 
    Nevada,

    #[geofabrik(us="new-hampshire-latest.osm.pbf"                                        )] 
    #[strum(serialize = "NewHampshire",   serialize = "New Hampshire",  serialize = "NH" )] 
    NewHampshire,

    #[geofabrik(us="new-jersey-latest.osm.pbf"                                           )] 
    #[strum(serialize = "NewJersey",      serialize = "New Jersey",     serialize = "NJ" )] 
    NewJersey,

    #[geofabrik(us="new-mexico-latest.osm.pbf"                                           )] 
    #[strum(serialize = "NewMexico",      serialize = "New Mexico",     serialize = "NM" )] 
    NewMexico,

    #[geofabrik(us="new-york-latest.osm.pbf"                                             )] 
    #[strum(serialize = "NewYork",        serialize = "New York",       serialize = "NY" )] 
    NewYork,

    #[geofabrik(us="north-carolina-latest.osm.pbf"                                       )] 
    #[strum(serialize = "NorthCarolina",  serialize = "North Carolina", serialize = "NC" )] 
    NorthCarolina,

    #[geofabrik(us="north-dakota-latest.osm.pbf"                                         )] 
    #[strum(serialize = "NorthDakota",    serialize = "North Dakota",   serialize = "ND" )] 
    NorthDakota,

    #[geofabrik(us="ohio-latest.osm.pbf"                   )] 
    #[strum(serialize = "Ohio",           serialize = "OH" )] 
    Ohio,

    #[geofabrik(us="oklahoma-latest.osm.pbf"               )] 
    #[strum(serialize = "Oklahoma",       serialize = "OK" )] 
    Oklahoma,

    #[geofabrik(us="oregon-latest.osm.pbf"                 )] 
    #[strum(serialize = "Oregon",         serialize = "OR" )] 
    Oregon,

    #[geofabrik(us="pennsylvania-latest.osm.pbf"           )] 
    #[strum(serialize = "Pennsylvania",   serialize = "PA" )] 
    Pennsylvania,

    #[geofabrik(us="rhode-island-latest.osm.pbf"                                         )] 
    #[strum(serialize = "RhodeIsland",    serialize = "Rhode Island",   serialize = "RI" )] 
    RhodeIsland,

    #[geofabrik(us="south-carolina-latest.osm.pbf"                                       )] 
    #[strum(serialize = "SouthCarolina",  serialize = "South Carolina", serialize = "SC" )] 
    SouthCarolina,

    #[geofabrik(us="south-dakota-latest.osm.pbf"                                         )] 
    #[strum(serialize = "SouthDakota",    serialize = "South Dakota",   serialize = "SD" )] 
    SouthDakota,

    #[geofabrik(us="tennessee-latest.osm.pbf"              )] 
    #[strum(serialize = "Tennessee",      serialize = "TN" )] 
    Tennessee,

    #[geofabrik(us="texas-latest.osm.pbf"                  )] 
    #[strum(serialize = "Texas",          serialize = "TX" )] 
    Texas,

    #[geofabrik(us="utah-latest.osm.pbf"                   )] 
    #[strum(serialize = "Utah",           serialize = "UT" )] 
    Utah,

    #[geofabrik(us="vermont-latest.osm.pbf"                )] 
    #[strum(serialize = "Vermont",        serialize = "VT" )] 
    Vermont,

    #[geofabrik(us="virginia-latest.osm.pbf"               )] 
    #[strum(serialize = "Virginia",       serialize = "VA" )] 
    Virginia,

    #[geofabrik(us="washington-latest.osm.pbf"             )] 
    #[strum(serialize = "Washington",     serialize = "WA" )] 
    Washington,

    #[geofabrik(us="west-virginia-latest.osm.pbf"                                        )] 
    #[strum(serialize = "WestVirginia",   serialize = "West Virginia",  serialize = "WV" )] 
    WestVirginia,

    #[geofabrik(us="wisconsin-latest.osm.pbf"              )] 
    #[strum(serialize = "Wisconsin",      serialize = "WI" )] 
    Wisconsin,

    #[geofabrik(us="wyoming-latest.osm.pbf"                )] 
    #[strum(serialize = "Wyoming",        serialize = "WY" )] 
    Wyoming,
}

impl UnitedState {

    pub fn all_states() -> Vec<UnitedState> {
        UnitedState::iter().collect()
    }
}

#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for UnitedState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for UnitedState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for UnitedState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        s.parse::<UnitedState>().map_err(|_| serde::de::Error::unknown_variant(&s, UnitedState::VARIANTS))
    }
}
