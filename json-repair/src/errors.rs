crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum JsonRepairError {
        UnexpectedEOF,
        UnexpectedToken,
        InvalidToken(String),
        InvalidNumber(String),
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
