// ---------------- [ File: hydro2-async-scheduler/src/async_scheduler_config.rs ]
crate::ix!();

/// Configuration for our upgraded asynchronous scheduler.
#[derive(Debug, Builder, Clone, Getters, Setters)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct AsyncSchedulerConfig {

    /// Maximum concurrency (thread pool size / parallel tasks).
    #[builder(default = "4")]
    max_parallelism: usize,

    /// Determines wave-chunk scheduling vs. immediate node scheduling.
    #[builder(default = "BatchingStrategy::Immediate")]
    batching_strategy: BatchingStrategy,

    /// If true, we will spawn a channel to stream each node’s outputs.
    #[builder(default = "false")]
    enable_streaming: bool,

    /// If present, invoked after each node completes to record progress.
    #[builder(default)]
    checkpoint_callback: Option<Arc<dyn CheckpointCallback>>,
}

/// A macro to build an `AsyncSchedulerConfig` without using `.unwrap()` or `.expect()`.
/// It returns a `Result<AsyncSchedulerConfig, NetworkError>` so you can use `?`.
/// 
/// This macro also **auto-upcasts** `checkpoint_callback` if the user writes
/// `Some(Arc::new(YourCallbackType { ... }))`. 
///
/// ### Usage
/// ```ignore
/// let checkpoints = Arc::new(AsyncMutex::new(Vec::<Vec<usize>>::new()));
///
/// let cfg = try_build_async_scheduler_config!(
///     max_parallelism     = 4,
///     batching_strategy   = BatchingStrategy::Immediate,
///     enable_streaming    = true,
///     // No manual upcasting needed here:
///     checkpoint_callback = Some(Arc::new(MockCheckpointCallback {
///         checkpoints
///     }))
/// )?;
///
/// // Now `cfg.checkpoint_callback()` is `Some(Arc<dyn CheckpointCallback>)`.
/// ```
#[macro_export]
macro_rules! try_build_async_scheduler_config {
    // Entry rule: gather `field = value` pairs.
    ( $( $field:ident = $value:expr ),* $(,)? ) => {{
        let mut builder = $crate::AsyncSchedulerConfigBuilder::default();
        $(
            try_build_async_scheduler_config!(@assign builder, $field, $value);
        )*
        match builder.build() {
            Ok(cfg) => Ok(cfg),
            Err(e) => Err(NetworkError::AsyncSchedulerConfigBuilderFailure),
        }
    }};

    //=== Special Handling for checkpoint_callback ===//

    // If user explicitly sets `checkpoint_callback = None`
    (@assign $builder:ident, checkpoint_callback, None) => {{
        $builder.checkpoint_callback(None);
    }};

    // If user explicitly sets `checkpoint_callback = Some(...)`
    // We upcast the inside of Some(...) to Arc<dyn CheckpointCallback>.
    (@assign $builder:ident, checkpoint_callback, Some($val:expr)) => {{
        // Coerce e.g. Arc<MockCheckpointCallback> -> Arc<dyn CheckpointCallback>
        let upcasted: ::std::sync::Arc<dyn $crate::CheckpointCallback> = $val;
        $builder.checkpoint_callback(Some(upcasted));
    }};

    // If user sets `checkpoint_callback = <expression>`, we interpret that
    // as "wrap it in `Some(...)`, upcast to Arc<dyn CheckpointCallback>."
    // e.g. `checkpoint_callback = Arc::new(... MyCallback ...)`.
    (@assign $builder:ident, checkpoint_callback, $val:expr) => {{
        let upcasted: ::std::sync::Arc<dyn $crate::CheckpointCallback> = $val;
        $builder.checkpoint_callback(Some(upcasted));
    }};

    //=== Fallback: assign other fields normally ===//
    (@assign $builder:ident, $field:ident, $value:expr) => {{
        $builder.$field($value);
    }};
}
