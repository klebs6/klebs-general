crate::ix!();

//------------------------------------[credit-card-variants]

/// Enumerates distinct variants of credit cards, each potentially requiring specific implementation details.
pub enum CreditCardVariant {
    /// Personal revolving credit card issued by banks and honored by financial institution networks.
    PersonalRevolving,

    /// Organization-branded credit card, associated explicitly with a sponsoring organization.
    OrganizationBranded,

    /// Corporate-user credit card issued to corporate entities and their authorized employees.
    CorporateUser,

    /// Store-specific credit card usable only at the issuing retail or merchant outlets.
    StoreSpecific,
}

/// Represents the primary utility provided by credit cards.
pub struct CreditCardUtility {
    /// Indicates credit card portability and independence from local banking facilities.
    portable_credit_facility: bool,

    /// Specifies the credit card variant explicitly associated with this utility.
    variant: CreditCardVariant,
}

//------------------------------------[international-expansion]

/// Represents historical international expansion milestones for credit cards.
pub struct InternationalExpansionMilestone {
    /// Name of the card introduced (e.g., Barclaycard).
    card_name_ascii: Vec<u8>, // Numeric ASCII storage, e.g., "Barclaycard"

    /// Year of introduction.
    introduction_year: u16,

    /// Country where the credit card was introduced internationally.
    country_of_introduction_ascii: Vec<u8>,
}

//------------------------------------[regional-adoption-patterns]

/// Represents the various factors influencing regional adoption patterns for credit cards.
pub struct RegionalAdoptionFactors {
    /// Cultural influence significantly affecting regional adoption.
    cultural_influence_strong: bool,

    /// Regulatory environment affecting card adoption practices.
    strict_banking_regulations: bool,

    /// Early adoption of enhanced security technologies such as chip-based cards.
    early_chip_adoption: bool,

    /// Preference for alternative payment methods (debit cards, online banking, installment plans).
    alternative_payment_methods_preferred: bool,

    /// Banking infrastructure maturity influencing adoption speed.
    mature_banking_infrastructure: bool,
}

/// Enumerates representative regional adoption patterns for credit cards.
pub enum Region {
    /// Regions achieving high adoption rapidly (U.S., Canada, U.K., Australia, New Zealand).
    HighRapidAdoption,

    /// Regions initially favoring alternative systems (France, Germany, Switzerland, etc.).
    AlternativeSystemsPreferred,

    /// Regions adopting advanced chip-based security measures early (e.g., France).
    EarlyChipAdopters,

    /// Regions heavily reliant on alternative payment methods, limiting credit card adoption.
    AlternativePaymentsDominant,
}

/// Represents overall international credit card adoption, including specific regional patterns.
pub struct InternationalAdoptionSpec {
    /// Specific region characterized by the described adoption pattern.
    region: Region,

    /// Detailed factors influencing adoption patterns in the region.
    factors: RegionalAdoptionFactors,
}

//------------------------------------[historical-global-credit-card-system]

/// Comprehensive representation encapsulating global credit card utility, variants, international milestones, and regional adoption patterns.
pub struct HistoricalGlobalCreditCardSystem {
    /// Primary utility provided by the credit card variant.
    utility: CreditCardUtility,

    /// Historical international expansion milestones (e.g., Barclaycard in 1966).
    international_expansion: Option<InternationalExpansionMilestone>,

    /// International adoption pattern explicitly characterized by region-specific factors.
    regional_adoption: InternationalAdoptionSpec,
}
