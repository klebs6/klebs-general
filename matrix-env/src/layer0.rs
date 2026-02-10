// src/layer0.rs

crate::ix!();

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Layer0ShutdownReason {
    CtrlC,
    Sigterm,
    InternalRequest,
}

impl Layer0ShutdownReason {
    fn as_str(&self) -> &'static str {
        match self {
            Self::CtrlC => "ctrl_c",
            Self::Sigterm => "sigterm",
            Self::InternalRequest => "internal_request",
        }
    }
}

#[derive(Clone, Debug)]
struct ShutdownState {
    requested: bool,
    reason: Option<Layer0ShutdownReason>,
}

impl Default for ShutdownState {
    fn default() -> Self {
        Self {
            requested: false,
            reason: None,
        }
    }
}

#[derive(Clone, Debug)]
struct Layer0ShutdownTrigger {
    tx: watch::Sender<ShutdownState>,
}

impl Layer0ShutdownTrigger {
    fn request_shutdown(&self, reason: Layer0ShutdownReason) {
        let mut current = (*self.tx.borrow()).clone();

        if current.requested {
            debug!(
                shutdown_reason = current
                    .reason
                    .as_ref()
                    .map(|r| r.as_str())
                    .unwrap_or("unknown"),
                new_reason = reason.as_str(),
                "shutdown already requested; ignoring subsequent request"
            );
            return;
        }

        current.requested = true;
        current.reason = Some(reason.clone());

        if let Err(_send_err) = self.tx.send(current) {
            warn!(shutdown_reason = reason.as_str(), "shutdown request send failed (no receivers)");
        } else {
            info!(shutdown_reason = reason.as_str(), "shutdown requested");
        }
    }
}

#[derive(Clone, Debug)]
struct Layer0ShutdownWatch {
    rx: watch::Receiver<ShutdownState>,
}

impl Layer0ShutdownWatch {
    fn is_shutdown_requested(&self) -> bool {
        self.rx.borrow().requested
    }

    async fn wait_for_shutdown(&mut self) -> Result<Layer0ShutdownReason, Layer0Error> {
        loop {
            let snapshot = (*self.rx.borrow()).clone();
            if snapshot.requested {
                return snapshot
                    .reason
                    .ok_or(Layer0Error::ShutdownStateMissingReason);
            }

            self.rx.changed()
                .await
                .map_err(|_closed| Layer0Error::ShutdownChannelClosed)?;
        }
    }
}

#[derive(Clone, Debug)]
struct Layer0ShutdownGate {
    trigger: Layer0ShutdownTrigger,
    watch: Layer0ShutdownWatch,
}

impl Layer0ShutdownGate {
    fn new() -> Self {
        let (tx, rx) = watch::channel(ShutdownState::default());
        Self {
            trigger: Layer0ShutdownTrigger { tx },
            watch: Layer0ShutdownWatch { rx },
        }
    }

    fn trigger(&self) -> Layer0ShutdownTrigger {
        self.trigger.clone()
    }

    fn watch(&self) -> Layer0ShutdownWatch {
        self.watch.clone()
    }
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "matrix-term",
    about = "Layer0 runtime skeleton (no Matrix protocol yet)"
)]
pub struct Layer0Cli {
    /// Override tracing filter directives (e.g. "info,matrix_term=debug").
    /// If absent, RUST_LOG is used (and if that’s absent, defaults to "info").
    #[structopt(long = "log-filter")]
    log_filter: Option<String>,

    /// Emit logs as JSON (useful for journald / log shippers).
    #[structopt(long = "log-json")]
    log_json: bool,

    /// Heartbeat tick interval in milliseconds (debug logs).
    #[structopt(long = "heartbeat-ms", default_value = "500")]
    heartbeat_ms: u64,

    /// Grace period for background tasks to stop after shutdown is requested.
    #[structopt(long = "shutdown-grace-ms", default_value = "1500")]
    shutdown_grace_ms: u64,
}

#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct Layer0ValidatedConfig {
    log: Layer0LogConfig,
    heartbeat_interval: Duration,
    shutdown_grace: Duration,
}

