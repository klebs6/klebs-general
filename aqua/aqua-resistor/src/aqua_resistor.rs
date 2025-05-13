crate::ix!();

/// -------------------------------------------
/// Core Resistor Struct
/// -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct Resistor {
    resistor_id:                    ResistorId,
    manufacturer_info:              Option<ManufacturerInfo>,
    electrical_characteristics:     ElectricalCharacteristics,
    physical_dimensions:            PhysicalDimensions,
    thermal_environmental:          ThermalEnvironmentalCharacteristics,
    frequency_characteristics:      Option<FrequencyCharacteristics>,
    aging_characteristics:          Option<AgingCharacteristics>,
    performance_characteristics:    PerformanceCharacteristics,
    packaging_assortment:           PackagingAssortment,
    verification_record_ids:        Vec<VerificationRecordId>,
}

/// -------------------------------------------
/// Type Aliases
/// -------------------------------------------
pub type ResistorId           = Uuid;
pub type VerificationRecordId = Uuid;

/// -------------------------------------------
/// Manufacturer Info
/// -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct ManufacturerInfo {
    name:                   String,
    part_number:            String,
    batch_number:           String,
    manufacture_date:       DateTime<Utc>,
    manufacturing_location: WorldAddress,
}

/// -------------------------------------------
/// Electrical Characteristics
/// -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct ElectricalCharacteristics {
    nominal_resistance_ohms:            f64,
    tolerance_percent:                  f64,
    rated_power_watts:                  f64,
    temperature_coefficient_ppm_per_c:  f64,
    voltage_coefficient_ppm_per_v:      Option<f64>,
    noise_figure_db:                    Option<f64>,
    excess_noise_index_db:              Option<f64>,
}

/// -------------------------------------------
/// Physical Dimensions
/// -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct PhysicalDimensions {
    length_mm:          f64,
    diameter_mm:        f64,
    lead_length_mm:     f64,
    lead_spacing_mm:    Option<f64>,
    lead_gauge_awg:     Option<u32>,
    package_type:       ResistorPackageType,
}

/// -------------------------------------------
/// Thermal & Environmental Characteristics
/// -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct ThermalEnvironmentalCharacteristics {
    thermal_resistance_c_per_w: Option<f64>,
    max_operating_temp_c:       f64,
    operating_temp_range_c:     RangeInclusive<f64>,
    max_humidity_percent:       Option<f64>,
    vibration_tolerance_g:      Option<f64>,
    shock_resistance_g:         Option<f64>,
}

/// -------------------------------------------
/// Frequency Characteristics
/// -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct FrequencyCharacteristics {
    max_operational_frequency_hz:   f64,
    self_inductance_henry:          Option<f64>,
    parasitic_capacitance_farads:   Option<f64>,
}

/// -------------------------------------------
/// Aging Characteristics
/// -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct AgingCharacteristics {
    resistance_drift_per_year_ohms: Option<f64>,
    shelf_life_years:               Option<f64>,
    eol_resistance_threshold_ohms:  Option<f64>,
}

/// -------------------------------------------
/// Performance Characteristics (Little Rebels Integration)
/// -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct PerformanceCharacteristics {
    stability:              ResistorStability,
    life_expectancy:        ResistorLifeExpectancy,
    derating_info:          DeratingInfo,
    application_suitability: ApplicationSuitability,
}

/// -------------------------------------------
/// Packaging Assortment & Availability
/// -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct PackagingAssortment {
    availability:        AssortmentAvailability,
    values_per_decade:   u32,
    quantity_per_reel:   QuantityPerReel,
}

/// -------------------------------------------
/// Supporting Enums and Structs
/// -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResistorPackageType {
    Axial,
    Radial,
    SurfaceMount(SmdSize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmdSize {
    length_mm: f64,
    width_mm:  f64,
    height_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResistorStability {
    HighStability,
    StandardStability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResistorLifeExpectancy {
    LongLife,
    StandardLife,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeratingInfo {
    start_temp_celsius: f32,
    end_temp_celsius:   f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApplicationSuitability {
    LowPowerDissipationSteadyState,
    GeneralPurpose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssortmentAvailability {
    OrganizedInResistorCabinetCollections,
    BulkPackaging,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityPerReel {
    oj: u32,
    ok: u32,
    ol: u32,
    om: u32,
    on: u32,
}
