// ---------------- [ File: src/report.rs ]
crate::ix!();

pub trait GenerateReport {

    type Report;
    type Error;

    fn generate_report(&self) -> Result<Self::Report,Self::Error>;
}