#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct Layer0LogConfig {
    filter_directives: Option<String>,
    json: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Layer0CliValidationError {
    HeartbeatMsMustBePositive,
    ShutdownGraceMsMustBePositive,
}

impl fmt::Display for Layer0CliValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeartbeatMsMustBePositive => {
                write!(f, "heartbeat-ms must be a positive, non-zero integer")
            }
            Self::ShutdownGraceMsMustBePositive => {
                write!(f, "shutdown-grace-ms must be a positive, non-zero integer")
            }
        }
    }
}

impl std::error::Error for Layer0CliValidationError {}

impl Layer0Cli {
    pub fn parse_from_env() -> Result<Self, Layer0Error> {
        Self::from_iter_safe(std::env::args()).map_err(Layer0Error::CliParse)
    }

    pub fn validate(&self) -> Result<Layer0ValidatedConfig, Layer0Error> {
        if self.heartbeat_ms == 0 {
            return Err(Layer0Error::CliValidation(
                Layer0CliValidationError::HeartbeatMsMustBePositive,
            ));
        }
        if self.shutdown_grace_ms == 0 {
            return Err(Layer0Error::CliValidation(
                Layer0CliValidationError::ShutdownGraceMsMustBePositive,
            ));
        }

        Ok(Layer0ValidatedConfig {
            log: Layer0LogConfig {
                filter_directives: self.log_filter.clone(),
                json: self.log_json,
            },
            heartbeat_interval: Duration::from_millis(self.heartbeat_ms),
            shutdown_grace: Duration::from_millis(self.shutdown_grace_ms),
        })
    }
}

#[derive(Debug)]
pub enum Layer0Error {
    CliParse(structopt::clap::Error),
    CliValidation(Layer0CliValidationError),
    EnvFilterParse(EnvFilterParseError),
    TracingInit(Box<dyn std::error::Error + Send + Sync>),
    TokioRuntimeBuild(io::Error),
    ShutdownChannelClosed,
    ShutdownStateMissingReason,
    TaskJoin {
        task_name: &'static str,
        join_error: JoinError,
    },
    TaskTimeout {
        task_name: &'static str,
        waited: Duration,
    },
}

impl fmt::Display for Layer0Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CliParse(e) => write!(f, "cli parse error: {e}"),
            Self::CliValidation(e) => write!(f, "cli validation error: {e}"),
            Self::EnvFilterParse(e) => write!(f, "invalid log filter directives: {e}"),
            Self::TracingInit(e) => write!(f, "failed to set tracing subscriber: {e}"),
            Self::TokioRuntimeBuild(e) => write!(f, "failed to build tokio runtime: {e}"),
            Self::ShutdownChannelClosed => write!(f, "shutdown channel closed unexpectedly"),
            Self::ShutdownStateMissingReason => write!(f, "shutdown requested but reason missing"),
            Self::TaskJoin { task_name, join_error } => {
                write!(f, "task join failed ({task_name}): {join_error}")
            }
            Self::TaskTimeout { task_name, waited } => write!(
                f,
                "task failed to stop within grace period ({task_name}, waited {:?})",
                waited
            ),
        }
    }
}

impl std::error::Error for Layer0Error {}


pub struct Layer0Entrypoint;

impl Layer0Entrypoint {
    pub fn run_from_env() -> ExitCode {
        match Self::try_run_from_env() {
            Ok(code) => code,
            Err(e) => {
                error!(error = %e, "layer0 failed");
                ExitCode::FAILURE
            }
        }
    }

    pub fn try_run_from_env() -> Result<ExitCode, Layer0Error> {
        let cli = Layer0Cli::parse_from_env()?;
        Self::try_run(cli)
    }

