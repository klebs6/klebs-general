crate::ix!();

//------------------------------------[card-material-evolution]

/// Enumerates historical and modern materials used for credit card manufacture.
pub enum CardMaterialEvolution {
    /// Modern standard material: Polyvinyl Chloride (PVC).
    ModernPVC,

    /// Earliest form: Celluloid plastic.
    HistoricalCelluloid,

    /// Metal-based card materials.
    HistoricalMetal,

    /// Fiber-based card materials historically used.
    HistoricalFiber,

    /// Paper-based cards historically issued by merchants.
    HistoricalPaper,
}

/// Specifies the embedded smart card chip's material composition.
pub struct EmbeddedChipComposition {
    /// Metal components of the embedded chip, distinct from card body material.
    chip_material: ChipMaterial,
}

/// Enumerates typical materials for embedded smart card chips.
pub enum ChipMaterial {
    Metal,
}
