// src/layer2.rs

crate::ix!();

#[derive(Debug)]
pub enum Layer2Error {
    InvalidHomeserverUrl {
        value: String,
        source: UrlParseError,
    },
    UnsupportedHomeserverUrlScheme {
        value: String,
        scheme: String,
    },
    UrlJoinFailed {
        base: String,
        path: &'static str,
        source: UrlParseError,
    },
    HttpClientBuildFailed {
        source: reqwest::Error,
    },
    HttpSendFailed {
        method: &'static str,
        url: String,
        source: reqwest::Error,
    },
    HttpBodyReadFailed {
        url: String,
        source: reqwest::Error,
    },
    UnexpectedHttpStatus {
        url: String,
        status: u16,
        body: String,
    },
    JsonDecodeFailed {
        url: String,
        source: serde_json::Error,
        body: String,
    },
}

impl fmt::Display for Layer2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHomeserverUrl { value, source } => {
                write!(f, "invalid homeserver url {:?}: {}", value, source)
            }
            Self::UnsupportedHomeserverUrlScheme { value, scheme } => write!(
                f,
                "unsupported homeserver url scheme {:?} for {:?} (expected http/https)",
                scheme, value
            ),
            Self::UrlJoinFailed { base, path, source } => write!(
                f,
                "failed to join url {:?} with path {:?}: {}",
                base, path, source
            ),
            Self::HttpClientBuildFailed { source } => {
                write!(f, "failed to build http client: {source}")
            }
            Self::HttpSendFailed {
                method,
                url,
                source,
            } => write!(f, "http request failed ({} {}): {}", method, url, source),
            Self::HttpBodyReadFailed { url, source } => {
                write!(f, "failed reading response body ({url}): {source}")
            }
            Self::UnexpectedHttpStatus { url, status, body } => write!(
                f,
                "unexpected http status ({url}): {} body={:?}",
                status, body
            ),
            Self::JsonDecodeFailed { url, source, body } => write!(
                f,
                "failed to decode json ({url}): {} body={:?}",
                source, body
            ),
        }
    }
}

impl std::error::Error for Layer2Error {}

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Layer2HttpClientConfig {
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: String,
}

impl Layer2HttpClientConfig {
    pub fn standard() -> Self {
        Self::default()
    }

    pub fn for_tests() -> Self {
        Self {
            connect_timeout: Duration::from_millis(250),
            request_timeout: Duration::from_secs(2),
            user_agent: "matrix-term-layer2-tests".to_string(),
        }
    }
}

impl Default for Layer2HttpClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            user_agent: "matrix-term-layer2".to_string(),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum Layer2AuthMaterial {
    None,
    BearerToken { access_token: String },
}

impl Layer2AuthMaterial {
    pub fn none() -> Self {
        Self::None
    }

    pub fn bearer_token(access_token: String) -> Self {
        Self::BearerToken { access_token }
    }

    pub fn from_optional_access_token(access_token: Option<String>) -> Self {
        match access_token {
            Some(t) => Self::BearerToken { access_token: t },
            None => Self::None,
        }
    }

    fn apply(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self {
            Self::None => rb,
            Self::BearerToken { access_token } => rb.bearer_auth(access_token),
        }
    }
}

impl fmt::Debug for Layer2AuthMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.debug_tuple("Layer2AuthMaterial::None").finish(),
            Self::BearerToken { .. } => f
                .debug_struct("Layer2AuthMaterial::BearerToken")
                .field("access_token", &"<redacted>")
                .finish(),
        }
    }
}

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub struct Layer2HttpClient {
    homeserver_url: Url,
    auth: Layer2AuthMaterial,
    client: reqwest::Client,
    config: Layer2HttpClientConfig,
}

impl fmt::Debug for Layer2HttpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layer2HttpClient")
            .field("homeserver_url", &self.homeserver_url)
            .field("auth", &self.auth)
            .field("config", &self.config)
            .finish()
    }
}

