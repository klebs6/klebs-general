crate::ix!();

//------------------------------------[historical-card-variants]

/// Enumerates historical credit instrument types predating modern credit cards.
pub enum HistoricalCreditInstrument {
    /// Early charge cards existing in various shapes, sizes, and materials.
    EarlyChargeCard(EarlyChargeCardSpec),

    /// Charge coins—small, round metal tokens associated with customer accounts.
    ChargeCoin(ChargeCoinSpec),

    /// Charga-Plate system consisting of embossed metal plates used in retail transactions.
    ChargaPlate(ChargaPlateSpec),
}

/// Early charge card specifications, historically available in diverse materials and form factors.
pub struct EarlyChargeCardSpec {
    /// Material composition of the early charge card.
    material: HistoricalMaterial,

    /// Physical form factor description.
    form_factor: CardFormFactor,
}

/// Enumerates historical materials used in early credit instruments.
pub enum HistoricalMaterial {
    Celluloid,
    Copper,
    Aluminum,
    Steel,
    OtherMetal,
}

/// Describes the general physical form factor of early charge cards.
pub enum CardFormFactor {
    Rectangular,
    Square,
    OtherIrregularShape,
}

/// Charge coin specifications historically used as credit tokens in merchant transactions.
pub struct ChargeCoinSpec {
    /// Unique numeric charge account number associated with the customer account.
    charge_account_number: Vec<u8>,

    /// Merchant’s identifying details embossed on the coin.
    merchant_identification: MerchantIdentification,

    /// Indicates physical hole for key-ring attachment.
    key_ring_hole_present: bool,
}

/// Represents embossed merchant identification details.
pub struct MerchantIdentification {
    /// Numeric or symbolic representation of the merchant's identity (historically embossed).
    identifier: Vec<u8>,
}

/// Charga-Plate specifications historically used for customer identification in retail transactions.
pub struct ChargaPlateSpec {
    /// Embossed customer identification details (name, city, and state).
    customer_identification: CustomerIdentification,

    /// Indicates presence of a signature card slot on the reverse side.
    signature_card_slot_present: bool,

    /// Indicates whether the plate was customer-held or stored on merchant premises.
    storage_location: PlateStorageLocation,
}

/// Represents embossed customer details on Charga-Plates.
pub struct CustomerIdentification {
    /// Embossed representation of customer's name (stored numerically as ASCII codes).
    name_ascii: Vec<u8>,

    /// Embossed representation of customer's city (stored numerically as ASCII codes).
    city_ascii: Vec<u8>,

    /// Embossed representation of customer's state (stored numerically as ASCII codes).
    state_ascii: Vec<u8>,
}

/// Specifies whether a Charga-Plate was customer-held or stored by merchants.
pub enum PlateStorageLocation {
    CustomerHeld,
    MerchantStored,
}

/// Describes historical transaction processing workflow using Charga-Plates.
pub struct ChargaPlateTransactionProcess {
    /// Plate positioning into imprinter recess step.
    plate_positioned_into_imprinter: bool,

    /// Paper charge slip placed over plate.
    charge_slip_placed_over_plate: bool,

    /// Inked ribbon pressed against slip to record transaction.
    imprinting_completed: bool,
}

/// Represents overall historical attributes and their integration within a legacy credit system.
pub struct HistoricalCreditSystem {
    instrument: HistoricalCreditInstrument,

    /// Optional description of the physical transaction workflow (primarily for Charga-Plate systems).
    transaction_process: Option<ChargaPlateTransactionProcess>,
}

//------------------------------------[early-general-purpose-charge-cards]

/// Enumerates early general-purpose charge cards, clearly differentiating historical card brands.
pub enum EarlyGeneralPurposeChargeCard {
    DinersClub(HistoricDinersClubSpec),
    CarteBlanche(HistoricCarteBlancheSpec),
    AmericanExpress(HistoricAmericanExpressSpec),
}

/// Diners Club card specification, historically introduced in 1950.
pub struct HistoricDinersClubSpec {
    /// Allowed transactions across multiple merchants.
    multi_merchant_support: bool,

    /// Initially required full repayment of outstanding balance each billing cycle.
    full_balance_repayment_required: bool,
}

/// Carte Blanche card specification, historically introduced in 1958.
pub struct HistoricCarteBlancheSpec {
    /// Enabled international transaction support across a network of merchants.
    international_network_support: bool,

    /// Payment method initially required full balance repayment each billing cycle.
    /// Later evolved to support revolving credit.
    payment_method: HistoricalPaymentMethod,
}

/// American Express card specification, historically introduced in 1958.
pub struct HistoricAmericanExpressSpec {
    /// Enabled international transaction support across a network of merchants.
    international_network_support: bool,

