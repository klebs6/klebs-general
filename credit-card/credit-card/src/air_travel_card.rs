crate::ix!();

//------------------------------------[air-travel-card-historical]

/// Historical credit instrument representing the Air Travel Card,
/// first introduced in 1934 by American Airlines and the Air Transport Association.
///
/// Provided passengers the ability to purchase airline tickets on credit,
/// supported international validity, discounts, and installment plans.
pub struct AirTravelCard {
    /// Structured numbering scheme clearly identifying the issuer and customer account.
    number: AirTravelCardNumber,

    /// Operational features defining usage and financial attributes.
    operational_features: AirTravelCardOperationalFeatures,
}

/// Represents the numbering scheme of Air Travel Cards,
/// clearly separating issuer identification from customer account numbers.
pub struct AirTravelCardNumber {
    /// First digit '1', inherited historically and preserved in modern UATP cards.
    first_digit: u8, // Historically and currently fixed to '1'

    /// Numeric digits identifying the card issuer (airline).
    issuer_identifier_digits: Vec<u8>,

    /// Numeric digits uniquely identifying the customer account.
    customer_account_digits: Vec<u8>,

    /// Validity check digit, historically used for number validation.
    check_digit: u8,
}

/// Represents operational features historically associated with Air Travel Cards.
pub struct AirTravelCardOperationalFeatures {
    /// Fixed historical discount percentage on ticket purchases (e.g., historically 15%).
    fixed_discount_percentage: f64,

    /// Indicates if installment payment plans were supported.
    installment_plans_supported: bool,

    /// List of airline identifiers accepting the card historically (e.g., 17 airlines by the 1940s).
    participating_airlines: Vec<AirlineIdentifier>,

    /// Indicates international validity and recognition across IATA members.
    internationally_valid: bool,
}

/// Represents historical identifiers for airlines participating in the Air Travel Card program.
pub struct AirlineIdentifier {
    /// Numeric identifier uniquely associated with an airline.
    airline_id: Vec<u8>,
}

/// Comprehensive historical representation of Air Travel Cards,
/// explicitly capturing both identification schemes and operational characteristics.
pub struct HistoricalAirTravelCardSystem {
    /// The historical Air Travel Card representation.
    card: AirTravelCard,

    /// Historical context details such as introduction year and initiating entities.
    historical_context: HistoricalIntroductionContext,
}

/// Provides contextual historical details about the introduction and origin of the Air Travel Card.
pub struct HistoricalIntroductionContext {
    /// Year the card was first introduced (1934).
    introduction_year: u16,

    /// Entities responsible for the introduction of the Air Travel Card.
    introduced_by: Vec<IntroducingEntity>,
}

/// Entities responsible for historically introducing the Air Travel Card.
pub enum IntroducingEntity {
    AmericanAirlines,
    AirTransportAssociation,
    Other(Vec<u8>), // Numeric identifier for other entities
}