impl Layer2HttpClient {
    pub fn new(
        homeserver_url: &str,
        auth: Layer2AuthMaterial,
        cfg: &Layer2HttpClientConfig,
    ) -> Result<Self, Layer2Error> {
        Layer2RustlsCryptoProviderInstaller::ensure_ring_is_process_default();

        let parsed = Url::parse(homeserver_url).map_err(|e| Layer2Error::InvalidHomeserverUrl {
            value: homeserver_url.to_string(),
            source: e,
        })?;

        match parsed.scheme() {
            "http" | "https" => {}
            other => {
                return Err(Layer2Error::UnsupportedHomeserverUrlScheme {
                    value: homeserver_url.to_string(),
                    scheme: other.to_string(),
                });
            }
        }

        let client = reqwest::Client::builder()
            .connect_timeout(*cfg.connect_timeout())
            .timeout(*cfg.request_timeout())
            .user_agent(cfg.user_agent().clone())
            .build()
            .map_err(|e| Layer2Error::HttpClientBuildFailed { source: e })?;

        debug!(
            homeserver_url = %parsed,
            connect_timeout_ms = cfg.connect_timeout().as_millis() as u64,
            request_timeout_ms = cfg.request_timeout().as_millis() as u64,
            "layer2 http client constructed"
        );

        Ok(Self {
            homeserver_url: parsed,
            auth,
            client,
            config: cfg.clone(),
        })
    }

    pub async fn probe_matrix_client_versions(
        &self,
    ) -> Result<Layer2MatrixClientVersions, Layer2Error> {
        const PATH: &str = "/_matrix/client/versions";

        let url = self
            .homeserver_url
            .join(PATH)
            .map_err(|e| Layer2Error::UrlJoinFailed {
                base: self.homeserver_url.to_string(),
                path: PATH,
                source: e,
            })?;

        let span = tracing::info_span!("layer2_probe_matrix_client_versions", url = %url);
        async move {
            debug!("sending versions probe");

            let req = self.client.get(url.clone());
            let req = self.auth.apply(req);

            let res = req
                .send()
                .await
                .map_err(|e| Layer2Error::HttpSendFailed {
                    method: "GET",
                    url: url.to_string(),
                    source: e,
                })?;

            let status = res.status();
            let body = res.text().await.map_err(|e| Layer2Error::HttpBodyReadFailed {
                url: url.to_string(),
                source: e,
            })?;

            if !status.is_success() {
                warn!(
                    http_status = status.as_u16(),
                    body_len = body.len(),
                    "versions probe returned non-success"
                );
                return Err(Layer2Error::UnexpectedHttpStatus {
                    url: url.to_string(),
                    status: status.as_u16(),
                    body: truncate_for_error(&body, 16 * 1024),
                });
            }

            let wire: Layer2MatrixVersionsWire =
                serde_json::from_str(&body).map_err(|e| Layer2Error::JsonDecodeFailed {
                    url: url.to_string(),
                    source: e,
                    body: truncate_for_error(&body, 16 * 1024),
                })?;

            let unstable_features = if wire.unstable_features.is_empty() {
                None
            } else {
                Some(wire.unstable_features)
            };

            info!(
                versions_count = wire.versions.len(),
                unstable_features_count = unstable_features.as_ref().map(|m| m.len()).unwrap_or(0),
                "versions probe succeeded"
            );

            Ok(Layer2MatrixClientVersions {
                versions: wire.versions,
                unstable_features,
            })
        }
        .instrument(span)
        .await
    }
}

fn truncate_for_error(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    let mut out = s[..max_bytes].to_string();
    out.push_str("…<truncated>");
    out
}

#[derive(Clone, Debug, Deserialize)]
struct Layer2MatrixVersionsWire {
    versions: Vec<String>,
    #[serde(default)]
    unstable_features: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Layer2MatrixClientVersions {
    versions: Vec<String>,
    unstable_features: Option<BTreeMap<String, bool>>,
}

pub struct Layer2RustlsCryptoProviderInstaller;

impl Layer2RustlsCryptoProviderInstaller {
    pub fn ensure_ring_is_process_default() {
        if rustls::crypto::CryptoProvider::get_default().is_some() {
            debug!("rustls crypto provider already installed; leaving as-is");
            return;
        }

        let provider = rustls::crypto::ring::default_provider();

        match provider.install_default() {
            Ok(()) => {
                info!("installed rustls ring crypto provider as process default");
            }
            Err(_already_installed_provider) => {
                debug!("rustls crypto provider was installed concurrently; continuing");
            }
        }
    }
}


#[cfg(test)]
mod layer2_http_reachability_contract_suite {
    use super::*;

    use std::net::SocketAddr;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::oneshot;

    #[derive(Clone, Debug)]
    struct Layer2TestResponse {
        status: u16,
        content_type: &'static str,
        body: String,
    }

    #[derive(Clone, Debug)]
    struct Layer2TestServerPlan {
        require_authorization: Option<String>,
        ok: Layer2TestResponse,
        unauthorized: Layer2TestResponse,
    }

    struct Layer2TestHttpServer {
        base_url: String,
        shutdown_tx: oneshot::Sender<()>,
        task: JoinHandle<()>,
    }