    /// Payment method initially required full balance repayment each billing cycle.
    /// Later evolved to support revolving credit.
    payment_method: HistoricalPaymentMethod,
}

/// Represents the payment method historically associated with early charge cards.
pub enum HistoricalPaymentMethod {
    /// Required complete repayment of outstanding balance each billing cycle.
    FullBalanceRepayment,

    /// Allowed carrying forward balances (revolving credit), introduced at a later stage.
    RevolvingCredit,
}

/// Comprehensive historical representation of early general-purpose charge cards,
/// explicitly capturing initial operational characteristics and later evolutions.
pub struct HistoricalChargeCardSystem {
    /// Specific charge card variant (Diners Club, Carte Blanche, American Express).
    card_variant: EarlyGeneralPurposeChargeCard,

    /// Year of introduction.
    introduction_year: u16,
    
    /// Indicates if revolving credit was integrated at a later stage.
    revolving_credit_introduced_later: bool,
}

//------------------------------------[bankamericard-historical]

/// Historical credit card system representing BankAmericard,
/// launched by Bank of America in Fresno, California (1958).
///
/// Notable for successfully addressing the merchant-consumer adoption challenge
/// and evolving into the global \"Visa\" network in 1976.
pub struct BankAmericardHistoricalSpec {
    /// Original introduction details.
    introduction_context: BankAmericardIntroductionContext,

    /// Adoption strategy details emphasizing simultaneous consumer issuance and merchant acceptance.
    adoption_strategy: AdoptionStrategySpec,

    /// Operational details involving third-party bank issuers.
    third_party_issuance: ThirdPartyIssuanceSpec,

    /// Expansion strategy via nationwide and international licensing agreements.
    licensing_expansion: LicensingExpansionSpec,

    /// Rebranding information as \"Visa\" (1976).
    global_rebranding: GlobalRebrandingSpec,
}

/// Contextual details of BankAmericard's introduction.
pub struct BankAmericardIntroductionContext {
    /// Year of introduction (1958).
    introduction_year: u16,

    /// Introducing financial institution.
    introducing_bank: IntroducingBank,

    /// Geographic launch location.
    initial_launch_location: LaunchLocation,
}

/// Enumeration identifying the introducing financial institution.
pub enum IntroducingBank {
    BankOfAmerica,
}

/// Represents geographic launch details.
pub struct LaunchLocation {
    city_ascii: Vec<u8>,
    state_ascii: Vec<u8>,
}

/// Represents BankAmericard's adoption strategy explicitly addressing
/// the \"chicken-and-egg\" problem of merchant-consumer acceptance.
pub struct AdoptionStrategySpec {
    /// Initial number of consumer cards issued simultaneously to incentivize merchant adoption.
    initial_consumer_base_size: u32,

    /// Indicates simultaneous consumer and merchant targeting strategy.
    simultaneous_adoption_strategy: bool,
}

/// Specifies third-party issuance operational model of BankAmericard.
pub struct ThirdPartyIssuanceSpec {
    /// Indicates support for third-party financial institution issuance.
    third_party_bank_issuance_supported: bool,

    /// Indicates that card acceptance was not restricted to individual merchants.
    merchant_acceptance_universal: bool,
}

/// Represents BankAmericard's nationwide and international expansion through licensing.
pub struct LicensingExpansionSpec {
    /// Indicates nationwide expansion via third-party licensing.
    nationwide_licensing_expansion: bool,

    /// Indicates international expansion via third-party licensing.
    international_licensing_expansion: bool,
}

/// Represents global rebranding details under the \"Visa\" name.
pub struct GlobalRebrandingSpec {
    /// Year of global unification and rebranding under the \"Visa\" brand (1976).
    rebranding_year: u16,

    /// Indicates successful global integration of previously regional/national variants.
    globally_unified: bool,

    /// New unified global brand name after rebranding.
    global_brand_name_ascii: Vec<u8>, // \"Visa\" stored numerically as ASCII codes
}

/// Comprehensive historical representation encapsulating all aspects
/// of the BankAmericard historical credit system.
pub struct HistoricalBankAmericardSystem {
    historical_spec: BankAmericardHistoricalSpec,
}

//------------------------------------[master-charge-historical]

/// Historical representation of Master Charge, initially introduced in 1966
/// as a collaborative bank initiative to compete directly with BankAmericard.
///
/// Significantly expanded in 1969 following Citibank's merger of its \"Everything Card,\"
/// later evolving into the modern \"MasterCard\" brand.
pub struct MasterChargeHistoricalSpec {
    /// Initial introduction details, including collaboration among banks.
    introduction_context: MasterChargeIntroductionContext,

