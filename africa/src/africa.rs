crate::ix!();

//-------------------------------------------------------------
// AfricaRegion Enum
//
// Since the user provided a list of African regions/countries
// and stated that there are no subdivided regions, everything
// is top-level. We will treat special combined regions similarly
// to how "GccStates" or "MalaysiaSingaporeBrunei" was handled
// in the Asia example.
//
// Default will be Algeria.
//-------------------------------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum AfricaRegion {
    #[default]
    Algeria,
    Angola,
    Benin,
    Botswana,
    BurkinaFaso,
    Burundi,
    Cameroon,
    CanaryIslands, // territory of Spain (not a sovereign country)
    CapeVerde,
    CentralAfricanRepublic,
    Chad,
    Comores, // Comoros
    CongoRepublicBrazzaville,

    #[strum(serialize = "Congo (Democratic Republic/Kinshasa)")]
    CongoDemocraticRepublicKinshasa,
    Djibouti,
    Egypt,
    EquatorialGuinea,
    Eritrea,
    Ethiopia,
    Gabon,
    Ghana,
    Guinea,
    GuineaBissau,
    IvoryCoast,
    Kenya,
    Lesotho,
    Liberia,
    Libya,
    Madagascar,
    Malawi,
    Mali,
    Mauritania,
    Mauritius,
    Morocco,
    Mozambique,
    Namibia,
    Niger,
    Nigeria,
    Rwanda,
    SaintHelenaAscensionTristanDaCunha, // combined territory
    SaoTomeAndPrincipe,

    //#[strum(serialize = "Senegal and Gambia")]
    SenegalAndGambia, // combined region

    Seychelles,
    SierraLeone,
    Somalia,
    SouthAfrica,
    SouthSudan,
    Sudan,
    Swaziland, // Eswatini
    Tanzania,
    Togo,
    Tunisia,
    Uganda,
    Zambia,
    Zimbabwe,
}
