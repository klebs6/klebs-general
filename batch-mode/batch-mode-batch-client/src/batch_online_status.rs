// ---------------- [ File: src/batch_online_status.rs ]
crate::ix!();

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub struct BatchOnlineStatus {
    output_file_available: bool,
    error_file_available:  bool,
}

impl From<&Batch> for BatchOnlineStatus {

    fn from(batch: &Batch) -> Self {
        Self {
            output_file_available: batch.output_file_id.is_some(),
            error_file_available:  batch.error_file_id.is_some(),
        }
    }
}

impl BatchOnlineStatus {

    pub fn output_file_available(&self) -> bool { 
        self.output_file_available 
    }

    pub fn error_file_available(&self) -> bool { 
        self.error_file_available 
    }
}
