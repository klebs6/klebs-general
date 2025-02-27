// ---------------- [ File: src/async_scheduler.rs ]
crate::ix!();

#[derive(Getters,Builder,Debug, Clone)]
#[getset(get="pub")]
#[builder(setter(into))]
pub struct AsyncScheduler {
    config: AsyncSchedulerConfig,
}

impl AsyncScheduler {

    /// Helper that builds an `AsyncScheduler` with provided config.
    pub fn with_config(cfg: AsyncSchedulerConfig) -> AsyncScheduler {
        AsyncSchedulerBuilder::default()
            .config(cfg)
            .build()
            .unwrap() // building a struct: we assume it never fails in normal usage
    }

    /// Test-only constructor that lets us pick a BatchingStrategy quickly.
    pub fn new_test(strategy: BatchingStrategy) -> Result<Self, NetworkError> {
        // You can define a config with default or typical test settings
        let config_result = AsyncSchedulerConfigBuilder::default()
            .batching_strategy(strategy)
            .max_parallelism(2_usize)      // or any small number
            .enable_streaming(false)
            .build();

        // Avoid .unwrap(), return an error if config fails to build
        let config = match config_result {
            Ok(cfg) => cfg,
            Err(_builder_err) => {
                return Err(NetworkError::AsyncSchedulerConfigBuilderFailure);
            }
        };

        let scheduler = AsyncScheduler { config };
        Ok(scheduler)
    }
}
