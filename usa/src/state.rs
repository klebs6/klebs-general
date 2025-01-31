crate::ix!();

/// All US States (excluding D.C. since it's not a state).
/// 
/// We use strum attributes to handle various forms (full name, abbreviation, and spaceless name).
/// `ascii_case_insensitive` ensures case-insensitive matching.
/// `serialize_all = "title_case"` ensures variant names are considered in Title Case by default.
#[derive(FileDownloader,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumString,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum UnitedState {

    #[download_link("https://download.geofabrik.de/north-america/us/alabama-latest.osm.pbf")] 
    #[strum(serialize = "Alabama", serialize = "AL"        )] 
    Alabama,

    #[download_link("https://download.geofabrik.de/north-america/us/alaska-latest.osm.pbf")] 
    #[strum(serialize = "Alaska",         serialize = "AK" )] 
    Alaska,

    #[download_link("https://download.geofabrik.de/north-america/us/arizona-latest.osm.pbf")] 
    #[strum(serialize = "Arizona",        serialize = "AZ" )] 
    Arizona,

    #[download_link("https://download.geofabrik.de/north-america/us/arkansas-latest.osm.pbf")] 
    #[strum(serialize = "Arkansas",       serialize = "AR" )] 
    Arkansas,

    #[download_link("https://download.geofabrik.de/north-america/us/california-latest.osm.pbf")] 
    #[strum(serialize = "California",     serialize = "CA" )] 
    California,

    #[download_link("https://download.geofabrik.de/north-america/us/colorado-latest.osm.pbf")] 
    #[strum(serialize = "Colorado",       serialize = "CO" )] 
    Colorado,

    #[download_link("https://download.geofabrik.de/north-america/us/connecticut-latest.osm.pbf")] 
    #[strum(serialize = "Connecticut",    serialize = "CT" )] 
    Connecticut,

    #[download_link("https://download.geofabrik.de/north-america/us/delaware-latest.osm.pbf")] 
    #[strum(serialize = "Delaware",       serialize = "DE" )] 
    Delaware,

    #[download_link("https://download.geofabrik.de/north-america/us/florida-latest.osm.pbf")] 
    #[strum(serialize = "Florida",        serialize = "FL" )] 
    Florida,

    #[download_link("https://download.geofabrik.de/north-america/us/georgia-latest.osm.pbf")] 
    #[strum(serialize = "Georgia",        serialize = "GA" )] 
    Georgia,

    #[download_link("https://download.geofabrik.de/north-america/us/hawaii-latest.osm.pbf")] 
    #[strum(serialize = "Hawaii",         serialize = "HI" )] 
    Hawaii,

    #[download_link("https://download.geofabrik.de/north-america/us/idaho-latest.osm.pbf")] 
    #[strum(serialize = "Idaho",          serialize = "ID" )] 
    Idaho,

    #[download_link("https://download.geofabrik.de/north-america/us/illinois-latest.osm.pbf")] 
    #[strum(serialize = "Illinois",       serialize = "IL" )] 
    Illinois,

    #[download_link("https://download.geofabrik.de/north-america/us/indiana-latest.osm.pbf")] 
    #[strum(serialize = "Indiana",        serialize = "IN" )] 
    Indiana,

    #[download_link("https://download.geofabrik.de/north-america/us/iowa-latest.osm.pbf")] 
    #[strum(serialize = "Iowa",           serialize = "IA" )] 
    Iowa,

    #[download_link("https://download.geofabrik.de/north-america/us/kansas-latest.osm.pbf")] 
    #[strum(serialize = "Kansas",         serialize = "KS" )] 
    Kansas,

    #[download_link("https://download.geofabrik.de/north-america/us/kentucky-latest.osm.pbf")] 
    #[strum(serialize = "Kentucky",       serialize = "KY" )] 
    Kentucky,

    #[download_link("https://download.geofabrik.de/north-america/us/louisiana-latest.osm.pbf")] 
    #[strum(serialize = "Louisiana",      serialize = "LA" )] 
    Louisiana,

    #[download_link("https://download.geofabrik.de/north-america/us/maine-latest.osm.pbf")] 
    #[strum(serialize = "Maine",          serialize = "ME" )] 
    Maine,

    #[download_link("https://download.geofabrik.de/north-america/us/maryland-latest.osm.pbf")] 
    #[strum(serialize = "Maryland",       serialize = "MD" )] 
    Maryland,

    #[download_link("https://download.geofabrik.de/north-america/us/massachusetts-latest.osm.pbf")] 
    #[strum(serialize = "Massachusetts",  serialize = "MA" )] 
    Massachusetts,

    #[download_link("https://download.geofabrik.de/north-america/us/michigan-latest.osm.pbf")] 
    #[strum(serialize = "Michigan",       serialize = "MI" )] 
    Michigan,

    #[download_link("https://download.geofabrik.de/north-america/us/minnesota-latest.osm.pbf")] 
    #[strum(serialize = "Minnesota",      serialize = "MN" )] 
    Minnesota,

    #[download_link("https://download.geofabrik.de/north-america/us/mississippi-latest.osm.pbf")] 
    #[strum(serialize = "Mississippi",    serialize = "MS" )] 
    Mississippi,

    #[download_link("https://download.geofabrik.de/north-america/us/missouri-latest.osm.pbf")] 
    #[strum(serialize = "Missouri",       serialize = "MO" )] 
    Missouri,

    #[download_link("https://download.geofabrik.de/north-america/us/montana-latest.osm.pbf")] 
    #[strum(serialize = "Montana",        serialize = "MT" )] 
    Montana,

    #[download_link("https://download.geofabrik.de/north-america/us/nebraska-latest.osm.pbf")] 
    #[strum(serialize = "Nebraska",       serialize = "NE" )] 
    Nebraska,

    #[download_link("https://download.geofabrik.de/north-america/us/nevada-latest.osm.pbf")] 
    #[strum(serialize = "Nevada",         serialize = "NV" )] 
    Nevada,

    #[download_link("https://download.geofabrik.de/north-america/us/new-hampshire-latest.osm.pbf")] 
    #[strum(serialize = "NewHampshire",   serialize = "New Hampshire",  serialize = "NH" )] 
    NewHampshire,

    #[download_link("https://download.geofabrik.de/north-america/us/new-jersey-latest.osm.pbf")] 
    #[strum(serialize = "NewJersey",      serialize = "New Jersey",     serialize = "NJ" )] 
    NewJersey,

    #[download_link("https://download.geofabrik.de/north-america/us/new-mexico-latest.osm.pbf")] 
    #[strum(serialize = "NewMexico",      serialize = "New Mexico",     serialize = "NM" )] 
    NewMexico,

    #[download_link("https://download.geofabrik.de/north-america/us/new-york-latest.osm.pbf")] 
    #[strum(serialize = "NewYork",        serialize = "New York",       serialize = "NY" )] 
    NewYork,

    #[download_link("https://download.geofabrik.de/north-america/us/north-carolina-latest.osm.pbf")] 
    #[strum(serialize = "NorthCarolina",  serialize = "North Carolina", serialize = "NC" )] 
    NorthCarolina,

    #[download_link("https://download.geofabrik.de/north-america/us/north-dakota-latest.osm.pbf")] 
    #[strum(serialize = "NorthDakota",    serialize = "North Dakota",   serialize = "ND" )] 
    NorthDakota,

    #[download_link("https://download.geofabrik.de/north-america/us/ohio-latest.osm.pbf")] 
    #[strum(serialize = "Ohio",           serialize = "OH" )] 
    Ohio,

    #[download_link("https://download.geofabrik.de/north-america/us/oklahoma-latest.osm.pbf")] 
    #[strum(serialize = "Oklahoma",       serialize = "OK" )] 
    Oklahoma,

    #[download_link("https://download.geofabrik.de/north-america/us/oregon-latest.osm.pbf")] 
    #[strum(serialize = "Oregon",         serialize = "OR" )] 
    Oregon,

    #[download_link("https://download.geofabrik.de/north-america/us/pennsylvania-latest.osm.pbf")] 
    #[strum(serialize = "Pennsylvania",   serialize = "PA" )] 
    Pennsylvania,

    #[download_link("https://download.geofabrik.de/north-america/us/rhode-island-latest.osm.pbf")] 
    #[strum(serialize = "RhodeIsland",    serialize = "Rhode Island",   serialize = "RI" )] 
    RhodeIsland,

    #[download_link("https://download.geofabrik.de/north-america/us/south-carolina-latest.osm.pbf")] 
    #[strum(serialize = "SouthCarolina",  serialize = "South Carolina", serialize = "SC" )] 
    SouthCarolina,

    #[download_link("https://download.geofabrik.de/north-america/us/south-dakota-latest.osm.pbf")] 
    #[strum(serialize = "SouthDakota",    serialize = "South Dakota",   serialize = "SD" )] 
    SouthDakota,

    #[download_link("https://download.geofabrik.de/north-america/us/tennessee-latest.osm.pbf")] 
    #[strum(serialize = "Tennessee",      serialize = "TN" )] 
    Tennessee,

    #[download_link("https://download.geofabrik.de/north-america/us/texas-latest.osm.pbf")] 
    #[strum(serialize = "Texas",          serialize = "TX" )] 
    Texas,

    #[download_link("https://download.geofabrik.de/north-america/us/utah-latest.osm.pbf")] 
    #[strum(serialize = "Utah",           serialize = "UT" )] 
    Utah,

    #[download_link("https://download.geofabrik.de/north-america/us/vermont-latest.osm.pbf")] 
    #[strum(serialize = "Vermont",        serialize = "VT" )] 
    Vermont,

    #[download_link("https://download.geofabrik.de/north-america/us/virginia-latest.osm.pbf")] 
    #[strum(serialize = "Virginia",       serialize = "VA" )] 
    Virginia,

    #[download_link("https://download.geofabrik.de/north-america/us/washington-latest.osm.pbf")] 
    #[strum(serialize = "Washington",     serialize = "WA" )] 
    Washington,

    #[download_link("https://download.geofabrik.de/north-america/us/west-virginia-latest.osm.pbf")] 
    #[strum(serialize = "WestVirginia",   serialize = "West Virginia",  serialize = "WV" )] 
    WestVirginia,

    #[download_link("https://download.geofabrik.de/north-america/us/wisconsin-latest.osm.pbf")] 
    #[strum(serialize = "Wisconsin",      serialize = "WI" )] 
    Wisconsin,

    #[download_link("https://download.geofabrik.de/north-america/us/wyoming-latest.osm.pbf")] 
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
