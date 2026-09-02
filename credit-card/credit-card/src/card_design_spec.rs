crate::ix!();

//------------------------------------[card-collectibility-and-design]

/// Represents aesthetic and collectible aspects of credit card design.
pub struct CardDesignSpec {
    /// Indicates whether the card has marketing-driven design aesthetics.
    marketing_focused_design: bool,

    /// Indicates whether the card is intentionally differentiated for collectible purposes.
    collectible_design: bool,

    /// Design aesthetic variations.
    design_variant: DesignVariant,
}

/// Enumerates potential design aesthetics used for marketing and differentiation.
pub enum DesignVariant {
    StandardCorporate,
    ArtisticOrUnique,
    LimitedEdition,
}