    pub fn try_run(cli: Layer0Cli) -> Result<ExitCode, Layer0Error> {
        let cfg = cli.validate()?;

        install_tracing(cfg.log())?;

        let span = tracing::info_span!(
            "layer0",
            heartbeat_ms = cfg.heartbeat_interval().as_millis() as u64,
            shutdown_grace_ms = cfg.shutdown_grace().as_millis() as u64,
            log_json = cfg.log().json(),
            log_filter = cfg
                .log()
                .filter_directives()
                .as_deref()
                .unwrap_or("<env_or_default>")
        );
        let _guard = span.enter();

        info!("starting layer0 runtime");

        let rt = build_tokio_runtime()?;
        rt.block_on(async move {
            let orchestrator = Layer0Orchestrator::new(cfg);
            orchestrator.run(wait_for_os_shutdown()).await
        })
    }
}

fn build_tokio_runtime() -> Result<TokioRuntime, Layer0Error> {
    TokioRuntimeBuilder::new_multi_thread()
        .enable_all()
        .thread_name("matrix-term")
        .build()
        .map_err(Layer0Error::TokioRuntimeBuild)
}

fn install_tracing(cfg: &Layer0LogConfig) -> Result<(), Layer0Error> {
    let env_filter = match cfg.filter_directives() {
        Some(directives) => EnvFilter::try_new(directives).map_err(Layer0Error::EnvFilterParse)?,
        None => EnvFilter::try_from_default_env().unwrap_or_else(|_e| EnvFilter::new("info")),
    };

    let base = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true);

    let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = if *cfg.json() {
        base.json().try_init().map_err(|e| e)
    } else {
        base.try_init().map_err(|e| e)
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            if tracing::dispatcher::has_been_set() {
                debug!("tracing subscriber already set; skipping init");
                Ok(())
            } else {
                Err(Layer0Error::TracingInit(e))
            }
        }
    }
}

async fn wait_for_os_shutdown() -> Layer0ShutdownReason {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let ctrl_c = tokio::signal::ctrl_c();

        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "failed to install SIGTERM handler; falling back to ctrl-c only");
                ctrl_c.await.ok();
                return Layer0ShutdownReason::CtrlC;
            }
        };

        tokio::select! {
            _ = ctrl_c => Layer0ShutdownReason::CtrlC,
            _ = sigterm.recv() => Layer0ShutdownReason::Sigterm,
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.ok();
        Layer0ShutdownReason::CtrlC
    }
}

#[derive(Clone, Debug)]
struct Layer0Orchestrator {
    cfg: Layer0ValidatedConfig,
}

impl Layer0Orchestrator {
    fn new(cfg: Layer0ValidatedConfig) -> Self {
        Self { cfg }
    }

    async fn run<F>(&self, shutdown_source: F) -> Result<ExitCode, Layer0Error>
    where
        F: Future<Output = Layer0ShutdownReason> + Send,
    {
        let gate = Layer0ShutdownGate::new();
        let trigger = gate.trigger();

        let heartbeat_span = tracing::info_span!(
            "heartbeat_worker",
            interval_ms = self.cfg.heartbeat_interval().as_millis() as u64
        );

        let heartbeat_task: JoinHandle<Result<(), Layer0Error>> = tokio::spawn(
            HeartbeatWorker::run(gate.watch(), *self.cfg.heartbeat_interval())
                .instrument(heartbeat_span),
        );

        let mut shutdown_source = shutdown_source;
        tokio::pin!(shutdown_source);

        let mut heartbeat_task = heartbeat_task;

        let shutdown_reason = tokio::select! {
            reason = &mut shutdown_source => {
                reason
            }
            join_outcome = &mut heartbeat_task => {
                match join_outcome {
                    Ok(Ok(())) => {
                        warn!("heartbeat worker exited without shutdown request; requesting internal shutdown");
                        Layer0ShutdownReason::InternalRequest
                    }
                    Ok(Err(e)) => {
                        error!(error = %e, "heartbeat worker returned error; requesting internal shutdown");
                        Layer0ShutdownReason::InternalRequest
                    }
                    Err(join_error) => {
                        error!(error = %join_error, "heartbeat worker join error; requesting internal shutdown");
                        Layer0ShutdownReason::InternalRequest
                    }
                }
            }
        };

        trigger.request_shutdown(shutdown_reason.clone());

        let grace = *self.cfg.shutdown_grace();
        await_task_with_grace("heartbeat_worker", heartbeat_task, grace).await??;

        info!(
            shutdown_reason = shutdown_reason.as_str(),
            "layer0 shutdown complete"
        );
        Ok(ExitCode::SUCCESS)
    }
}

