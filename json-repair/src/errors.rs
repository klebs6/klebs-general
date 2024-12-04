crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum JsonRepairError {
        FailedToParseRepairedJson {
            details: String,
        },
        Unrepairable(String),
        AllAttemptedRepairsFailed,
        CouldNotConvertTheOutputOfDuplicateQuoteRemovalToJson,

        #[cmp_neq]
        SerdeParseError {
            inner: serde_json::Error
        },
    }
}
