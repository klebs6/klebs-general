// ---------------- [ File: generate-report/src/lib.rs ]

pub trait GenerateReport {

    type Report;
    type Error;

    fn generate_report(&self) -> Result<Self::Report,Self::Error>;
}

