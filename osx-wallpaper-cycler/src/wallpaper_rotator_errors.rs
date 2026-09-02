// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotator_errors.rs ]
crate::ix!();

pub fn wallpaper_rotator_io_failure(
    op: FilesystemOperationKind,
    path: std::path::PathBuf,
    source: std::io::Error,
) -> WallpaperRotatorError {
    WallpaperRotatorError::IoFailure { op, path, source }
}

pub fn wallpaper_rotator_invalid_configuration(
    details: WallpaperRotatorInvalidConfigurationDetails,
) -> WallpaperRotatorError {
    WallpaperRotatorError::InvalidConfiguration { details }
}

pub fn wallpaper_rotator_dropbox_api_failure(
    endpoint: DropboxEndpointKind,
    status: reqwest::StatusCode,
    request_id: Option<String>,
    body: Option<serde_json::Value>,
) -> WallpaperRotatorError {
    WallpaperRotatorError::DropboxApiFailure {
        endpoint,
        status,
        request_id,
        body,
    }
}

pub fn wallpaper_rotator_dropbox_protocol_violation(
    endpoint: DropboxEndpointKind,
    detail: DropboxProtocolViolationDetail,
) -> WallpaperRotatorError {
    WallpaperRotatorError::DropboxProtocolViolation { endpoint, detail }
}

pub fn wallpaper_rotator_applescript_failure(
    action: AppleScriptActionKind,
    error_number: Option<i32>,
) -> WallpaperRotatorError {
    WallpaperRotatorError::AppleScriptFailure { action, error_number }
}

#[cfg(test)]
mod wallpaper_rotator_errors_mapping_contract_suite {
    use super::*;

    #[traced_test]
    fn io_failure_helper_constructs_expected_variant() {
        let path = std::path::PathBuf::from("/tmp/x");
        let src = std::io::Error::new(std::io::ErrorKind::Other, "io");
        let err = wallpaper_rotator_io_failure(FilesystemOperationKind::ReadFile, path.clone(), src);

        match err {
            WallpaperRotatorError::IoFailure { op, path: p, .. } => {
                assert_eq!(op, FilesystemOperationKind::ReadFile);
                assert_eq!(p, path);
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[traced_test]
    fn invalid_configuration_helper_constructs_expected_variant() {
        let details = WallpaperRotatorInvalidConfigurationDetails::new(true, false, false, false, false, false);
        let err = wallpaper_rotator_invalid_configuration(details);

        match err {
            WallpaperRotatorError::InvalidConfiguration { details: d } => {
                assert!(d.any_invalid());
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[traced_test]
    fn dropbox_api_failure_helper_constructs_expected_variant() {
        let err = wallpaper_rotator_dropbox_api_failure(
            DropboxEndpointKind::Download,
            reqwest::StatusCode::CONFLICT,
            Some("RID".to_string()),
            Some(serde_json::json!({"error_summary":"x"})),
        );

        match err {
            WallpaperRotatorError::DropboxApiFailure { endpoint, status, request_id, body } => {
                assert_eq!(endpoint, DropboxEndpointKind::Download);
                assert_eq!(status, reqwest::StatusCode::CONFLICT);
                assert_eq!(request_id.as_deref(), Some("RID"));
                assert!(body.is_some());
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[traced_test]
    fn dropbox_protocol_violation_helper_constructs_expected_variant() {
        let err = wallpaper_rotator_dropbox_protocol_violation(
            DropboxEndpointKind::ListFolder,
            DropboxProtocolViolationDetail::MissingEntries,
        );

        match err {
            WallpaperRotatorError::DropboxProtocolViolation { endpoint, detail } => {
                assert_eq!(endpoint, DropboxEndpointKind::ListFolder);
                assert_eq!(detail, DropboxProtocolViolationDetail::MissingEntries);
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[traced_test]
    fn applescript_failure_helper_constructs_expected_variant() {
        let err = wallpaper_rotator_applescript_failure(AppleScriptActionKind::CountDesktops, Some(-1));
        match err {
            WallpaperRotatorError::AppleScriptFailure { action, error_number } => {
                assert_eq!(action, AppleScriptActionKind::CountDesktops);
                assert_eq!(error_number, Some(-1));
            }
            _ => panic!("unexpected error variant"),
        }
    }
}
