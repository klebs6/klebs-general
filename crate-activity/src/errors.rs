crate::ix!();

error_tree!{

    pub enum PcaError {
        NoActivityDataAvailable,
        PcaDataLengthMismatch {
            expected_num_elements: usize,
            found_num_elements:    usize,
        }
    }

    pub enum CrateActivityError {
        Reqwest(reqwest::Error),
        Serde(serde_json::Error),
        Io(std::io::Error),
        ShapeError(ndarray::ShapeError),
        HierarchicalClusteringError(HierarchicalClusteringError),
        PcaError(PcaError),
    }
}