    /// Details of significant expansion event (Citibank's merger of \"Everything Card\").
    expansion_via_merger: Option<MasterChargeExpansionSpec>,

    /// Historical rebranding to \"MasterCard\".
    rebranding_spec: MasterChargeRebrandingSpec,
}

/// Contextual introduction details of Master Charge.
pub struct MasterChargeIntroductionContext {
    /// Year of initial introduction (1966).
    introduction_year: u16,

    /// Collaboration details indicating multiple founding banks.
    collaborative_origins: CollaborativeOriginsSpec,

    /// Identified primary competitor (BankAmericard).
    primary_competitor: HistoricalCompetitor,
}

/// Represents collaborative origins involving multiple founding banks.
pub struct CollaborativeOriginsSpec {
    /// Number of initial collaborating banks.
    number_of_banks: u32,

    /// Indicates formation as a joint venture among banks.
    joint_venture: bool,
}

/// Enumerates historically recognized primary competitors.
pub enum HistoricalCompetitor {
    BankAmericard,
    Other(Vec<u8>), // Numeric representation for other competitors
}

/// Details significant expansion event via Citibank's \"Everything Card\" merger.
pub struct MasterChargeExpansionSpec {
    /// Year Citibank merged its existing \"Everything Card\" into Master Charge (1969).
    merger_year: u16,

    /// Financial institution involved in the merger (Citibank).
    merging_institution: MergingInstitution,

    /// Name of the card merged (\"Everything Card\").
    merged_card_name_ascii: Vec<u8>, // Stored numerically as ASCII codes
}

/// Enumerates financial institutions historically merging into Master Charge.
pub enum MergingInstitution {
    Citibank,
    Other(Vec<u8>),
}

/// Details historical rebranding from \"Master Charge\" to \"MasterCard\".
pub struct MasterChargeRebrandingSpec {
    /// Indicates rebranding event occurred.
    rebranded: bool,

    /// New brand name (\"MasterCard\") stored numerically as ASCII codes.
    new_brand_name_ascii: Vec<u8>,
}

/// Comprehensive historical representation encapsulating all details of the Master Charge system.
pub struct HistoricalMasterChargeSystem {
    historical_spec: MasterChargeHistoricalSpec,
}


//------------------------------------[visa-computerization-and-verification]

/// Represents Visa's historical introduction of computerized transaction processing in 1973,
/// significantly reducing transaction processing times.
pub struct VisaComputerizationHistoricalSpec {
    /// Year Visa implemented computerized transaction processing (1973).
    computerization_year: u16,

    /// Indicates significant reduction in transaction processing times due to computerization.
    significantly_reduced_transaction_times: bool,
}

/// Represents early manual verification practices for credit card transactions,
/// common before widespread adoption of always-connected payment terminals.
pub struct EarlyTransactionVerificationSpec {
    /// Merchants conducted phone-based verifications for higher-risk transactions.
    phone_verification: PhoneVerificationSpec,

    /// Manual distribution and usage of stolen card number lists.
    stolen_card_list_distribution: StolenCardListDistributionSpec,

    /// Signature-based manual verification practices.
    signature_matching: SignatureMatchingSpec,

    /// Liability and risk assumptions by merchants.
    merchant_liability_spec: MerchantLiabilitySpec,
}

/// Represents historical phone-based transaction verification details.
pub struct PhoneVerificationSpec {
    /// Merchants typically verified transactions over a certain monetary threshold.
    verification_threshold_exists: bool,

    /// Merchants typically verified transactions involving unknown customers.
    unknown_customer_verification_required: bool,
}

/// Represents the historical manual distribution of stolen card number lists to merchants.
pub struct StolenCardListDistributionSpec {
    /// Indicates manual (non-digital) distribution method.
    manual_distribution: bool,

    /// Indicates requirement for merchants to cross-check card numbers against lists.
    merchant_required_to_check_lists: bool,
}

/// Represents historical signature matching practices on charge slips.
pub struct SignatureMatchingSpec {
    /// Indicates that merchants matched customer signatures on charge slips to signatures on cards.
    manual_signature_matching_required: bool,
}

/// Represents merchant financial liability and assumptions of risk in fraudulent transactions.
pub struct MerchantLiabilitySpec {
    /// Merchants held liable for fraudulent transactions if verification steps were skipped.
    merchant_liable_if_no_verification: bool,

    /// Merchants frequently assumed risk for small-value transactions to avoid cumbersome verifications.
    merchants_commonly_skipped_verification_for_small_transactions: bool,
}

/// Comprehensive historical representation encompassing Visa computerization and early manual transaction verification processes.
pub struct HistoricalVisaTransactionSystem {
    visa_computerization: VisaComputerizationHistoricalSpec,
    early_verification: EarlyTransactionVerificationSpec,
}