    impl Layer2TestHttpServer {
        async fn start(plan: Layer2TestServerPlan) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind test listener");

            let addr: SocketAddr = listener.local_addr().expect("local addr");
            let base_url = format!("http://{}", addr);

            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
            let plan = Arc::new(plan);

            let task = tokio::spawn(Self::serve(listener, plan, shutdown_rx));

            Self {
                base_url,
                shutdown_tx,
                task,
            }
        }

        fn base_url(&self) -> &str {
            &self.base_url
        }

        async fn shutdown(self) {
            let _ = self.shutdown_tx.send(());
            let _ = self.task.await;
        }

        async fn serve(
            listener: TcpListener,
            plan: Arc<Layer2TestServerPlan>,
            mut shutdown_rx: oneshot::Receiver<()>,
        ) {
            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        debug!("test server shutdown requested");
                        break;
                    }
                    accept = listener.accept() => {
                        match accept {
                            Ok((stream, peer)) => {
                                trace!(peer = %peer, "accepted test connection");
                                let plan = plan.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = Self::handle_connection(stream, plan).await {
                                        warn!(error = %e, "test connection handler error");
                                    }
                                });
                            }
                            Err(e) => {
                                warn!(error = %e, "accept failed");
                                break;
                            }
                        }
                    }
                }
            }
        }

        async fn handle_connection(
            mut stream: TcpStream,
            plan: Arc<Layer2TestServerPlan>,
        ) -> Result<(), io::Error> {
            let req = read_http_request(&mut stream).await?;
            let auth_ok = match plan.require_authorization.as_ref() {
                None => true,
                Some(expected) => req
                    .headers
                    .get("authorization")
                    .map(|v| v == expected)
                    .unwrap_or(false),
            };

            let response = if auth_ok {
                plan.ok.clone()
            } else {
                plan.unauthorized.clone()
            };

            write_http_response(&mut stream, response).await?;
            Ok(())
        }
    }

    #[derive(Clone, Debug)]
    struct Layer2ParsedRequest {
        method: String,
        path: String,
        headers: BTreeMap<String, String>,
    }

    async fn read_http_request(stream: &mut TcpStream) -> Result<Layer2ParsedRequest, io::Error> {
        let mut buf = Vec::<u8>::new();
        let mut tmp = [0u8; 1024];

        loop {
            let n = stream.read(&mut tmp).await?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&tmp[..n]);

            if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }

            if buf.len() > 64 * 1024 {
                break;
            }
        }

        let text = String::from_utf8_lossy(&buf);
        let mut lines = text.split("\r\n");

        let request_line = lines.next().unwrap_or_default();
        let mut parts = request_line.split_whitespace();

        let method = parts.next().unwrap_or("").to_string();
        let path = parts.next().unwrap_or("").to_string();

        let mut headers = BTreeMap::<String, String>::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some((k, v)) = line.split_once(':') {
                headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
            }
        }

        Ok(Layer2ParsedRequest {
            method,
            path,
            headers,
        })
    }

    async fn write_http_response(
        stream: &mut TcpStream,
        response: Layer2TestResponse,
    ) -> Result<(), io::Error> {
        let status_text = match response.status {
            200 => "OK",
            401 => "Unauthorized",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "OK",
        };

        let body_bytes = response.body.as_bytes();
        let header = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            response.status,
            status_text,
            response.content_type,
            body_bytes.len()
        );

        stream.write_all(header.as_bytes()).await?;
        stream.write_all(body_bytes).await?;
        stream.flush().await?;
        Ok(())
    }

    fn build_test_runtime() -> TokioRuntime {
        TokioRuntimeBuilder::new_multi_thread()
            .enable_all()
            .thread_name("layer2-tests")
            .build()
            .expect("build tokio runtime")
    }

    #[traced_test]
    fn layer2_versions_probe_succeeds_and_parses_versions() {
        let rt = build_test_runtime();

        rt.block_on(async {
            let plan = Layer2TestServerPlan {
                require_authorization: None,
                ok: Layer2TestResponse {
                    status: 200,
                    content_type: "application/json",
                    body: r#"{"versions":["r0.6.1","v1.11"],"unstable_features":{"org.matrix.msc3575":true}}"#.to_string(),
                },
                unauthorized: Layer2TestResponse {
                    status: 401,
                    content_type: "text/plain",
                    body: "unauthorized".to_string(),
                },
            };

            let server = Layer2TestHttpServer::start(plan).await;

            let cfg = Layer2HttpClientConfig::for_tests();
            let client = Layer2HttpClient::new(
                server.base_url(),
                Layer2AuthMaterial::none(),
                &cfg,
            )
            .expect("client build");

            let versions = client
                .probe_matrix_client_versions()
                .await
                .expect("probe ok");

            assert_eq!(
                versions.versions(),
                &vec!["r0.6.1".to_string(), "v1.11".to_string()]
            );

            let unstable = versions.unstable_features().as_ref().expect("unstable");
            assert_eq!(unstable.get("org.matrix.msc3575").copied(), Some(true));

            server.shutdown().await;
        });
    }

    #[traced_test]
    fn layer2_non_success_status_returns_unexpected_http_status_error() {
        let rt = build_test_runtime();

        rt.block_on(async {
            let plan = Layer2TestServerPlan {
                require_authorization: None,
                ok: Layer2TestResponse {
                    status: 404,
                    content_type: "text/plain",
                    body: "not found".to_string(),
                },
                unauthorized: Layer2TestResponse {
                    status: 401,
                    content_type: "text/plain",
                    body: "unauthorized".to_string(),
                },
            };

            let server = Layer2TestHttpServer::start(plan).await;

            let cfg = Layer2HttpClientConfig::for_tests();
            let client = Layer2HttpClient::new(
                server.base_url(),
                Layer2AuthMaterial::none(),
                &cfg,
            )
            .expect("client build");

            let err = client
                .probe_matrix_client_versions()
                .await
                .expect_err("expected error");

            match err {
                Layer2Error::UnexpectedHttpStatus { status, body, .. } => {
                    assert_eq!(status, 404);
                    assert!(body.contains("not found"));
                }
                other => panic!("unexpected error variant: {other}"),
            }

            server.shutdown().await;
        });
    }

    #[traced_test]
    fn layer2_invalid_json_returns_json_decode_failed_error() {
        let rt = build_test_runtime();

        rt.block_on(async {
            let plan = Layer2TestServerPlan {
                require_authorization: None,
                ok: Layer2TestResponse {
                    status: 200,
                    content_type: "application/json",
                    body: r#"{"versions": [}"#.to_string(),
                },
                unauthorized: Layer2TestResponse {
                    status: 401,
                    content_type: "text/plain",
                    body: "unauthorized".to_string(),
                },
            };

            let server = Layer2TestHttpServer::start(plan).await;

            let cfg = Layer2HttpClientConfig::for_tests();
            let client = Layer2HttpClient::new(
                server.base_url(),
                Layer2AuthMaterial::none(),
                &cfg,
            )
            .expect("client build");

            let err = client
                .probe_matrix_client_versions()
                .await
                .expect_err("expected json decode error");

            match err {
                Layer2Error::JsonDecodeFailed { body, .. } => {
                    assert!(body.contains(r#""versions""#));
                }
                other => panic!("unexpected error variant: {other}"),
            }

            server.shutdown().await;
        });
    }

    #[traced_test]
    fn layer2_bearer_auth_is_required_when_server_demands_it() {
        let rt = build_test_runtime();

        rt.block_on(async {
            let plan = Layer2TestServerPlan {
                require_authorization: Some("Bearer TOKEN123".to_string()),
                ok: Layer2TestResponse {
                    status: 200,
                    content_type: "application/json",
                    body: r#"{"versions":["v1.1"]}"#.to_string(),
                },
                unauthorized: Layer2TestResponse {
                    status: 401,
                    content_type: "text/plain",
                    body: "unauthorized".to_string(),
                },
            };

            let server = Layer2TestHttpServer::start(plan).await;

            let cfg = Layer2HttpClientConfig::for_tests();

            let unauth_client = Layer2HttpClient::new(
                server.base_url(),
                Layer2AuthMaterial::none(),
                &cfg,
            )
            .expect("client build");

            let err = unauth_client
                .probe_matrix_client_versions()
                .await
                .expect_err("expected unauthorized");

            match err {
                Layer2Error::UnexpectedHttpStatus { status, .. } => assert_eq!(status, 401),
                other => panic!("unexpected error variant: {other}"),
            }

            let auth_client = Layer2HttpClient::new(
                server.base_url(),
                Layer2AuthMaterial::bearer_token("TOKEN123".to_string()),
                &cfg,
            )
            .expect("client build");

            let versions = auth_client
                .probe_matrix_client_versions()
                .await
                .expect("authorized probe ok");

            assert_eq!(versions.versions(), &vec!["v1.1".to_string()]);

            server.shutdown().await;
        });
    }
}
