// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotator_error.rs ]
crate::ix!();

#[derive(Debug)]
pub enum WallpaperRotatorError {
    UnsupportedPlatform,
    HomeDirectoryUnavailable,
    InvalidConfiguration {
        details: WallpaperRotatorInvalidConfigurationDetails,
    },
    IoFailure {
        op: FilesystemOperationKind,
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    TomlDecodeFailure {
        path: std::path::PathBuf,
        source: toml::de::Error,
    },
    JsonDecodeFailure {
        source: serde_json::Error,
    },
    JsonEncodeFailure {
        source: serde_json::Error,
    },
    HttpFailure {
        source: reqwest::Error,
    },
    DropboxApiFailure {
        endpoint: DropboxEndpointKind,
        status: reqwest::StatusCode,
        request_id: Option<String>,
        body: Option<serde_json::Value>,
    },
    DropboxProtocolViolation {
        endpoint: DropboxEndpointKind,
        detail: DropboxProtocolViolationDetail,
    },
    AppleScriptFailure {
        action: AppleScriptActionKind,
        error_number: Option<i32>,
    },
    NoEligibleWallpapers {
        scanned_roots: Vec<String>,
        allowed_extensions: Vec<String>,
    },
    CacheStateCorrupt {
        path: std::path::PathBuf,
    },
    ConcurrencyPermitUnavailable,
    JoinFailure,
}

impl std::error::Error for WallpaperRotatorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoFailure { source, .. } => Some(source),
            Self::TomlDecodeFailure { source, .. } => Some(source),
            Self::JsonDecodeFailure { source } => Some(source),
            Self::JsonEncodeFailure { source } => Some(source),
            Self::HttpFailure { source } => Some(source),
            _ => None,
        }
    }
}

impl std::fmt::Display for WallpaperRotatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(f, "unsupported platform"),
            Self::HomeDirectoryUnavailable => write!(f, "home directory unavailable"),
            Self::InvalidConfiguration { details } => write!(f, "invalid configuration: {details}"),
            Self::IoFailure { op, path, source } => {
                write!(f, "{op} failed for path {}: {}", path.display(), source)
            }
            Self::TomlDecodeFailure { path, source } => {
                write!(f, "failed to decode config TOML at {}: {}", path.display(), source)
            }
            Self::JsonDecodeFailure { source } => write!(f, "failed to decode JSON: {source}"),
            Self::JsonEncodeFailure { source } => write!(f, "failed to encode JSON: {source}"),
            Self::HttpFailure { source } => write!(f, "http failure: {source}"),
            Self::DropboxApiFailure {
                endpoint,
                status,
                request_id,
                body,
            } => {
                if let Some(req_id) = request_id.as_ref() {
                    if let Some(body) = body.as_ref() {
                        write!(
                            f,
                            "dropbox api failure at {endpoint} (status {status}, request_id {req_id}): {body}"
                        )
                    } else {
                        write!(
                            f,
                            "dropbox api failure at {endpoint} (status {status}, request_id {req_id})"
                        )
                    }
                } else if let Some(body) = body.as_ref() {
                    write!(f, "dropbox api failure at {endpoint} (status {status}): {body}")
                } else {
                    write!(f, "dropbox api failure at {endpoint} (status {status})")
                }
            }
            Self::DropboxProtocolViolation { endpoint, detail } => {
                write!(f, "dropbox protocol violation at {endpoint}: {detail}")
            }
            Self::AppleScriptFailure {
                action,
                error_number,
            } => {
                if let Some(n) = error_number {
                    write!(f, "{action} failed with error number {n}")
                } else {
                    write!(f, "{action} failed")
                }
            }
            Self::NoEligibleWallpapers {
                scanned_roots,
                allowed_extensions,
            } => write!(
                f,
                "no eligible wallpapers found (roots: {:?}, allowed_extensions: {:?})",
                scanned_roots, allowed_extensions
            ),
            Self::CacheStateCorrupt { path } => {
                write!(f, "cache state is corrupt or unreadable: {}", path.display())
            }
            Self::ConcurrencyPermitUnavailable => write!(f, "concurrency permit unavailable"),
            Self::JoinFailure => write!(f, "task join failure"),
        }
    }
}

#[cfg(test)]
mod wallpaper_rotator_error_display_and_source_contract_suite {
    use super::*;

    #[traced_test]
    fn display_is_stable_for_simple_variants() {
        assert_eq!(
            WallpaperRotatorError::UnsupportedPlatform.to_string(),
            "unsupported platform"
        );

        assert_eq!(
            WallpaperRotatorError::HomeDirectoryUnavailable.to_string(),
            "home directory unavailable"
        );

        assert_eq!(
            WallpaperRotatorError::ConcurrencyPermitUnavailable.to_string(),
            "concurrency permit unavailable"
        );

        assert_eq!(
            WallpaperRotatorError::JoinFailure.to_string(),
            "task join failure"
        );
    }

    #[traced_test]
    fn io_failure_display_includes_operation_and_path_and_has_source() {
        let err = WallpaperRotatorError::IoFailure {
            op: FilesystemOperationKind::ReadFile,
            path: std::path::PathBuf::from("/tmp/missing"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "nf"),
        };

        let s = err.to_string();
        assert!(s.contains("io.read_file failed for path"));
        assert!(s.contains("/tmp/missing"));

        let src = err.source();
        assert!(src.is_some());
    }

    #[traced_test]
    fn dropbox_api_failure_display_includes_request_id_when_present() {
        let err = WallpaperRotatorError::DropboxApiFailure {
            endpoint: DropboxEndpointKind::Download,
            status: reqwest::StatusCode::CONFLICT,
            request_id: Some("RID123".to_string()),
            body: Some(serde_json::json!({"error_summary":"conflict"})),
        };

        let s = err.to_string();
        assert!(s.contains("dropbox api failure at dropbox.files.download"));
        assert!(s.contains("RID123"));
        assert!(s.contains("conflict"));
    }

    #[traced_test]
    fn no_eligible_wallpapers_display_mentions_roots_and_extensions() {
        let err = WallpaperRotatorError::NoEligibleWallpapers {
            scanned_roots: vec!["/A".to_string(), "/B".to_string()],
            allowed_extensions: vec!["jpg".to_string(), "png".to_string()],
        };

        let s = err.to_string();
        assert!(s.contains("no eligible wallpapers found"));
        assert!(s.contains("/A"));
        assert!(s.contains("jpg"));
    }
}
