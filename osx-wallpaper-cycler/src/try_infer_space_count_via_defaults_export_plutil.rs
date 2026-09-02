// ---------------- [ File: osx-wallpaper-cycler/src/try_infer_space_count_via_defaults_export_plutil.rs ]
crate::ix!();

#[cfg(not(target_os = "macos"))]
pub fn try_infer_space_count_via_defaults_export_plutil() -> Option<usize> {
    None
}

#[cfg(target_os = "macos")]
pub fn try_infer_space_count_via_defaults_export_plutil() -> Option<usize> {
    use std::process::Command;

    const DEFAULTS_BIN: &str = "/usr/bin/defaults";
    const PLUTIL_BIN: &str = "/usr/bin/plutil";
    const DOMAIN: &str = "com.apple.spaces";

    tracing::debug!(
        defaults_bin = %DEFAULTS_BIN,
        plutil_bin = %PLUTIL_BIN,
        domain = %DOMAIN,
        "attempting to infer Space count via defaults export + plutil"
    );

    let defaults_out = Command::new(DEFAULTS_BIN)
        .arg("export")
        .arg(DOMAIN)
        .arg("-")
        .output()
        .ok()?;

    tracing::debug!(
        status = %defaults_out.status,
        stdout_bytes = defaults_out.stdout.len(),
        stderr_bytes = defaults_out.stderr.len(),
        "defaults export completed"
    );

    if !defaults_out.status.success() {
        tracing::warn!(
            status = %defaults_out.status,
            stdout_bytes = defaults_out.stdout.len(),
            stderr_bytes = defaults_out.stderr.len(),
            "defaults export failed; cannot infer Space count"
        );
        return None;
    }

    if defaults_out.stdout.is_empty() {
        tracing::warn!("defaults export returned empty stdout; cannot infer Space count");
        return None;
    }

    let dir = tempfile::TempDir::new().ok()?;
    let xml_path = dir.path().join("com.apple.spaces.export.xml");

    if let Err(e) = std::fs::write(&xml_path, &defaults_out.stdout) {
        tracing::warn!(
            path = %xml_path.display(),
            error = %e,
            "failed to write defaults export xml to temp file"
        );
        return None;
    }

    let plutil_out = Command::new(PLUTIL_BIN)
        .arg("-convert")
        .arg("json")
        .arg("-o")
        .arg("-")
        .arg(&xml_path)
        .output()
        .ok()?;

    tracing::debug!(
        status = %plutil_out.status,
        stdout_bytes = plutil_out.stdout.len(),
        stderr_bytes = plutil_out.stderr.len(),
        "plutil conversion completed"
    );

    if !plutil_out.status.success() {
        tracing::warn!(
            status = %plutil_out.status,
            stdout_bytes = plutil_out.stdout.len(),
            stderr_bytes = plutil_out.stderr.len(),
            "plutil conversion failed; cannot infer Space count"
        );
        return None;
    }

    if plutil_out.stdout.is_empty() {
        tracing::warn!("plutil produced empty stdout; cannot infer Space count");
        return None;
    }

    let v: serde_json::Value = match serde_json::from_slice(plutil_out.stdout.as_slice()) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                error = %e,
                stdout_bytes = plutil_out.stdout.len(),
                "failed to decode plutil json output; cannot infer Space count"
            );
            return None;
        }
    };

    let inferred = infer_space_count_from_macos_spaces_defaults_json(&v);

    tracing::info!(
        inferred_space_count = inferred,
        "Space count inference via com.apple.spaces completed"
    );

    inferred
}