struct HeartbeatWorker;

impl HeartbeatWorker {
    async fn run(
        mut shutdown: Layer0ShutdownWatch,
        interval: Duration,
    ) -> Result<(), Layer0Error> {
        let mut ticker = time::interval(interval);
        ticker.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

        debug!("heartbeat worker started");

        let mut shutdown_wait = shutdown.clone();
        let shutdown_fut = shutdown_wait.wait_for_shutdown();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    trace!("heartbeat tick");
                    debug!(
                        shutdown_requested = shutdown.is_shutdown_requested(),
                        "layer0 alive"
                    );
                }
                reason = &mut shutdown_fut => {
                    let reason = reason?;
                    info!(shutdown_reason = reason.as_str(), "heartbeat worker stopping");
                    return Ok(());
                }
            }
        }
    }
}

async fn await_task_with_grace<T>(
    task_name: &'static str,
    handle: JoinHandle<T>,
    grace: Duration,
) -> Result<T, Layer0Error> {
    let mut handle = handle;

    tokio::select! {
        res = &mut handle => {
            res.map_err(|join_error| Layer0Error::TaskJoin { task_name, join_error })
        }
        _ = time::sleep(grace) => {
            warn!(task_name, waited_ms = grace.as_millis() as u64, "grace timeout; aborting task");
            handle.abort();
            Err(Layer0Error::TaskTimeout { task_name, waited: grace })
        }
    }
}

#[cfg(test)]
mod layer0_runtime_contract_suite {
    use super::*;

    #[traced_test]
    fn layer0_cli_defaults_validate_and_produce_config() {
        let cli = Layer0Cli::from_iter_safe(["matrix-term"].into_iter()).expect("cli parse");
        let cfg = cli.validate().expect("cli validate");

        assert_eq!(*cfg.heartbeat_interval(), Duration::from_millis(500));
        assert_eq!(*cfg.shutdown_grace(), Duration::from_millis(1500));
        assert_eq!(cfg.log().filter_directives().as_deref(), None);
        assert_eq!(*cfg.log().json(), false);
    }

    #[traced_test]
    fn layer0_cli_rejects_zero_heartbeat_interval() {
        let cli = Layer0Cli::from_iter_safe(["matrix-term", "--heartbeat-ms", "0"].into_iter())
            .expect("cli parse");
        let err = cli.validate().expect_err("expected validation failure");

        match err {
            Layer0Error::CliValidation(Layer0CliValidationError::HeartbeatMsMustBePositive) => {}
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[traced_test]
    fn layer0_cli_rejects_zero_shutdown_grace() {
        let cli = Layer0Cli::from_iter_safe(
            ["matrix-term", "--shutdown-grace-ms", "0"].into_iter(),
        )
        .expect("cli parse");
        let err = cli.validate().expect_err("expected validation failure");

        match err {
            Layer0Error::CliValidation(Layer0CliValidationError::ShutdownGraceMsMustBePositive) => {
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[traced_test]
    fn layer0_orchestrator_exits_successfully_on_internal_shutdown_source() {
        let cli = Layer0Cli::from_iter_safe(
            [
                "matrix-term",
                "--heartbeat-ms",
                "10",
                "--shutdown-grace-ms",
                "250",
            ]
            .into_iter(),
        )
        .expect("cli parse");

        let cfg = cli.validate().expect("cli validate");
        install_tracing(cfg.log()).expect("tracing init (idempotent)");

        let rt = build_tokio_runtime().expect("runtime build");
        let code = rt
            .block_on(async move {
                let orchestrator = Layer0Orchestrator::new(cfg);
                orchestrator
                    .run(async {
                        time::sleep(Duration::from_millis(35)).await;
                        Layer0ShutdownReason::InternalRequest
                    })
                    .await
            })
            .expect("orchestrator run");

        assert_eq!(code, ExitCode::SUCCESS);
    }
}
