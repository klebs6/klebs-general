crate::ix!();

//------------------------------------[alternative-payment-technologies]

/// Represents distinct alternative and complementary payment technologies adopted regionally.
pub enum AlternativePaymentTechnology {
    /// Independent national credit card networks (e.g., Barclaycard in U.K., Bankcard in Australia).
    IndependentNationalNetwork(NationalCardNetwork),

    /// Stored-value cards, commonly used as cash alternatives (e.g., telephone cards in Japan).
    StoredValueCard,

    /// RFID-based systems integrated into various everyday objects.
    RfidPaymentSystem(RfidIntegrationType),
}

/// Identifies specific national card networks distinct from global networks.
pub enum NationalCardNetwork {
    BarclaycardUK,
    BankcardAustralia,
}

/// Represents integration types for RFID-based payment systems.
pub enum RfidIntegrationType {
    Cards,
    Cellphones,
    OtherObjects,
}
